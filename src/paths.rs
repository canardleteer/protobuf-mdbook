//! Shared path normalization and proto file discovery.

use anyhow::{Context, Result, bail};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

pub fn normalize_rel_path(path: &str, default: &str) -> Result<String> {
    let path = path.trim().trim_matches('/');
    if path.is_empty() {
        return Ok(default.to_string());
    }
    if path.contains("..") {
        bail!("path must not contain `..`: {path:?}");
    }
    Ok(path.replace('\\', "/"))
}

pub fn join_book_root(book_root: &str, rel: &str) -> String {
    let rel = rel.trim_start_matches('/');
    if book_root == "." {
        rel.to_string()
    } else {
        format!("{book_root}/{rel}")
    }
}

/// Strip non-normal path components (`.`, `..`, prefixes).
pub fn normalize_rel_dir(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_owned()),
            _ => None,
        })
        .collect()
}

pub fn is_proto_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("proto")
}

/// Descriptor-style name for a `.proto` path (forward slashes, no `..`).
pub fn proto_name_from_path(path: &Path) -> Result<String> {
    let name = path.to_string_lossy().replace('\\', "/");
    if name.contains("..") {
        bail!("proto path must not contain `..`: {name}");
    }
    Ok(name.trim_start_matches("./").to_string())
}

/// Path relative to a module root as a `/`-separated descriptor name.
pub fn proto_name_relative_to_module(module_root: &Path, path: &Path) -> Result<String> {
    let module_root = module_root
        .canonicalize()
        .unwrap_or_else(|_| module_root.to_path_buf());
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let rel = path.strip_prefix(&module_root).with_context(|| {
        format!(
            "{} is not under module root {}",
            path.display(),
            module_root.display()
        )
    })?;
    let mut parts = Vec::new();
    for c in rel.components() {
        match c {
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::CurDir => {}
            other => bail!("invalid path component in {}: {other:?}", path.display()),
        }
    }
    Ok(parts.join("/"))
}

/// Collect `.proto` files from explicit file or directory inputs.
pub fn collect_proto_inputs(inputs: &[PathBuf]) -> Result<(Vec<PathBuf>, Vec<String>)> {
    let mut rel_files = Vec::new();
    let mut generate_names = Vec::new();

    for input in inputs {
        if input.is_file() {
            if input.extension().and_then(|e| e.to_str()) != Some("proto") {
                bail!("not a .proto file: {}", input.display());
            }
            let name = proto_name_from_path(input)?;
            rel_files.push(input.to_path_buf());
            generate_names.push(name);
            continue;
        }
        if input.is_dir() {
            for entry in WalkDir::new(input).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if is_proto_file(path) {
                    let name = proto_name_from_path(path)?;
                    rel_files.push(path.to_path_buf());
                    generate_names.push(name);
                }
            }
            continue;
        }
        bail!("input not found: {}", input.display());
    }

    rel_files.sort();
    rel_files.dedup();
    generate_names.sort();
    generate_names.dedup();
    Ok((rel_files, generate_names))
}

/// All `.proto` files under `root` whose descriptor names appear in `proto_file`.
pub fn collect_module_proto_names(
    module_root: &Path,
    proto_file: &[crate::plugin_api::FileDescriptorProto],
) -> Vec<String> {
    let module_root = module_root
        .canonicalize()
        .unwrap_or_else(|_| module_root.to_path_buf());
    let mut names = Vec::new();
    for entry in WalkDir::new(&module_root)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !is_proto_file(path) {
            continue;
        }
        if let Ok(name) = proto_name_relative_to_module(&module_root, path)
            && proto_file
                .iter()
                .any(|f| f.name.as_deref() == Some(name.as_str()))
        {
            names.push(name);
        }
    }
    names.sort();
    names.dedup();
    names
}
