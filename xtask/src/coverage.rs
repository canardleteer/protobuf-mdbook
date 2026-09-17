//! Coverage generation with engine-specific reports.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

use crate::process::{CommandSpec, require, run as run_command};
use crate::{CoverageEngine, FeatureArgs, policy};

const LLVM_INSTALL: &str = "Install it with `cargo install --locked cargo-llvm-cov`.";
const TARPAULIN_INSTALL: &str = "Install it with `cargo install --locked cargo-tarpaulin`.";

pub fn run(root: &Path, engine: CoverageEngine, features: &FeatureArgs, open: bool) -> Result<()> {
    let (probe, guidance) = match engine {
        CoverageEngine::LlvmCov => (
            CommandSpec::new("cargo").args(["llvm-cov", "--version"]),
            LLVM_INSTALL,
        ),
        CoverageEngine::Tarpaulin => (
            CommandSpec::new("cargo").args(["tarpaulin", "--version"]),
            TARPAULIN_INSTALL,
        ),
    };
    require(root, coverage_label(engine), &probe, guidance)?;
    for command in commands(engine, features) {
        run_command(root, &command)?;
    }
    let report = report_path(root, engine)?;
    if open {
        open_report(root, &report)?;
    }
    Ok(())
}

fn commands(engine: CoverageEngine, features: &FeatureArgs) -> Vec<CommandSpec> {
    match engine {
        CoverageEngine::LlvmCov => {
            let clean = CommandSpec::new("cargo").args(["llvm-cov", "clean", "--workspace"]);
            let mut generate = CommandSpec::new("cargo").args([
                "llvm-cov",
                "--locked",
                "--workspace",
                "--all-targets",
            ]);
            generate.args.extend(features.cargo_args());
            generate.args.extend([
                "--html".into(),
                "--output-dir".into(),
                OsString::from("target/coverage/llvm-cov"),
            ]);
            vec![clean, generate]
        }
        CoverageEngine::Tarpaulin => {
            let mut generate = CommandSpec::new("cargo").args([
                "tarpaulin",
                "--locked",
                "--workspace",
                "--all-targets",
            ]);
            generate.args.extend(features.cargo_args());
            generate.args.extend([
                "--out".into(),
                "Html".into(),
                "--output-dir".into(),
                OsString::from("target/coverage/tarpaulin"),
            ]);
            vec![generate]
        }
    }
}

fn coverage_label(engine: CoverageEngine) -> &'static str {
    match engine {
        CoverageEngine::LlvmCov => "cargo-llvm-cov",
        CoverageEngine::Tarpaulin => "cargo-tarpaulin",
    }
}

fn report_path(root: &Path, engine: CoverageEngine) -> Result<PathBuf> {
    let relative = match engine {
        CoverageEngine::LlvmCov => policy::LLVM_COV_REPORT,
        CoverageEngine::Tarpaulin => policy::TARPAULIN_REPORT,
    };
    let report = root.join(relative);
    ensure!(
        report.is_file(),
        "coverage command succeeded but no report exists at {}",
        report.display()
    );
    report
        .canonicalize()
        .with_context(|| format!("resolving coverage report {}", report.display()))
}

fn open_report(root: &Path, report: &Path) -> Result<()> {
    let command = if cfg!(target_os = "macos") {
        CommandSpec::new("open").arg(report)
    } else if cfg!(target_os = "windows") {
        CommandSpec::new("cmd").args([
            OsString::from("/C"),
            OsString::from("start"),
            OsString::new(),
            report.as_os_str().to_owned(),
        ])
    } else {
        CommandSpec::new("xdg-open").arg(report)
    };
    run_command(root, &command).or_else(|error| {
        eprintln!(
            "xtask: could not open {}; open this file manually ({error})",
            report.display()
        );
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_engines_have_distinct_report_paths() {
        assert_ne!(policy::LLVM_COV_REPORT, policy::TARPAULIN_REPORT);
    }

    #[test]
    fn explicit_features_reach_both_engines() {
        let features = FeatureArgs {
            all_features: false,
            features: vec!["alpha".into()],
            no_default_features: true,
        };
        for engine in [CoverageEngine::LlvmCov, CoverageEngine::Tarpaulin] {
            let generated = commands(engine, &features);
            let args = &generated.last().expect("generation command").args;
            assert!(args.contains(&"--no-default-features".into()));
            assert!(args.contains(&"alpha".into()));
        }
    }
}
