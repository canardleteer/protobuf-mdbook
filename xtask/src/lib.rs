//! Repository development tasks for `protobuf-mdbook`.

mod book;
mod check;
mod ci;
mod coverage;
mod highlight;
mod image;
mod policy;
mod process;
mod workspace;

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use book::GeneratorArg;
use clap::{Args, Parser, Subcommand, ValueEnum};

/// The repository development command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "cargo xtask",
    bin_name = "cargo xtask",
    about = "Run repository development tasks"
)]
pub struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Debug, Subcommand)]
enum Task {
    /// Run all or a selected subset of repository checks.
    #[command(visible_alias = "ci")]
    Check(CheckArgs),
    /// Generate a fresh coverage report and optionally open it.
    Coverage(CoverageArgs),
    /// Generate a fresh coverage report and open it.
    CoverageOpen(CoverageSelection),
    /// Build and smoke-test the scratch plugin image without pushing it.
    #[command(visible_alias = "docker")]
    Image(ImageArgs),
    /// `cargo fmt` plus `buf format -w` on `examples/proto/`.
    Fmt,
    /// `cargo fmt --check` plus `buf format --diff` on `examples/proto/`.
    FmtCheck,
    Clippy,
    Test,
    /// Refresh checked-in golden fixtures under `tests/fixtures/golden/`.
    UpdateGolden,
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
    CheckHighlightRust,
    UpdateHighlightGolden,
    BufLint,
    BufFormat,
    BufFormatCheck,
    CheckToolchain {
        #[arg(long)]
        strict: bool,
    },
}

/// Selectable check steps. Absence of a selector means all steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum CheckStep {
    Toolchain,
    BufLint,
    Fmt,
    Check,
    Clippy,
    Test,
    BuildPlugin,
    HighlightRust,
    BookInit,
    BookLinks,
}

impl CheckStep {
    fn needs_buf(self) -> bool {
        matches!(self, Self::BufLint | Self::Fmt | Self::BookInit)
    }
}

impl fmt::Display for CheckStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Toolchain => "toolchain",
            Self::BufLint => "buf-lint",
            Self::Fmt => "fmt",
            Self::Check => "check",
            Self::Clippy => "clippy",
            Self::Test => "test",
            Self::BuildPlugin => "build-plugin",
            Self::HighlightRust => "highlight-rust",
            Self::BookInit => "book-init",
            Self::BookLinks => "book-links",
        })
    }
}

#[derive(Debug, Args)]
struct CheckArgs {
    /// Run only these comma-separated checks.
    #[arg(long, value_delimiter = ',', num_args = 1.., conflicts_with = "exclude")]
    only: Vec<CheckStep>,
    /// Run all checks except these comma-separated checks.
    #[arg(long, value_delimiter = ',', num_args = 1.., conflicts_with = "only")]
    exclude: Vec<CheckStep>,
    #[command(flatten)]
    features: FeatureArgs,
}

/// Cargo feature-selection arguments shared by checks and coverage.
#[derive(Clone, Debug, Default, Args)]
pub struct FeatureArgs {
    /// Build with all features (also the default when no feature option is set).
    #[arg(
        long,
        conflicts_with = "features",
        conflicts_with = "no_default_features"
    )]
    all_features: bool,
    /// Comma-separated Cargo features to enable.
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        value_parser = parse_nonempty,
        conflicts_with = "all_features"
    )]
    features: Vec<String>,
    /// Disable default features; may be combined with --features.
    #[arg(long, conflicts_with = "all_features")]
    no_default_features: bool,
}

impl FeatureArgs {
    fn cargo_args(&self) -> Vec<OsString> {
        if !self.all_features && self.features.is_empty() && !self.no_default_features {
            return vec!["--all-features".into()];
        }
        let mut args = Vec::new();
        if self.all_features {
            args.push("--all-features".into());
        }
        if self.no_default_features {
            args.push("--no-default-features".into());
        }
        if !self.features.is_empty() {
            args.push("--features".into());
            args.push(self.features.join(",").into());
        }
        args
    }
}

fn parse_nonempty(value: &str) -> std::result::Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err("feature names cannot be empty".to_string())
    } else {
        Ok(value.to_string())
    }
}

/// Supported coverage engines.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum CoverageEngine {
    #[default]
    LlvmCov,
    Tarpaulin,
}

#[derive(Clone, Debug, Args)]
struct CoverageSelection {
    /// Coverage engine.
    #[arg(long, value_enum, default_value_t)]
    engine: CoverageEngine,
    #[command(flatten)]
    features: FeatureArgs,
}

#[derive(Debug, Args)]
struct CoverageArgs {
    #[command(flatten)]
    selection: CoverageSelection,
    /// Open the newly generated report.
    #[arg(long)]
    open: bool,
}

/// Supported local OCI image engines.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ImageEngine {
    #[default]
    Auto,
    Docker,
    Buildah,
}

#[derive(Debug, Args)]
struct ImageArgs {
    /// OCI image engine.
    #[arg(long, value_enum, default_value_t)]
    engine: ImageEngine,
    /// Local image tag.
    #[arg(long, default_value = policy::IMAGE_TAG)]
    tag: String,
}

/// Dispatch a parsed repository task.
pub fn run(cli: Cli) -> Result<()> {
    let root = workspace_root();
    match cli.task {
        Task::Check(args) => {
            let steps = check::select_steps(&args.only, &args.exclude)?;
            check::run(&root, &steps, &args.features)
        }
        Task::Coverage(args) => coverage::run(
            &root,
            args.selection.engine,
            &args.selection.features,
            args.open,
        ),
        Task::CoverageOpen(args) => coverage::run(&root, args.engine, &args.features, true),
        Task::Image(args) => image::run(&root, args.engine, args.tag),
        Task::Fmt => ci::fmt(),
        Task::FmtCheck => check::run(&root, &[CheckStep::Fmt], &FeatureArgs::default()),
        Task::Clippy => check::run(&root, &[CheckStep::Clippy], &FeatureArgs::default()),
        Task::Test => check::run(&root, &[CheckStep::Test], &FeatureArgs::default()),
        Task::UpdateGolden => ci::update_golden(),
        Task::BuildPlugin => ci::build_plugin(),
        Task::BookInit {
            layout,
            summary,
            markdown_only,
            generator,
        } => book::book_init(&layout, summary, markdown_only, generator),
        Task::BookRefresh {
            layout,
            summary,
            generator,
        } => book::book_refresh(&layout, summary, generator),
        Task::BookLinks => book::book_links(),
        Task::BookBuild => book::book_build(),
        Task::RumdlCheck => ci::rumdl_check(),
        Task::RumdlFmt => ci::rumdl_fmt(),
        Task::CheckHighlightRust => highlight::check_highlight_rust(),
        Task::UpdateHighlightGolden => highlight::update_highlight_golden(),
        Task::BufLint => ci::buf_lint(),
        Task::BufFormat => ci::buf_format(),
        Task::BufFormatCheck => ci::buf_format_check(),
        Task::CheckToolchain { strict } => ci::check_toolchain(strict),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must remain directly below the workspace root")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_builds() {
        Cli::command().debug_assert();
    }

    #[test]
    fn ci_is_the_same_parsed_variant_as_check() {
        let cli = Cli::try_parse_from(["cargo xtask", "ci", "--only", "test,fmt"])
            .expect("ci alias should parse");
        let Task::Check(args) = cli.task else {
            panic!("ci must parse as check");
        };
        assert_eq!(args.only, vec![CheckStep::Test, CheckStep::Fmt]);
    }

    #[test]
    fn docker_is_the_same_parsed_variant_as_image() {
        let cli = Cli::try_parse_from(["cargo xtask", "docker", "--engine", "docker"])
            .expect("docker alias should parse");
        let Task::Image(args) = cli.task else {
            panic!("docker must parse as image");
        };
        assert_eq!(args.engine, ImageEngine::Docker);
        assert_eq!(args.tag, policy::IMAGE_TAG);
    }

    #[test]
    fn selectors_conflict_at_the_parser_boundary() {
        assert!(
            Cli::try_parse_from(["cargo xtask", "check", "--only", "fmt", "--exclude", "test"])
                .is_err()
        );
    }

    #[test]
    fn feature_defaults_and_explicit_combination_match_cargo() {
        assert_eq!(
            FeatureArgs::default().cargo_args(),
            vec![OsString::from("--all-features")]
        );
        let explicit = FeatureArgs {
            all_features: false,
            features: vec!["alpha".into(), "beta".into()],
            no_default_features: true,
        };
        assert_eq!(
            explicit.cargo_args(),
            vec![
                OsString::from("--no-default-features"),
                OsString::from("--features"),
                OsString::from("alpha,beta")
            ]
        );
    }

    #[test]
    fn open_aliases_accept_the_same_coverage_selection() {
        let cli = Cli::try_parse_from([
            "cargo xtask",
            "coverage-open",
            "--engine",
            "tarpaulin",
            "--features",
            "alpha",
        ])
        .expect("coverage-open should parse");
        let Task::CoverageOpen(args) = cli.task else {
            panic!("expected coverage-open");
        };
        assert_eq!(args.engine, CoverageEngine::Tarpaulin);
        assert_eq!(args.features.features, vec!["alpha"]);
    }
}
