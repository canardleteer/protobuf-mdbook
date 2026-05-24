//! CI helpers: fmt, clippy, test, toolchain, buf, coverage.

use crate::workspace::{WORKSPACE_ROOT, cargo};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub fn fmt() -> Result<()> {
    cargo_fmt(&[])?;
    buf_format_write()
}

pub fn fmt_check() -> Result<()> {
    cargo_fmt(&["--check"])?;
    buf_format_check()
}

fn cargo_fmt(extra: &[&str]) -> Result<()> {
    let mut args = vec![
        "fmt",
        "-p",
        "protobuf-mdbook",
        "-p",
        "protobuf-mdbook-xtask",
        "--",
    ];
    args.extend_from_slice(extra);
    cargo(&args)
}

pub fn clippy() -> Result<()> {
    cargo(&[
        "clippy",
        "--locked",
        "-p",
        "protobuf-mdbook",
        "-p",
        "protobuf-mdbook-xtask",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

pub fn test() -> Result<()> {
    cargo(&["test", "--locked", "-p", "protobuf-mdbook"])
}

pub fn update_golden() -> Result<()> {
    let status = Command::new("cargo")
        .current_dir(WORKSPACE_ROOT)
        .env("UPDATE_GOLDEN", "1")
        .args([
            "test",
            "--locked",
            "-p",
            "protobuf-mdbook",
            "output_regression",
            "--",
            "--nocapture",
        ])
        .status()
        .context("spawn cargo test output_regression")?;
    if status.success() {
        Ok(())
    } else {
        bail!("update-golden failed");
    }
}

pub fn build_plugin() -> Result<()> {
    cargo(&["build", "--locked", "--release", "-p", "protobuf-mdbook"])
}

pub fn release_bin(name: &str) -> Result<std::path::PathBuf> {
    let mut path = Path::new(WORKSPACE_ROOT).join(format!("target/release/{name}"));
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path.canonicalize()
        .with_context(|| format!("locate binary at {}", path.display()))
}

pub fn buf_command() -> Result<()> {
    let status = Command::new("buf")
        .arg("--version")
        .status()
        .context("spawn buf (--version)")?;
    if status.success() {
        Ok(())
    } else {
        bail!(
            "buf CLI not found or failed; install with \
             `cargo install buf-toolchain --locked --version 1.69.0` \
             (see Dockerfile buf-anchor)"
        );
    }
}

fn examples_proto() -> std::path::PathBuf {
    Path::new(WORKSPACE_ROOT).join("examples/proto")
}

pub fn buf_lint() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .arg("lint")
        .status()
        .context("buf lint")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf lint failed ({status})");
    }
}

fn buf_format_write() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .args(["format", "-w"])
        .status()
        .context("buf format -w")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf format -w failed ({status})");
    }
}

fn buf_format_check() -> Result<()> {
    buf_command()?;
    let proto_root = examples_proto();
    let status = Command::new("buf")
        .current_dir(&proto_root)
        .args(["format", "--diff"])
        .status()
        .context("buf format --diff")?;
    if status.success() {
        Ok(())
    } else {
        bail!("buf format --diff failed ({status}); run `cargo xtask fmt`");
    }
}

pub fn buf_format() -> Result<()> {
    buf_format_write()
}

pub fn buf_format_check_cmd() -> Result<()> {
    buf_format_check()
}

pub fn rumdl_check() -> Result<()> {
    let readme = Path::new(WORKSPACE_ROOT).join("README.md");
    let status = Command::new("rumdl")
        .args(["check", readme.to_str().expect("utf8")])
        .status()
        .context("rumdl check")?;
    if status.success() {
        Ok(())
    } else {
        bail!("rumdl check failed");
    }
}

pub fn rumdl_fmt() -> Result<()> {
    let readme = Path::new(WORKSPACE_ROOT).join("README.md");
    let status = Command::new("rumdl")
        .args(["fmt", readme.to_str().expect("utf8")])
        .status()
        .context("rumdl fmt")?;
    if status.success() {
        Ok(())
    } else {
        bail!("rumdl fmt failed");
    }
}

#[derive(Debug)]
struct ToolchainPin {
    channel: String,
    components: Vec<String>,
}

fn read_toolchain_pin() -> Result<ToolchainPin> {
    let path = Path::new(WORKSPACE_ROOT).join("rust-toolchain.toml");
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut channel = None;
    let mut components = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("channel = ") {
            channel = Some(trim_toml_string(v));
        } else if let Some(v) = line.strip_prefix("components = ") {
            components = parse_toml_string_array(v);
        }
    }
    let channel = channel.context("rust-toolchain.toml missing channel")?;
    Ok(ToolchainPin {
        channel,
        components,
    })
}

fn trim_toml_string(raw: &str) -> String {
    raw.trim().trim_matches('"').trim_matches('\'').to_string()
}

fn parse_toml_string_array(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| trim_toml_string(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn command_output(bin: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("{bin} {}", args.join(" ")))?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        bail!(
            "{bin} {} failed ({:?}): {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

pub fn check_toolchain(strict: bool) -> Result<()> {
    let pin = read_toolchain_pin()?;
    let rustc_v = command_output("rustc", &["-V"])?;
    let active = rustc_v
        .split_whitespace()
        .nth(1)
        .context("parse rustc -V")?;

    let mut divergent = false;
    if !rustc_v.contains(&pin.channel) {
        divergent = true;
        eprintln!(
            "xtask: warning: active rustc ({active}) does not match rust-toolchain.toml channel ({})",
            pin.channel
        );
    }

    let installed = command_output("rustup", &["component", "list", "--installed"])?;
    for component in &pin.components {
        if !installed.lines().any(|line| line.starts_with(component)) {
            divergent = true;
            eprintln!(
                "xtask: warning: rustup component {component:?} from rust-toolchain.toml is not installed"
            );
        }
    }

    if divergent {
        if strict {
            bail!(
                "toolchain diverges from rust-toolchain.toml (channel={}, components={:?}); \
                 run `rustup toolchain install` in the repo root",
                pin.channel,
                pin.components
            );
        }
        eprintln!("xtask: check-toolchain: divergent (non-strict mode; use --strict to fail)");
    } else {
        eprintln!(
            "xtask: check-toolchain ok (channel={}, components={:?})",
            pin.channel, pin.components
        );
    }
    Ok(())
}

fn ensure_cargo_llvm_cov() -> Result<()> {
    let status = Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("cargo llvm-cov --version")?;
    if status.success() {
        return Ok(());
    }
    bail!(
        "cargo llvm-cov not found; install with: cargo install cargo-llvm-cov --locked\n\
         also run: rustup component add llvm-tools-preview"
    );
}

pub fn coverage(open: bool, lcov: bool, output_path: &Path) -> Result<()> {
    ensure_cargo_llvm_cov()?;
    let mut args = vec![
        "llvm-cov",
        "--locked",
        "-p",
        "protobuf-mdbook",
        "--all-targets",
    ];
    if open {
        args.push("--open");
    } else if lcov {
        args.extend(["--lcov", "--output-path"]);
        args.push(output_path.to_str().context("lcov output path utf8")?);
    } else {
        args.push("--html");
    }
    let result = cargo(&args);
    if result.is_err() {
        eprintln!(
            "hint: if llvm-cov failed to find tools, run: rustup component add llvm-tools-preview"
        );
    }
    result
}
