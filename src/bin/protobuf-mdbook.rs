//! `protobuf-mdbook` — standalone CLI for generating mdBook docs from protobuf.

#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use protobuf_mdbook::input::{Compiler, ResolveArgs, resolve_inputs};
use protobuf_mdbook::options::{CliOptionsInput, EscapeTags, Layout, build_options_from_cli};
use protobuf_mdbook::{generate_from_input, write_generated_files};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum CompilerArg {
    #[default]
    Buf,
    Protoc,
}

impl From<CompilerArg> for Compiler {
    fn from(value: CompilerArg) -> Self {
        match value {
            CompilerArg::Buf => Compiler::Buf,
            CompilerArg::Protoc => Compiler::Protoc,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum LayoutArg {
    #[default]
    Package,
    Entity,
    Split,
}

impl From<LayoutArg> for Layout {
    fn from(value: LayoutArg) -> Self {
        match value {
            LayoutArg::Package => Layout::Package,
            LayoutArg::Entity => Layout::Entity,
            LayoutArg::Split => Layout::Split,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum IgnoreArg {
    #[default]
    Git,
    None,
}

#[derive(Parser, Debug)]
#[command(
    name = "protobuf-mdbook",
    about = "Generate mdBook / Markdown documentation from protobuf schemas",
    disable_version_flag = true
)]
struct Cli {
    /// Output root directory (like protoc `--mdbook_out`).
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Protoc import path / proto search path for companion docs (repeatable).
    #[arg(short = 'I', long = "proto-path")]
    proto_path: Vec<PathBuf>,

    /// Scaffold a full mdBook project (theme, `book.toml`, package SUMMARY, README).
    #[arg(long)]
    init: bool,

    /// Emit SUMMARY without mdBook scaffold.
    #[arg(long)]
    summary: bool,

    /// Documentation page layout (default: package).
    #[arg(long, value_enum, default_value_t = LayoutArg::Package)]
    layout: LayoutArg,

    /// Subdirectory under the output root for generated files (default `.`).
    #[arg(long = "book-root")]
    book_root: Option<String>,

    /// Book root or `book.toml`; loads `[book] src` via mdbook-core on refresh.
    #[arg(long)]
    book: Option<String>,

    /// API markdown directory relative to book root (default `src/packages`).
    #[arg(long = "markdown-root")]
    markdown_root: Option<String>,

    /// SUMMARY path when `--summary` / `--init` (default `src/SUMMARY.md`).
    #[arg(long = "summary-path")]
    summary_path: Option<String>,

    /// `book.toml` title when `--init` (default **Protobuf documentation**).
    #[arg(long)]
    title: Option<String>,

    /// Whether `--init` emits `.gitignore` (default: git).
    #[arg(long, value_enum, default_value_t = IgnoreArg::Git)]
    ignore: IgnoreArg,

    /// Skip protobuf Highlight.js grammar in `theme/index.hbs` (`--init` only).
    #[arg(long = "no-proto-highlight")]
    no_proto_highlight: bool,

    /// Skip CEL Highlight.js grammar in `theme/index.hbs` (`--init` only).
    #[arg(long = "no-cel-highlight")]
    no_cel_highlight: bool,

    /// Disable copying companion `.md` beside protos and companion SUMMARY entries.
    #[arg(long = "no-proto-markdown")]
    no_proto_markdown: bool,

    /// Rewrite HTML-like `<…>` in leading-comment prose (bare flag: backticks).
    #[arg(long = "escape-tags", num_args = 0..=1, default_missing_value = "backticks")]
    escape_tags: Option<String>,

    /// Prebuilt `FileDescriptorSet` (`.binpb`, `.fds`, …); repeatable.
    #[arg(long = "descriptor-set")]
    descriptor_set: Vec<PathBuf>,

    /// Compiler for `.proto` inputs.
    #[arg(long = "compiler", default_value = "buf", value_enum)]
    compiler: CompilerArg,

    /// Path to `protoc` (default: PATH, then vendored protoc).
    #[arg(long = "protoc")]
    protoc: Option<PathBuf>,

    /// Path to `buf` (default: PATH).
    #[arg(long = "buf")]
    buf: Option<PathBuf>,

    /// Export BSR deps via `buf export` for protoc `-I` (directory path).
    #[arg(long = "proto-deps-export")]
    proto_deps_export: Option<PathBuf>,

    /// Read `CodeGeneratorRequest` from stdin instead of compiling inputs.
    #[arg(long = "request")]
    request: bool,

    /// Proto files, directories, or Buf module roots.
    #[arg(value_name = "INPUT")]
    inputs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!(
            "protobuf-mdbook {} (mdbook {})",
            env!("CARGO_PKG_VERSION"),
            protobuf_mdbook::mdbook_version()
        );
        return Ok(());
    }

    let cli = Cli::parse();
    if cli.request && !cli.inputs.is_empty() {
        bail!("--request cannot be combined with INPUT paths");
    }

    let mut options = build_options_from_cli(cli_options_input(&cli)?)?;
    if let Some(out) = &cli.output {
        options.mdbook_out = Some(out.to_string_lossy().into_owned());
    }

    let resolve = ResolveArgs {
        compiler: cli.compiler.into(),
        descriptor_sets: cli.descriptor_set,
        inputs: cli.inputs,
        proto_paths: cli.proto_path,
        protoc_path: cli.protoc,
        buf_path: cli.buf,
        proto_deps_export: cli.proto_deps_export,
        from_request: cli.request,
    };

    let resolved = resolve_inputs(&resolve)?;
    let generate_input = resolved.into_generate_input(options.clone());

    let out_root = output_root(cli.output.as_deref(), &options)?;
    if out_root.exists() && options.init {
        std::fs::remove_dir_all(&out_root)
            .with_context(|| format!("clear output before init at {}", out_root.display()))?;
    }
    std::fs::create_dir_all(&out_root)
        .with_context(|| format!("create output directory {}", out_root.display()))?;

    let pairs = generate_from_input(&generate_input)?;
    write_generated_files(&out_root, &pairs)?;
    Ok(())
}

fn cli_options_input(cli: &Cli) -> Result<CliOptionsInput> {
    Ok(CliOptionsInput {
        init: cli.init,
        summary: cli.summary,
        layout: cli.layout.into(),
        book_root: cli.book_root.clone(),
        book: cli.book.clone(),
        markdown_root: cli.markdown_root.clone(),
        summary_path: cli.summary_path.clone(),
        title: cli.title.clone(),
        ignore_git: matches!(cli.ignore, IgnoreArg::Git),
        no_proto_highlight: cli.no_proto_highlight,
        no_cel_highlight: cli.no_cel_highlight,
        no_proto_markdown: cli.no_proto_markdown,
        escape_tags: parse_escape_tags(cli.escape_tags.as_deref())?,
    })
}

fn parse_escape_tags(value: Option<&str>) -> Result<EscapeTags> {
    match value {
        None => Ok(EscapeTags::Off),
        Some("backticks") => Ok(EscapeTags::Backticks),
        Some("entities") => Ok(EscapeTags::Entities),
        Some(other) => bail!("unknown --escape-tags value {other:?}; use backticks or entities"),
    }
}

fn output_root(
    cli_output: Option<&std::path::Path>,
    options: &protobuf_mdbook::options::Options,
) -> Result<PathBuf> {
    if let Some(out) = cli_output {
        return Ok(out.to_path_buf());
    }
    if let Some(mdbook_out) = &options.mdbook_out {
        return Ok(PathBuf::from(mdbook_out));
    }
    if options.book.is_some() {
        bail!("`-o / --output` is required when `--book` is set");
    }
    bail!("`-o / --output` is required");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_escape_tags_values() {
        assert_eq!(parse_escape_tags(None).unwrap(), EscapeTags::Off);
        assert_eq!(
            parse_escape_tags(Some("backticks")).unwrap(),
            EscapeTags::Backticks
        );
        assert_eq!(
            parse_escape_tags(Some("entities")).unwrap(),
            EscapeTags::Entities
        );
        assert!(parse_escape_tags(Some("bad")).is_err());
    }

    #[test]
    fn cli_options_input_maps_ignore() {
        let cli = Cli::parse_from(["protobuf-mdbook", "-o", "out", "--ignore", "none"]);
        let input = cli_options_input(&cli).unwrap();
        assert!(!input.ignore_git);
    }
}
