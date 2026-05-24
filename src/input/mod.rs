//! Resolve CLI / filesystem inputs into descriptors for [`GenerateInput`](crate::GenerateInput).

mod buf;
mod fds;
mod merge;
mod protoc;

use crate::GenerateInput;
use crate::options::Options;
use crate::plugin_api::FileDescriptorProto;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

pub use buf::{compile_with_buf, resolve_buf_path};
pub use fds::{load_descriptor_set, read_request_stdin};
pub use merge::{filter_file_to_generate, merge_proto_files};
pub use protoc::{compile_with_protoc, resolve_protoc_path};

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
    /// Build generation input; CLI search roots travel on the struct, not in `parameter`.
    pub fn into_generate_input(self, options: Options) -> GenerateInput {
        GenerateInput {
            proto_file: self.proto_file,
            file_to_generate: self.file_to_generate,
            parameter: None,
            options: Some(options),
            proto_search_paths: self.proto_search_paths,
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

/// Run `runner` to write a descriptor set, then load protobuf files from it.
pub(crate) fn compile_to_fds(
    runner: impl FnOnce(&Path) -> Result<()>,
) -> Result<Vec<FileDescriptorProto>> {
    let fds_file = tempfile::Builder::new()
        .prefix("protobuf-mdbook-")
        .suffix(".binpb")
        .tempfile()
        .context("create temp descriptor set")?;
    let fds_path = fds_file.path();
    runner(fds_path)?;
    let (proto_file, _) = load_descriptor_set(fds_path)?;
    Ok(proto_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

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
