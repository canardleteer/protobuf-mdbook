//! Guided `./api-book` generation and link checks.

use crate::ci::{buf_command, build_plugin, release_bin};
use crate::workspace::WORKSPACE_ROOT;
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repo-local mdBook output (`--mdbook_out` / `-o`).
const API_BOOK_DIR: &str = "api-book";

/// Which binary drives guided `book-*` generation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum GeneratorArg {
    /// `protoc` + `protoc-gen-mdbook` plugin (default; matches CI).
    #[default]
    Protoc,
    /// `protobuf-mdbook` CLI (default buf compiler on `examples/proto/`).
    Cli,
}

fn api_book() -> PathBuf {
    Path::new(WORKSPACE_ROOT).join(API_BOOK_DIR)
}

/// `--mdbook_opt` for guided `./api-book` runs.
fn mdbook_opt(layout: &str, summary: bool, init: bool, with_book: bool) -> String {
    let mut opt = if init {
        format!("init,layout={layout}")
    } else {
        format!("layout={layout}")
    };
    if summary {
        opt.push_str(",summary");
    }
    if with_book {
        let book = api_book();
        let book_s = book.to_string_lossy();
        opt.push_str(&format!(",book={book_s},mdbook_out={book_s}"));
    }
    opt
}

fn example_proto_paths() -> Vec<PathBuf> {
    protobuf_mdbook::examples::example_proto_inputs()
        .iter()
        .map(|rel| PathBuf::from(*rel))
        .collect()
}

fn prepare_example_output(out_dir: &Path, mdbook_opt: &str) -> Result<Vec<PathBuf>> {
    let inputs = example_proto_paths();
    if inputs.is_empty() {
        bail!("no example proto inputs (see protobuf_mdbook::examples::EXAMPLE_PROTO_INPUTS)");
    }
    if out_dir.exists() && mdbook_opt.contains("init") {
        std::fs::remove_dir_all(out_dir).context("clear output before init")?;
    }
    std::fs::create_dir_all(out_dir)?;
    Ok(inputs)
}

fn proto_deps_export() -> PathBuf {
    Path::new(WORKSPACE_ROOT).join("target/proto-deps")
}

fn ensure_proto_deps_export() -> Result<PathBuf> {
    buf_command()?;
    protobuf_mdbook::proto_deps::ensure_proto_deps_export(
        &protobuf_mdbook::examples::examples_proto_dir(),
        &proto_deps_export(),
        true,
    )
}

fn protoc_bin() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("PROTOC_BIN")))
}

fn run_protoc_on_examples(out_dir: &Path, mdbook_opt: &str) -> Result<()> {
    build_plugin()?;
    let protoc = protoc_bin()?;
    let plugin = release_bin("protoc-gen-mdbook")?;
    let proto_root = protobuf_mdbook::examples::examples_proto_dir();
    let inputs = prepare_example_output(out_dir, mdbook_opt)?;
    eprintln!(
        "xtask: protoc {} proto file(s) → {} (opt={mdbook_opt})",
        inputs.len(),
        out_dir.display()
    );

    let deps = ensure_proto_deps_export()?;
    let mut cmd = Command::new(protoc);
    cmd.current_dir(&proto_root)
        .arg("-I")
        .arg(".")
        .arg("-I")
        .arg(&deps)
        .arg(format!("--plugin=protoc-gen-mdbook={}", plugin.display()))
        .arg(format!("--mdbook_out={}", out_dir.display()))
        .arg(format!("--mdbook_opt={mdbook_opt}"));
    for rel in &inputs {
        cmd.arg(rel);
    }
    let status = cmd.status().context("protoc")?;
    if !status.success() {
        bail!("protoc failed");
    }
    Ok(())
}

fn run_cli_on_examples(out_dir: &Path, mdbook_opt: &str) -> Result<()> {
    build_plugin()?;
    buf_command()?;
    let cli = release_bin("protobuf-mdbook")?;
    let proto_root = protobuf_mdbook::examples::examples_proto_dir();
    let inputs = prepare_example_output(out_dir, mdbook_opt)?;
    eprintln!(
        "xtask: protobuf-mdbook {} proto file(s) → {} (opt={mdbook_opt})",
        inputs.len(),
        out_dir.display()
    );

    let cli_args = protobuf_mdbook::options::parameter_to_cli_args(mdbook_opt)?;

    let mut cmd = Command::new(cli);
    cmd.current_dir(&proto_root).arg("-o").arg(out_dir);
    for arg in &cli_args {
        cmd.arg(arg);
    }
    cmd.arg("-I").arg(".");
    for rel in &inputs {
        cmd.arg(rel);
    }
    let status = cmd.status().context("protobuf-mdbook")?;
    if !status.success() {
        bail!("protobuf-mdbook failed");
    }
    Ok(())
}

fn run_on_examples(out_dir: &Path, mdbook_opt: &str, generator: GeneratorArg) -> Result<()> {
    match generator {
        GeneratorArg::Protoc => run_protoc_on_examples(out_dir, mdbook_opt),
        GeneratorArg::Cli => run_cli_on_examples(out_dir, mdbook_opt),
    }
}

pub fn book_init(
    layout: &str,
    summary: bool,
    markdown_only: bool,
    generator: GeneratorArg,
) -> Result<()> {
    let out = api_book();
    if markdown_only && out.exists() {
        std::fs::remove_dir_all(&out).context("clear api-book before markdown-only init")?;
    }
    let init = !markdown_only;
    run_on_examples(&out, &mdbook_opt(layout, summary, init, false), generator)
}

pub fn book_refresh(layout: &str, summary: bool, generator: GeneratorArg) -> Result<()> {
    let out = api_book();
    if !out.join("book.toml").is_file() {
        bail!(
            "{} is missing book.toml; run `cargo xtask book-init` first",
            out.display()
        );
    }
    run_on_examples(&out, &mdbook_opt(layout, summary, false, true), generator)
}

pub fn book_links() -> Result<()> {
    let out_dir = api_book();
    let markdown_root = if out_dir.join("book.toml").is_file() {
        protobuf_mdbook::book_config::markdown_root_dir(&out_dir)
            .with_context(|| format!("load paths from {}", out_dir.join("book.toml").display()))?
    } else {
        out_dir.join("src/packages")
    };
    if !markdown_root.is_dir() {
        bail!(
            "{} is missing generated markdown under {}; run `cargo xtask book-init --markdown-only` or `book-init` first",
            out_dir.display(),
            markdown_root
                .strip_prefix(&out_dir)
                .unwrap_or(&markdown_root)
                .display()
        );
    }
    protobuf_mdbook::link_check::assert_tree(&out_dir).context("markdown link check")
}

pub fn book_build() -> Result<()> {
    let out_dir = api_book();
    if !out_dir.join("book.toml").is_file() {
        bail!(
            "{} is missing book.toml; run `cargo xtask book-init` first",
            out_dir.display()
        );
    }
    let status = Command::new("mdbook")
        .args(["build"])
        .current_dir(&out_dir)
        .status()
        .context("mdbook build")?;
    if status.success() {
        Ok(())
    } else {
        bail!("mdbook build failed ({status})");
    }
}
