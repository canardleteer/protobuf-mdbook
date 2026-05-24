//! `protobuf-mdbook` — standalone CLI for generating mdBook docs from protobuf.

#![forbid(unsafe_code)]

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use protobuf_mdbook::input::{Compiler, ResolveArgs, resolve_inputs};
use protobuf_mdbook::options::parse_parameter;
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

    /// Plugin option (repeatable; same values as `--mdbook_opt`).
    #[arg(long = "opt")]
    opts: Vec<String>,

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

    let parameter = merge_opts(&cli.opts);

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
    let mut generate_input = resolved.into_generate_input(parameter);
    merge_cli_output(&mut generate_input, cli.output.as_deref())?;

    let out_root = output_root(cli.output.as_deref(), &generate_input)?;
    if out_root.exists() && opts_init(&generate_input)? {
        std::fs::remove_dir_all(&out_root)
            .with_context(|| format!("clear output before init at {}", out_root.display()))?;
    }
    std::fs::create_dir_all(&out_root)
        .with_context(|| format!("create output directory {}", out_root.display()))?;

    let pairs = generate_from_input(&generate_input)?;
    write_generated_files(&out_root, &pairs)?;
    Ok(())
}

fn merge_opts(opts: &[String]) -> Option<String> {
    let joined: Vec<_> = opts
        .iter()
        .flat_map(|o| o.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if joined.is_empty() {
        None
    } else {
        Some(joined.join(","))
    }
}

fn merge_cli_output(
    input: &mut protobuf_mdbook::GenerateInput,
    output: Option<&std::path::Path>,
) -> Result<()> {
    let Some(out) = output else {
        return Ok(());
    };
    let out = out.to_string_lossy();
    let mut param = input.parameter.take().unwrap_or_default();
    if !param.contains("mdbook_out=") {
        if !param.is_empty() {
            param.push(',');
        }
        param.push_str(&format!("mdbook_out={out}"));
    }
    input.parameter = if param.is_empty() { None } else { Some(param) };
    Ok(())
}

fn output_root(
    cli_output: Option<&std::path::Path>,
    input: &protobuf_mdbook::GenerateInput,
) -> Result<PathBuf> {
    if let Some(out) = cli_output {
        return Ok(out.to_path_buf());
    }
    let opts = parse_parameter(&input.parameter)?;
    if let Some(mdbook_out) = opts.mdbook_out {
        return Ok(PathBuf::from(mdbook_out));
    }
    if opts.book.is_some() {
        bail!("`-o / --output` is required unless `mdbook_out=` is set in --opt");
    }
    bail!("`-o / --output` is required");
}

fn opts_init(input: &protobuf_mdbook::GenerateInput) -> Result<bool> {
    Ok(parse_parameter(&input.parameter)?.init)
}
