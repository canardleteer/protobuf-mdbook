//! Shared protoc-plugin and CLI generation invocations for tests and xtask.

use crate::input::Compiler;
use crate::options::parameter_to_cli_args;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Which binary drives generation.
#[derive(Clone, Debug)]
pub enum Driver {
    ProtocPlugin { protoc: PathBuf, plugin: PathBuf },
    Cli { cli: PathBuf, compiler: Compiler },
}

/// Inputs for a single generation run.
pub struct RunSpec<'a> {
    pub out: &'a Path,
    pub mdbook_opt: &'a str,
    pub inputs: &'a [PathBuf],
    pub search_paths: &'a [PathBuf],
    pub cwd: Option<&'a Path>,
}

impl Driver {
    pub fn protoc_plugin(protoc: PathBuf, plugin: PathBuf) -> Self {
        Self::ProtocPlugin { protoc, plugin }
    }

    pub fn cli(cli: PathBuf, compiler: Compiler) -> Self {
        Self::Cli { cli, compiler }
    }
}

/// Run protoc+plugin or `protobuf-mdbook` with the given spec.
pub fn run_generation(spec: &RunSpec<'_>, driver: &Driver) -> Result<()> {
    match driver {
        Driver::ProtocPlugin { protoc, plugin } => run_protoc(spec, protoc, plugin),
        Driver::Cli { cli, compiler } => run_cli(spec, cli, *compiler),
    }
}

fn run_protoc(spec: &RunSpec<'_>, protoc: &Path, plugin: &Path) -> Result<()> {
    std::fs::create_dir_all(spec.out).context("create output dir")?;

    let mut cmd = Command::new(protoc);
    if let Some(cwd) = spec.cwd {
        cmd.current_dir(cwd);
    }
    for inc in spec.search_paths {
        cmd.arg("-I").arg(inc);
    }
    cmd.arg(format!("--plugin=protoc-gen-mdbook={}", plugin.display()))
        .arg(format!("--mdbook_out={}", spec.out.display()));
    if !spec.mdbook_opt.is_empty() {
        cmd.arg(format!("--mdbook_opt={}", spec.mdbook_opt));
    }
    for input in spec.inputs {
        cmd.arg(input);
    }

    let status = cmd
        .status()
        .with_context(|| format!("spawn {}", protoc.display()))?;
    if status.success() {
        Ok(())
    } else {
        bail!("protoc failed (opt={})", spec.mdbook_opt);
    }
}

fn run_cli(spec: &RunSpec<'_>, cli: &Path, compiler: Compiler) -> Result<()> {
    std::fs::create_dir_all(spec.out).context("create output dir")?;

    let mut cmd = Command::new(cli);
    if let Some(cwd) = spec.cwd {
        cmd.current_dir(cwd);
    }
    cmd.arg("-o").arg(spec.out);
    if !spec.mdbook_opt.is_empty() {
        let cli_args = parameter_to_cli_args(spec.mdbook_opt)?;
        for arg in &cli_args {
            cmd.arg(arg);
        }
    }
    if compiler == Compiler::Protoc {
        cmd.args(["--compiler", "protoc"]);
    }
    for inc in spec.search_paths {
        cmd.arg("-I").arg(inc);
    }
    for input in spec.inputs {
        cmd.arg(input);
    }

    let status = cmd
        .status()
        .with_context(|| format!("spawn {}", cli.display()))?;
    if status.success() {
        Ok(())
    } else {
        bail!("protobuf-mdbook failed (opt={})", spec.mdbook_opt);
    }
}
