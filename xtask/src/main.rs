//! Crate-local CI and example generation for `protobuf-mdbook`.

mod book;
mod ci;
mod docker;
mod vendor;
mod workspace;

use anyhow::Result;
use book::GeneratorArg;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use workspace::run;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// buf-lint, fmt-check, clippy, test, build-plugin, book-init --markdown-only, book-links
    Ci,
    /// `cargo fmt` on this workspace plus `buf format -w` on `examples/proto/`.
    Fmt,
    /// `cargo fmt --check` plus `buf format --diff` on `examples/proto/`.
    FmtCheck,
    Clippy,
    Test,
    BuildPlugin,
    /// Scaffold or regenerate `./api-book` from `examples/proto/`.
    BookInit {
        #[arg(long, default_value = "package")]
        layout: String,
        #[arg(long)]
        summary: bool,
        #[arg(long)]
        markdown_only: bool,
        #[arg(long, value_enum, default_value_t = GeneratorArg::Protoc)]
        generator: GeneratorArg,
    },
    /// Refresh `./api-book` package markdown without `init`.
    BookRefresh {
        #[arg(long, default_value = "package")]
        layout: String,
        #[arg(long)]
        summary: bool,
        #[arg(long, value_enum, default_value_t = GeneratorArg::Protoc)]
        generator: GeneratorArg,
    },
    BookLinks,
    BookBuild,
    RumdlCheck,
    RumdlFmt,
    Docker,
    CheckHighlightjsVendor,
    BufLint,
    BufFormat,
    BufFormatCheck,
    CheckToolchain {
        #[arg(long)]
        strict: bool,
    },
    Coverage {
        #[arg(long)]
        open: bool,
        #[arg(long)]
        lcov: bool,
        #[arg(long, default_value = "lcov.info")]
        output_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Ci => {
            run("check-toolchain", || ci::check_toolchain(true))?;
            run("buf-lint", ci::buf_lint)?;
            run("fmt-check", ci::fmt_check)?;
            run("clippy", ci::clippy)?;
            run("test", ci::test)?;
            run("build-plugin", ci::build_plugin)?;
            run("check-highlightjs-vendor", vendor::check_highlightjs_vendor)?;
            run("book-init", || {
                book::book_init("package", false, true, GeneratorArg::Protoc)
            })?;
            run("book-links", book::book_links)?;
            Ok(())
        }
        Cmd::Fmt => ci::fmt(),
        Cmd::FmtCheck => ci::fmt_check(),
        Cmd::Clippy => ci::clippy(),
        Cmd::Test => ci::test(),
        Cmd::BuildPlugin => ci::build_plugin(),
        Cmd::BookInit {
            layout,
            summary,
            markdown_only,
            generator,
        } => book::book_init(&layout, summary, markdown_only, generator),
        Cmd::BookRefresh {
            layout,
            summary,
            generator,
        } => book::book_refresh(&layout, summary, generator),
        Cmd::BookLinks => book::book_links(),
        Cmd::BookBuild => book::book_build(),
        Cmd::RumdlCheck => ci::rumdl_check(),
        Cmd::RumdlFmt => ci::rumdl_fmt(),
        Cmd::Docker => docker::docker(),
        Cmd::CheckHighlightjsVendor => vendor::check_highlightjs_vendor(),
        Cmd::BufLint => ci::buf_lint(),
        Cmd::BufFormat => ci::buf_format(),
        Cmd::BufFormatCheck => ci::buf_format_check_cmd(),
        Cmd::CheckToolchain { strict } => ci::check_toolchain(strict),
        Cmd::Coverage {
            open,
            lcov,
            output_path,
        } => ci::coverage(open, lcov, &output_path),
    }
}
