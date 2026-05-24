//! Highlight.js vendor hash checks.

use crate::workspace::WORKSPACE_ROOT;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(sha256_hex_bytes(&bytes))
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Compare vendored Highlight.js grammars against `assets/highlightjs/*.meta.json`.
pub fn check_highlightjs_vendor() -> Result<()> {
    let highlight_dir = Path::new(WORKSPACE_ROOT).join("assets/highlightjs");
    let mut meta_paths: Vec<PathBuf> = std::fs::read_dir(&highlight_dir)
        .with_context(|| format!("read_dir {}", highlight_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|x| x.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".meta.json"))
        })
        .collect();
    meta_paths.sort();

    if meta_paths.is_empty() {
        bail!("no *.meta.json under {}", highlight_dir.display());
    }

    let compiled = protobuf_mdbook::mdbook_version();
    let mut mdbook_pin_checked = false;

    for meta_path in &meta_paths {
        let meta: serde_json::Value =
            serde_json::from_slice(&std::fs::read(meta_path).context("read meta.json")?)
                .context("parse meta.json")?;
        let grammar = meta["grammar"]
            .as_str()
            .or_else(|| {
                meta_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.strip_suffix(".meta").unwrap_or(s))
            })
            .context("grammar name in meta.json")?;

        if !mdbook_pin_checked {
            let mdbook_pin = meta["mdbook_pin"]
                .as_str()
                .context("mdbook_pin in meta.json")?;
            if !compiled.contains(mdbook_pin) {
                bail!(
                    "mdbook pin mismatch: meta.json mdbook_pin={mdbook_pin:?}, \
                     protobuf_mdbook::mdbook_version()={compiled:?}"
                );
            }
            mdbook_pin_checked = true;
        }

        let vendored_file = meta["vendored_file"]
            .as_str()
            .context("vendored_file in meta.json")?;
        let vendored_path = highlight_dir.join(vendored_file);
        let vendored_sha = sha256_hex(&vendored_path)?;
        let expected_vendored = meta["vendored_sha256"]
            .as_str()
            .context("vendored_sha256 in meta.json")?;
        if vendored_sha != expected_vendored {
            bail!(
                "vendored_sha256 mismatch for {grammar}: file={vendored_sha}, \
                 meta.json={expected_vendored}; update {} after editing {vendored_file}",
                meta_path.file_name().unwrap().to_string_lossy()
            );
        }

        let upstream_url = meta
            .get("upstream_file_url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        let expected_upstream = meta
            .get("upstream_sha256")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        match (upstream_url, expected_upstream) {
            (None, None) => {
                eprintln!("xtask: {grammar} ok (local grammar, vendored hash)");
            }
            (Some(url), Some(expected)) => {
                eprintln!("xtask: fetching upstream {url} ({grammar})");
                let output = Command::new("curl")
                    .args(["-fsSL", url])
                    .output()
                    .with_context(|| format!("curl upstream {grammar}"))?;
                if !output.status.success() {
                    bail!(
                        "curl failed for {grammar} ({:?}): {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                let upstream_sha = sha256_hex_bytes(&output.stdout);
                if upstream_sha != expected {
                    bail!(
                        "upstream_sha256 drift for {grammar}: fetched={upstream_sha}, \
                         meta.json={expected}; review and re-vendor if needed"
                    );
                }
                eprintln!("xtask: {grammar} ok (vendored + upstream hashes)");
            }
            _ => bail!(
                "invalid upstream fields in {}: upstream_file_url and upstream_sha256 \
                 must both be set or both absent/null",
                meta_path.display()
            ),
        }
    }

    eprintln!(
        "xtask: check-highlightjs-vendor ok ({} grammar(s), mdbook pin)",
        meta_paths.len()
    );
    Ok(())
}
