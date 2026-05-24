//! Buf module compilation.

use crate::input::{ResolveArgs, ResolvedInput, compile_to_fds};
use crate::paths::{collect_module_proto_names, is_proto_file, proto_name_relative_to_module};
use crate::plugin_api::FileDescriptorProto;
use crate::proto_deps;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

pub(crate) const BUF_INSTALL_HINT: &str = "cargo install buf-toolchain --locked --version 1.69.0";

pub fn compile_with_buf(args: &ResolveArgs) -> Result<ResolvedInput> {
    if args.inputs.is_empty() {
        bail!("no inputs provided for buf compilation");
    }

    let module_root = find_buf_module_root(&args.inputs[0])?.with_context(|| {
        format!(
            "no Buf module (buf.yaml) found for {}; pass a module root or use `--compiler protoc`",
            args.inputs[0].display()
        )
    })?;

    let buf = resolve_buf_path(args.buf_path.as_deref())?;
    let proto_file = compile_to_fds(|fds_path| {
        let status = Command::new(&buf)
            .current_dir(&module_root)
            .args(["build", "-o"])
            .arg(fds_path)
            .status()
            .with_context(|| format!("spawn {}", buf.display()))?;
        if !status.success() {
            bail!("buf build failed in {}", module_root.display());
        }
        Ok(())
    })?;
    let file_to_generate =
        resolve_file_to_generate_for_inputs(&module_root, &args.inputs, &proto_file)?;

    let mut proto_search_paths = vec![module_root.clone()];
    if let Some(export) = &args.proto_deps_export {
        proto_deps::ensure_proto_deps_export(&module_root, export, false)?;
        proto_search_paths.push(export.clone());
    }
    proto_search_paths.extend(args.proto_paths.iter().cloned());

    Ok(ResolvedInput {
        proto_file,
        file_to_generate,
        proto_search_paths,
    })
}

pub fn resolve_buf_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if tool_exists("buf") {
        return Ok(PathBuf::from("buf"));
    }
    bail!("buf not found on PATH; install with: {BUF_INSTALL_HINT}");
}

fn find_buf_module_root(start: &Path) -> Result<Option<PathBuf>> {
    let start = if start.is_file() {
        start
            .parent()
            .context("input file has no parent directory")?
    } else {
        start
    };
    let mut dir = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if dir.join("buf.yaml").is_file() || dir.join("buf.yml").is_file() {
            return Ok(Some(dir));
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(None)
}

fn resolve_file_to_generate_for_inputs(
    module_root: &Path,
    inputs: &[PathBuf],
    proto_file: &[FileDescriptorProto],
) -> Result<Vec<String>> {
    let explicit: Vec<PathBuf> = inputs
        .iter()
        .flat_map(|input| {
            if input.is_file() {
                vec![input.clone()]
            } else if input.is_dir() && input == module_root {
                Vec::new()
            } else if input.is_dir() {
                WalkDir::new(input)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| is_proto_file(e.path()))
                    .map(|e| e.path().to_path_buf())
                    .collect()
            } else {
                Vec::new()
            }
        })
        .collect();

    if explicit.is_empty() {
        return Ok(collect_module_proto_names(module_root, proto_file));
    }

    let mut names = Vec::new();
    for path in explicit {
        let name = proto_name_relative_to_module(module_root, &path)?;
        if proto_file
            .iter()
            .any(|f| f.name.as_deref() == Some(name.as_str()))
        {
            names.push(name);
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn tool_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success())
        || Command::new(name)
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .is_some_and(|s| s.success())
}
