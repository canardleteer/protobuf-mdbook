//! Resolve CLI / filesystem inputs into descriptors for [`GenerateInput`](crate::GenerateInput).

use crate::GenerateInput;
use crate::plugin_api::{CodeGeneratorRequest, FileDescriptorProto};
use crate::proto_deps;
use anyhow::{Context, Result, bail};
use buffa::Message;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

const BUF_INSTALL_HINT: &str = "cargo install buf-toolchain --locked --version 1.69.0";

/// Compiler for `.proto` inputs when not using a prebuilt descriptor set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Compiler {
    #[default]
    Buf,
    Protoc,
}

/// Arguments for input resolution (from the CLI or tests).
#[derive(Clone, Debug, Default)]
pub struct ResolveArgs {
    pub compiler: Compiler,
    pub descriptor_sets: Vec<PathBuf>,
    pub inputs: Vec<PathBuf>,
    pub proto_paths: Vec<PathBuf>,
    pub protoc_path: Option<PathBuf>,
    pub buf_path: Option<PathBuf>,
    pub proto_deps_export: Option<PathBuf>,
    pub from_request: bool,
}

/// Resolved descriptor payload before option parsing.
#[derive(Clone, Debug)]
pub struct ResolvedInput {
    pub proto_file: Vec<FileDescriptorProto>,
    pub file_to_generate: Vec<String>,
    pub proto_search_paths: Vec<PathBuf>,
}

impl ResolvedInput {
    pub fn into_generate_input(self, parameter: Option<String>) -> GenerateInput {
        let mut param = parameter.unwrap_or_default();
        if !self.proto_search_paths.is_empty() {
            let paths = self
                .proto_search_paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(":");
            if param.is_empty() {
                param = format!("proto_path={paths}");
            } else {
                param.push_str(&format!(",proto_path={paths}"));
            }
        }
        GenerateInput {
            proto_file: self.proto_file,
            file_to_generate: self.file_to_generate,
            parameter: if param.is_empty() { None } else { Some(param) },
        }
    }
}

/// Resolve inputs into descriptors and generation targets.
pub fn resolve_inputs(args: &ResolveArgs) -> Result<ResolvedInput> {
    if args.from_request {
        return read_request_stdin();
    }

    let mut proto_file = Vec::new();
    let mut file_to_generate = Vec::new();
    let mut proto_search_paths = args.proto_paths.clone();

    for path in &args.descriptor_sets {
        let (files, names) = load_descriptor_set(path)?;
        merge_proto_files(&mut proto_file, files);
        file_to_generate.extend(names);
    }

    if !args.descriptor_sets.is_empty() && args.inputs.is_empty() {
        // descriptor-set only
    } else if !args.inputs.is_empty() {
        let compiled = match args.compiler {
            Compiler::Buf => compile_with_buf(args)?,
            Compiler::Protoc => compile_with_protoc(args)?,
        };
        merge_proto_files(&mut proto_file, compiled.proto_file);
        if file_to_generate.is_empty() {
            file_to_generate = compiled.file_to_generate;
        } else {
            file_to_generate = filter_file_to_generate(&proto_file, &file_to_generate);
        }
        for p in compiled.proto_search_paths {
            if !proto_search_paths.iter().any(|x| x == &p) {
                proto_search_paths.push(p);
            }
        }
    } else {
        bail!("no inputs: pass proto paths, --descriptor-set, or --request -");
    }

    if proto_file.is_empty() {
        bail!("no protobuf descriptors resolved from inputs");
    }

    file_to_generate.sort();
    file_to_generate.dedup();
    if file_to_generate.is_empty() {
        bail!("file_to_generate is empty after resolving inputs");
    }

    Ok(ResolvedInput {
        proto_file,
        file_to_generate,
        proto_search_paths,
    })
}

fn read_request_stdin() -> Result<ResolvedInput> {
    let mut stdin = Vec::new();
    std::io::stdin()
        .read_to_end(&mut stdin)
        .context("read CodeGeneratorRequest from stdin")?;
    let req = CodeGeneratorRequest::decode_from_slice(&stdin)
        .map_err(|e| anyhow::anyhow!("decode CodeGeneratorRequest: {e}"))?;
    Ok(ResolvedInput {
        proto_file: req.proto_file,
        file_to_generate: req.file_to_generate,
        proto_search_paths: Vec::new(),
    })
}

fn load_descriptor_set(path: &Path) -> Result<(Vec<FileDescriptorProto>, Vec<String>)> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read descriptor set {}", path.display()))?;
    let set = FileDescriptorSet::decode_from_slice(&bytes)
        .map_err(|e| anyhow::anyhow!("decode FileDescriptorSet {}: {e}", path.display()))?;
    let files = set.file;
    let names: Vec<String> = files.iter().filter_map(|f| f.name.clone()).collect();
    Ok((files, names))
}

fn merge_proto_files(into: &mut Vec<FileDescriptorProto>, files: Vec<FileDescriptorProto>) {
    let mut by_name: BTreeMap<String, FileDescriptorProto> = into
        .drain(..)
        .filter_map(|f| f.name.clone().map(|n| (n, f)))
        .collect();
    for f in files {
        if let Some(name) = f.name.clone() {
            by_name.insert(name, f);
        }
    }
    *into = by_name.into_values().collect();
}

fn filter_file_to_generate(proto_file: &[FileDescriptorProto], wanted: &[String]) -> Vec<String> {
    let known: BTreeMap<_, _> = proto_file
        .iter()
        .filter_map(|f| f.name.as_deref().map(|n| (n, ())))
        .collect();
    wanted
        .iter()
        .filter(|n| known.contains_key(n.as_str()))
        .cloned()
        .collect()
}

fn compile_with_buf(args: &ResolveArgs) -> Result<ResolvedInput> {
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
    let fds_path = tempfile::Builder::new()
        .prefix("protobuf-mdbook-")
        .suffix(".binpb")
        .tempfile()
        .context("create temp descriptor set")?;
    let fds_path = fds_path.path().to_path_buf();

    let status = Command::new(&buf)
        .current_dir(&module_root)
        .args(["build", "-o"])
        .arg(&fds_path)
        .status()
        .with_context(|| format!("spawn {}", buf.display()))?;
    if !status.success() {
        bail!("buf build failed in {}", module_root.display());
    }

    let (proto_file, _) = load_descriptor_set(&fds_path)?;
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

fn compile_with_protoc(args: &ResolveArgs) -> Result<ResolvedInput> {
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
    let fds_path = tempfile::Builder::new()
        .prefix("protobuf-mdbook-")
        .suffix(".binpb")
        .tempfile()
        .context("create temp descriptor set")?;
    let fds_path = fds_path.path().to_path_buf();

    let mut cmd = Command::new(&protoc);
    cmd.arg("--descriptor_set_out").arg(&fds_path);
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

    let (proto_file, _) = load_descriptor_set(&fds_path)?;
    let file_to_generate: Vec<String> = protoc_inputs.into_iter().map(|(_, name)| name).collect();

    Ok(ResolvedInput {
        proto_file,
        file_to_generate,
        proto_search_paths: include_paths,
    })
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

fn collect_proto_inputs(inputs: &[PathBuf]) -> Result<(Vec<PathBuf>, Vec<String>)> {
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
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("proto") {
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
                    .filter(|e| {
                        e.path().is_file()
                            && e.path().extension().and_then(|x| x.to_str()) == Some("proto")
                    })
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

fn collect_module_proto_names(
    module_root: &Path,
    proto_file: &[FileDescriptorProto],
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
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("proto") {
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

fn proto_name_from_path(path: &Path) -> Result<String> {
    let name = path.to_string_lossy().replace('\\', "/");
    if name.contains("..") {
        bail!("proto path must not contain `..`: {name}");
    }
    Ok(name.trim_start_matches("./").to_string())
}

fn proto_name_relative_to_module(module_root: &Path, path: &Path) -> Result<String> {
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

pub fn resolve_buf_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if tool_exists("buf") {
        return Ok(PathBuf::from("buf"));
    }
    bail!("buf not found on PATH; install with: {BUF_INSTALL_HINT}");
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

fn tool_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .status()
        .ok()
        .is_some_and(|s| s.success())
        || Command::new(name)
            .arg("version")
            .status()
            .ok()
            .is_some_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_proto() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/doc_rich.proto")
    }

    #[test]
    fn resolve_inputs_empty_fails() {
        let err = resolve_inputs(&ResolveArgs::default()).expect_err("empty");
        assert!(err.to_string().contains("no inputs"));
    }

    #[test]
    fn resolve_inputs_non_proto_file_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bad = dir.path().join("foo.txt");
        std::fs::write(&bad, "nope").expect("write");
        let args = ResolveArgs {
            compiler: Compiler::Protoc,
            inputs: vec![bad],
            ..Default::default()
        };
        let err = resolve_inputs(&args).expect_err("not proto");
        assert!(err.to_string().contains("not a .proto file"));
    }

    #[test]
    fn resolve_inputs_buf_without_module_fails() {
        let args = ResolveArgs {
            compiler: Compiler::Buf,
            inputs: vec![fixture_proto()],
            ..Default::default()
        };
        let err = resolve_inputs(&args).expect_err("no buf module");
        let msg = err.to_string();
        assert!(msg.contains("buf.yaml") || msg.contains("--compiler protoc"));
    }

    #[test]
    fn resolve_protoc_explicit_path() {
        let path = PathBuf::from("/custom/protoc");
        assert_eq!(resolve_protoc_path(Some(&path)).expect("path"), path);
    }

    #[test]
    fn resolve_inputs_protoc_missing_include_fails() {
        let args = ResolveArgs {
            compiler: Compiler::Protoc,
            inputs: vec![fixture_proto()],
            proto_paths: vec![PathBuf::from("/nowhere")],
            ..Default::default()
        };
        let err = resolve_inputs(&args).expect_err("outside include");
        assert!(err.to_string().contains("not under any --proto-path"));
    }

    #[test]
    fn resolve_inputs_descriptor_set_only() {
        let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let fds = tempfile::NamedTempFile::new().expect("temp fds");
        let status = Command::new(protoc)
            .args([
                "-I",
                fixture_dir.to_str().expect("utf8"),
                "--descriptor_set_out",
                fds.path().to_str().expect("utf8"),
                "--include_imports",
                "doc_rich.proto",
            ])
            .status()
            .expect("spawn protoc");
        assert!(status.success());

        let args = ResolveArgs {
            descriptor_sets: vec![fds.path().to_path_buf()],
            ..Default::default()
        };
        let resolved = resolve_inputs(&args).expect("fds only");
        assert!(!resolved.file_to_generate.is_empty());
        assert!(!resolved.proto_file.is_empty());
    }

    #[test]
    fn resolve_inputs_protoc_fixture_succeeds() {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let args = ResolveArgs {
            compiler: Compiler::Protoc,
            inputs: vec![fixture_proto()],
            proto_paths: vec![fixture_dir],
            ..Default::default()
        };
        let resolved = resolve_inputs(&args).expect("protoc resolve");
        assert!(
            resolved
                .file_to_generate
                .iter()
                .any(|n| n.contains("doc_rich.proto"))
        );
    }
}
