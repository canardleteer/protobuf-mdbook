//! Golden HTML parity checks for the Rust protobuf / CEL highlighter.

use crate::workspace::WORKSPACE_ROOT;
use anyhow::{Context, Result, bail};
use protobuf_mdbook::highlight::{highlight_source, normalize_newlines};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_DIR: &str = "tests/fixtures/highlight";

fn fixture_root() -> PathBuf {
    Path::new(WORKSPACE_ROOT).join(FIXTURE_DIR)
}

fn language_for_input(path: &Path) -> Result<&'static str> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("fixture filename")?;
    if name.ends_with(".proto.in") {
        Ok("protobuf")
    } else if name.ends_with(".cel.in") {
        Ok("cel")
    } else {
        bail!("unknown fixture extension: {}", path.display())
    }
}

fn golden_path(input: &Path) -> PathBuf {
    input.with_extension("html")
}

pub fn check_highlight_rust() -> Result<()> {
    let root = fixture_root();
    if !root.is_dir() {
        bail!("missing fixture dir {}", root.display());
    }
    let mut inputs: Vec<PathBuf> = fs::read_dir(&root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|x| x.to_str()) == Some("in")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".proto.in") || n.ends_with(".cel.in"))
        })
        .collect();
    inputs.sort();
    if inputs.is_empty() {
        bail!("no *.proto.in / *.cel.in fixtures under {}", root.display());
    }

    let mut failures = Vec::new();
    for input in &inputs {
        let lang = language_for_input(input)?;
        let source =
            fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
        let got = normalize_html(&highlight_source(lang, &source));
        let golden = golden_path(input);
        if !golden.is_file() {
            failures.push(format!("missing golden {}", golden.display()));
            continue;
        }
        let expected = normalize_html(
            &fs::read_to_string(&golden)
                .with_context(|| format!("read golden {}", golden.display()))?,
        );
        if got != expected {
            failures.push(format!(
                "highlight mismatch for {}\n--- expected ({})\n+++ got\n{}",
                input.file_name().unwrap().to_string_lossy(),
                golden.display(),
                diff_lines(&expected, &got)
            ));
        }
    }

    if failures.is_empty() {
        eprintln!(
            "xtask: check-highlight-rust ok ({} fixture(s))",
            inputs.len()
        );
        Ok(())
    } else {
        bail!(
            "highlight golden drift ({} fixture(s)); run `cargo xtask update-highlight-golden`\n\n{}",
            failures.len(),
            failures.join("\n\n")
        )
    }
}

pub fn update_highlight_golden() -> Result<()> {
    let root = fixture_root();
    let mut inputs: Vec<PathBuf> = fs::read_dir(&root)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|x| x.to_str()) == Some("in")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".proto.in") || n.ends_with(".cel.in"))
        })
        .collect();
    inputs.sort();
    for input in &inputs {
        let lang = language_for_input(input)?;
        let source = fs::read_to_string(input)?;
        let html = normalize_html(&highlight_source(lang, &source));
        let golden = golden_path(input);
        fs::write(&golden, format!("{html}\n"))
            .with_context(|| format!("write golden {}", golden.display()))?;
        eprintln!("xtask: updated {}", golden.display());
    }
    eprintln!(
        "xtask: update-highlight-golden ok ({} fixture(s))",
        inputs.len()
    );
    Ok(())
}

fn normalize_html(s: &str) -> String {
    normalize_newlines(s.trim_end())
}

fn diff_lines(expected: &str, got: &str) -> String {
    let mut out = String::new();
    for (i, (e, g)) in expected.lines().zip(got.lines()).enumerate() {
        if e != g {
            out.push_str(&format!("@@ line {} @@\n- {e}\n+ {g}\n", i + 1));
        }
    }
    if expected.lines().count() != got.lines().count() {
        out.push_str(&format!(
            "line count: expected {} got {}\n",
            expected.lines().count(),
            got.lines().count()
        ));
    }
    if out.is_empty() {
        out.push_str("(whitespace-only drift)\n");
        out.push_str(&format!(
            "expected len {} got len {}\n",
            expected.len(),
            got.len()
        ));
    }
    out
}
