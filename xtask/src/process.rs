//! Typed subprocess construction and execution.

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, ensure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: OsString,
    pub args: Vec<OsString>,
}

impl CommandSpec {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

pub fn run(root: &Path, spec: &CommandSpec) -> Result<()> {
    eprintln!("+ {}", display(spec));
    let status = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(root)
        .status()
        .with_context(|| format!("starting {}", display(spec)))?;
    ensure!(status.success(), "{} failed with {status}", display(spec));
    Ok(())
}

pub fn output(root: &Path, spec: &CommandSpec) -> Result<std::process::Output> {
    eprintln!("+ {}", display(spec));
    Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(root)
        .output()
        .with_context(|| format!("starting {}", display(spec)))
}

pub fn best_effort(root: &Path, spec: &CommandSpec) {
    eprintln!("+ {} (cleanup)", display(spec));
    let _ = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn probe(root: &Path, label: &str, spec: &CommandSpec, guidance: &str) -> Result<(), String> {
    match Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(root)
        .output()
    {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(format!(
            "{label} is not installed or not on PATH. {guidance}"
        )),
        Err(error) => Err(format!(
            "{label} could not be launched: {error}. {guidance}"
        )),
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(format!(
            "{label} is installed but unusable ({}): {}. {guidance}",
            output.status,
            concise_output(&output.stdout, &output.stderr)
        )),
    }
}

pub fn require(root: &Path, label: &str, spec: &CommandSpec, guidance: &str) -> Result<()> {
    probe(root, label, spec, guidance).map_err(anyhow::Error::msg)
}

pub fn display(spec: &CommandSpec) -> String {
    std::iter::once(spec.program.as_os_str())
        .chain(spec.args.iter().map(OsString::as_os_str))
        .map(OsStr::to_string_lossy)
        .collect::<Vec<_>>()
        .join(" ")
}

fn concise_output(stdout: &[u8], stderr: &[u8]) -> String {
    let text = if stderr.is_empty() { stdout } else { stderr };
    let text = String::from_utf8_lossy(text);
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.is_empty() {
        "no diagnostic output".to_string()
    } else {
        one_line.chars().take(400).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_probe_diagnostic_is_helpful() {
        assert_eq!(concise_output(b"", b""), "no diagnostic output");
    }

    #[test]
    fn probe_distinguishes_a_missing_program() {
        let error = probe(
            Path::new("."),
            "fixture",
            &CommandSpec::new("__protobuf_mdbook_xtask_missing_probe_fixture__"),
            "Install the fixture.",
        )
        .expect_err("fixture must be absent");
        assert!(error.contains("not installed or not on PATH"));
        assert!(error.contains("Install the fixture."));
    }

    #[test]
    fn probe_distinguishes_an_unusable_program() {
        let error = probe(
            Path::new("."),
            "fixture",
            &CommandSpec::new("cargo").arg("__protobuf_mdbook_xtask_bad_subcommand_fixture__"),
            "Repair the fixture.",
        )
        .expect_err("Cargo must reject the impossible subcommand");
        assert!(error.contains("installed but unusable"));
        assert!(error.contains("Repair the fixture."));
    }
}
