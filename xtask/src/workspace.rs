//! Shared workspace paths and subprocess helpers.

use anyhow::{Context, Result, bail};
use std::process::Command;

pub const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

/// Log and run an xtask step.
pub fn run(name: &str, f: impl FnOnce() -> Result<()>) -> Result<()> {
    eprintln!("xtask: {name}");
    f()
}

/// Run `cargo` from the workspace root.
pub fn cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(WORKSPACE_ROOT)
        .status()
        .with_context(|| format!("cargo {}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        bail!("cargo {} failed ({status})", args.join(" "));
    }
}
