//! Resolve relative Markdown links in a generated documentation tree.

use crate::render::links::unique_heading_ids;
use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct LinkError {
    pub file: PathBuf,
    pub target: String,
    pub message: String,
}

pub fn check_tree(root: &Path) -> Result<Vec<LinkError>> {
    let mut errors = Vec::new();
    let mut headings_by_file: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let content =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        headings_by_file.insert(path.to_path_buf(), extract_heading_ids(&content));
    }

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let content =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        for (target, line) in extract_links(&content) {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            let (file_part, anchor) = split_anchor(&target);
            let resolved = if file_part.is_empty() {
                path.to_path_buf()
            } else {
                path.parent().unwrap_or(root).join(file_part)
            };
            if !resolved.exists() {
                errors.push(LinkError {
                    file: path.to_path_buf(),
                    target: target.clone(),
                    message: format!("broken link at line {line}: file not found"),
                });
                continue;
            }
            if let Some(anchor) = anchor {
                let headings = headings_by_file.get(&resolved).cloned().unwrap_or_default();
                if !headings.iter().any(|h| h == anchor) {
                    errors.push(LinkError {
                        file: path.to_path_buf(),
                        target: target.clone(),
                        message: format!("broken anchor #{anchor} at line {line}"),
                    });
                }
            }
        }
    }

    Ok(errors)
}

pub fn assert_tree(root: &Path) -> Result<()> {
    let errors = check_tree(root)?;
    if errors.is_empty() {
        return Ok(());
    }
    let mut msg = String::from("markdown link check failed:\n");
    for e in &errors {
        msg.push_str(&format!(
            "  {}: {} — {}\n",
            e.file.display(),
            e.target,
            e.message
        ));
    }
    bail!("{msg}");
}

fn split_anchor(target: &str) -> (&str, Option<&str>) {
    match target.split_once('#') {
        Some((f, a)) => (f, Some(a)),
        None => (target, None),
    }
}

fn extract_heading_ids(content: &str) -> Vec<String> {
    let titles: Vec<&str> = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let hashes = line.chars().take_while(|c| *c == '#').count();
            if hashes == 0 {
                return None;
            }
            Some(line[hashes..].trim())
        })
        .collect();
    unique_heading_ids(titles)
}

fn extract_links(content: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let before = &rest[..start];
            if before.rfind('[').is_some() {
                let target_start = start + 2;
                if let Some(end) = rest[target_start..].find(')') {
                    let target = &rest[target_start..target_start + end];
                    out.push((target.to_string(), i + 1));
                    rest = &rest[target_start + end + 1..];
                    continue;
                }
            }
            break;
        }
    }
    out
}
