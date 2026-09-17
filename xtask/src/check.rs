//! Canonically ordered repository checks.

use std::path::Path;

use anyhow::{Result, ensure};

use crate::book::GeneratorArg;
use crate::process::{CommandSpec, require, run as run_command};
use crate::{CheckStep, FeatureArgs, book, ci, highlight, policy};

const RUSTFMT_INSTALL: &str = "Install it with `rustup component add rustfmt`.";
const CLIPPY_INSTALL: &str = "Install it with `rustup component add clippy`.";

pub fn select_steps(only: &[CheckStep], exclude: &[CheckStep]) -> Result<Vec<CheckStep>> {
    let selected: Vec<_> = policy::CHECK_ORDER
        .iter()
        .copied()
        .filter(|step| {
            if only.is_empty() {
                !exclude.contains(step)
            } else {
                only.contains(step)
            }
        })
        .collect();
    ensure!(
        !selected.is_empty(),
        "the check selection is empty; choose at least one registered step"
    );
    Ok(selected)
}

pub fn run(root: &Path, steps: &[CheckStep], features: &FeatureArgs) -> Result<()> {
    require(
        root,
        "Cargo",
        &CommandSpec::new("cargo").arg("--version"),
        "Install a Rust toolchain from https://rustup.rs/.",
    )?;
    if steps.contains(&CheckStep::Fmt) {
        require(
            root,
            "rustfmt",
            &CommandSpec::new("cargo").args(["fmt", "--version"]),
            RUSTFMT_INSTALL,
        )?;
    }
    if steps.contains(&CheckStep::Clippy) {
        require(
            root,
            "Clippy",
            &CommandSpec::new("cargo").args(["clippy", "--version"]),
            CLIPPY_INSTALL,
        )?;
    }
    if steps.iter().any(|step| step.needs_buf()) {
        ci::buf_command()?;
    }

    for step in steps {
        eprintln!("xtask: {step}");
        run_step(root, *step, features)?;
    }
    Ok(())
}

fn run_step(root: &Path, step: CheckStep, features: &FeatureArgs) -> Result<()> {
    match step {
        CheckStep::Toolchain => ci::check_toolchain(true),
        CheckStep::BufLint => ci::buf_lint_after_probe(),
        CheckStep::Fmt => {
            run_command(root, &fmt_check_command())?;
            ci::buf_format_check_after_probe()
        }
        CheckStep::Check => run_command(root, &cargo_quality("check", features, false)),
        CheckStep::Clippy => run_command(root, &cargo_quality("clippy", features, true)),
        CheckStep::Test => run_command(root, &cargo_quality("test", features, false)),
        CheckStep::BuildPlugin => ci::build_plugin(),
        CheckStep::HighlightRust => highlight::check_highlight_rust(),
        CheckStep::BookInit => book::book_init("package", false, true, GeneratorArg::Protoc),
        CheckStep::BookLinks => book::book_links(),
    }
}

fn fmt_check_command() -> CommandSpec {
    CommandSpec::new("cargo").args(["fmt", "--all", "--", "--check"])
}

fn cargo_quality(subcommand: &str, features: &FeatureArgs, deny_warnings: bool) -> CommandSpec {
    let mut command =
        CommandSpec::new("cargo").args([subcommand, "--locked", "--workspace", "--all-targets"]);
    command.args.extend(features.cargo_args());
    if deny_warnings {
        command
            .args
            .extend(["--".into(), "-D".into(), "warnings".into()]);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_deduplicated_and_canonical() {
        let selected = select_steps(&[CheckStep::Test, CheckStep::Fmt, CheckStep::Test], &[])
            .expect("selection should work");
        assert_eq!(selected, vec![CheckStep::Fmt, CheckStep::Test]);
    }

    #[test]
    fn exclusion_preserves_registry_order() {
        let selected =
            select_steps(&[], &[CheckStep::Clippy, CheckStep::BookInit]).expect("selection");
        assert_eq!(
            selected,
            vec![
                CheckStep::Toolchain,
                CheckStep::BufLint,
                CheckStep::Fmt,
                CheckStep::Check,
                CheckStep::Test,
                CheckStep::BuildPlugin,
                CheckStep::HighlightRust,
                CheckStep::BookLinks,
            ]
        );
    }

    #[test]
    fn excluding_every_step_is_an_error() {
        assert!(select_steps(&[], &policy::CHECK_ORDER).is_err());
    }

    #[test]
    fn fmt_does_not_receive_feature_arguments() {
        let command = fmt_check_command();
        assert!(!command.args.contains(&"--all-features".into()));
    }

    #[test]
    fn cargo_quality_forwards_explicit_features() {
        let features = FeatureArgs {
            all_features: false,
            features: vec!["alpha".into()],
            no_default_features: true,
        };
        let command = cargo_quality("test", &features, false);
        assert!(command.args.contains(&"--no-default-features".into()));
        assert!(command.args.contains(&"alpha".into()));
        assert!(!command.args.contains(&"--all-features".into()));
    }
}
