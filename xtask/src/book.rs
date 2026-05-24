//! Guided `./api-book` generation and link checks.

use crate::ci::{buf_command, build_plugin, release_bin};
use crate::workspace::WORKSPACE_ROOT;
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use protobuf_mdbook::examples::{MdbookOptFlags, format_mdbook_opt};
use protobuf_mdbook::input::Compiler;
use protobuf_mdbook::runner::{Driver, RunSpec, run_generation};
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

fn example_proto_paths() -> Vec<PathBuf> {
    protobuf_mdbook::examples::EXAMPLE_PROTO_INPUTS
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

fn driver_for(generator: GeneratorArg) -> Result<Driver> {
    build_plugin()?;
    match generator {
        GeneratorArg::Protoc => Ok(Driver::protoc_plugin(
            protoc_bin()?,
            release_bin("protoc-gen-mdbook")?,
        )),
        GeneratorArg::Cli => {
            buf_command()?;
            Ok(Driver::cli(release_bin("protobuf-mdbook")?, Compiler::Buf))
        }
    }
}

fn run_on_examples(out_dir: &Path, mdbook_opt: &str, generator: GeneratorArg) -> Result<()> {
    let inputs = prepare_example_output(out_dir, mdbook_opt)?;
    eprintln!(
        "xtask: {} {} proto file(s) → {} (opt={mdbook_opt})",
        match generator {
            GeneratorArg::Protoc => "protoc",
            GeneratorArg::Cli => "protobuf-mdbook",
        },
        inputs.len(),
        out_dir.display()
    );

    let proto_root = protobuf_mdbook::examples::examples_proto_dir();
    let deps = ensure_proto_deps_export()?;
    let search_paths = vec![PathBuf::from("."), deps];
    let spec = RunSpec {
        out: out_dir,
        mdbook_opt,
        inputs: &inputs,
        search_paths: &search_paths,
        cwd: Some(&proto_root),
    };
    run_generation(&spec, &driver_for(generator)?)
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
    let opt = format_mdbook_opt(
        layout,
        "",
        MdbookOptFlags {
            summary,
            init,
            ..MdbookOptFlags::default()
        },
    );
    run_on_examples(&out, &opt, generator)
}

pub fn book_refresh(layout: &str, summary: bool, generator: GeneratorArg) -> Result<()> {
    let out = api_book();
    if !out.join("book.toml").is_file() {
        bail!(
            "{} is missing book.toml; run `cargo xtask book-init` first",
            out.display()
        );
    }
    let book_s = out.to_string_lossy().into_owned();
    let opt = format_mdbook_opt(
        layout,
        "",
        MdbookOptFlags {
            summary,
            with_book_paths: Some((book_s.clone(), book_s)),
            ..MdbookOptFlags::default()
        },
    );
    run_on_examples(&out, &opt, generator)
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
