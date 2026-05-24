//! Docker build and runtime smoke checks.

use crate::workspace::WORKSPACE_ROOT;
use anyhow::{Context, Result, bail};
use std::process::Command;

const DOCKER_IMAGE: &str = "protobuf-mdbook:local";

pub fn docker() -> Result<()> {
    docker_build()?;
    docker_smoke_version()?;
    docker_smoke_config()?;
    Ok(())
}

fn docker_build() -> Result<()> {
    eprintln!("xtask: docker build --platform linux/amd64 -t {DOCKER_IMAGE}");
    let status = Command::new("docker")
        .args([
            "build",
            "--platform",
            "linux/amd64",
            "-f",
            &format!("{WORKSPACE_ROOT}/Dockerfile"),
            "-t",
            DOCKER_IMAGE,
            WORKSPACE_ROOT,
        ])
        .status()
        .context("docker build")?;
    if status.success() {
        Ok(())
    } else {
        bail!("docker build failed ({status})");
    }
}

/// Runtime image must answer `--version`.
fn docker_smoke_version() -> Result<()> {
    eprintln!("xtask: docker run --version smoke");
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--entrypoint",
            "/protoc-gen-mdbook",
            DOCKER_IMAGE,
            "--version",
        ])
        .output()
        .context("docker run --version")?;
    if !output.status.success() {
        bail!(
            "docker --version failed ({:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("protoc-gen-mdbook") {
        bail!("docker --version stdout missing plugin name: {stdout}");
    }
    let pin = protobuf_mdbook::mdbook_version();
    if !stdout.contains(pin) {
        bail!("docker --version stdout missing mdbook pin {pin}: {stdout}");
    }
    eprintln!("xtask: docker --version ok: {}", stdout.trim());
    Ok(())
}

/// Scratch runtime: non-root user, static entrypoint only.
fn docker_smoke_config() -> Result<()> {
    eprintln!("xtask: docker inspect config smoke");
    let user = docker_inspect_format("{{.Config.User}}")?;
    if user != "nobody" {
        bail!("expected Config.User=nobody, got {user:?}");
    }
    let entrypoint = docker_inspect_format("{{json .Config.Entrypoint}}")?;
    if !entrypoint.contains("protoc-gen-mdbook") {
        bail!("unexpected Entrypoint: {entrypoint}");
    }
    let os = docker_inspect_format("{{.Os}}")?;
    let arch = docker_inspect_format("{{.Architecture}}")?;
    if os != "linux" || arch != "amd64" {
        bail!("expected linux/amd64 image, got {os}/{arch}");
    }
    Ok(())
}

fn docker_inspect_format(format: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["image", "inspect", "--format", format, DOCKER_IMAGE])
        .output()
        .with_context(|| format!("docker image inspect --format {format}"))?;
    if !output.status.success() {
        bail!(
            "docker image inspect failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
