//! protoc compilation and include-path resolution.

use crate::input::{ResolveArgs, ResolvedInput, compile_to_fds};
use crate::paths::collect_proto_inputs;
use crate::proto_deps;
use anyhow::{Context, Result, bail};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

pub fn compile_with_protoc(args: &ResolveArgs) -> Result<ResolvedInput> {
    if args.inputs.is_empty() {
        bail!("no inputs provided for protoc compilation");
    }

    let mut include_paths = args.proto_paths.clone();
    if include_paths.is_empty() {
        include_paths.push(infer_proto_root(&args.inputs)?);
    }

    if let Some(export_dir) = &args.proto_deps_export {
        let module_root = infer_proto_root(&args.inputs)?;
        proto_deps::ensure_proto_deps_export(&module_root, export_dir, false)?;
        if !include_paths.iter().any(|p| p == export_dir) {
            include_paths.push(export_dir.clone());
        }
    }

    let protoc_inputs = resolve_protoc_file_args(&args.inputs, &include_paths)?;
    if protoc_inputs.is_empty() {
        bail!("no .proto inputs found");
    }

    let protoc = resolve_protoc_path(args.protoc_path.as_deref())?;
    let proto_file = compile_to_fds(|fds_path| {
        let mut cmd = Command::new(&protoc);
        cmd.arg("--descriptor_set_out").arg(fds_path);
        cmd.arg("--include_imports");
        cmd.arg("--include_source_info");
        for inc in &include_paths {
            cmd.arg("-I").arg(inc);
        }
        for (protoc_arg, _) in &protoc_inputs {
            cmd.arg(protoc_arg);
        }

        let status = cmd
            .status()
            .with_context(|| format!("spawn {}", protoc.display()))?;
        if !status.success() {
            bail!("protoc failed");
        }
        Ok(())
    })?;
    let file_to_generate: Vec<String> = protoc_inputs.into_iter().map(|(_, name)| name).collect();

    Ok(ResolvedInput {
        proto_file,
        file_to_generate,
        proto_search_paths: include_paths,
    })
}

pub fn resolve_protoc_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if tool_exists("protoc") {
        return Ok(PathBuf::from("protoc"));
    }
    protoc_bin_vendored::protoc_bin_path().map_err(|e| {
        anyhow::anyhow!(
            "protoc not found on PATH and vendored protoc unavailable ({e}); \
             install protoc via your OS package manager or https://github.com/protocolbuffers/protobuf/releases, \
             pass `--protoc PATH`, or use default `--compiler buf` for Buf modules"
        )
    })
}

fn infer_proto_root(inputs: &[PathBuf]) -> Result<PathBuf> {
    for input in inputs {
        let path = if input.is_file() {
            input.parent().context("input file has no parent")?
        } else {
            input.as_path()
        };
        if path.join("buf.yaml").is_file() {
            return Ok(path.to_path_buf());
        }
    }
    if let Some(first) = inputs.first() {
        if first.is_file() {
            return Ok(first
                .parent()
                .context("input file has no parent")?
                .to_path_buf());
        }
        return Ok(first.clone());
    }
    bail!("no inputs to infer proto root");
}

/// `(protoc CLI path, descriptor name)` pairs relative to `-I` include paths.
fn resolve_protoc_file_args(
    inputs: &[PathBuf],
    include_paths: &[PathBuf],
) -> Result<Vec<(PathBuf, String)>> {
    let mut canon_includes = Vec::new();
    for inc in include_paths {
        let c = inc.canonicalize().unwrap_or_else(|_| inc.clone());
        canon_includes.push(c);
    }
    canon_includes.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

    let (abs_files, _) = collect_proto_inputs(inputs)?;
    let mut out = Vec::new();
    for abs in abs_files {
        let canonical = abs.canonicalize().unwrap_or(abs);
        let mut matched = false;
        for inc in &canon_includes {
            if let Ok(rel) = canonical.strip_prefix(inc) {
                let name = rel
                    .components()
                    .filter_map(|c| match c {
                        Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                        Component::CurDir => None,
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("/");
                if name.is_empty() {
                    continue;
                }
                out.push((PathBuf::from(&name), name));
                matched = true;
                break;
            }
        }
        if !matched {
            bail!(
                "{} is not under any --proto-path (-I) directory; add an include path",
                canonical.display()
            );
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out.dedup_by(|a, b| a.1 == b.1);
    Ok(out)
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
