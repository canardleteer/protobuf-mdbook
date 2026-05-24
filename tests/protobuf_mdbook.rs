//! CLI-specific integration tests (`protobuf-mdbook` input paths not covered elsewhere).

mod common;

use buffa::Message;
use buffa_descriptor::generated::compiler::CodeGeneratorRequest;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn cli_bin() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_protobuf-mdbook").into()
}

fn run_cli(args: &[&str], out: &Path) -> Command {
    let mut cmd = Command::new(cli_bin());
    cmd.arg("-o").arg(out);
    for a in args {
        cmd.arg(a);
    }
    cmd
}

fn run_cli_raw(args: &[&str]) -> Command {
    let mut cmd = Command::new(cli_bin());
    for a in args {
        cmd.arg(a);
    }
    cmd
}

/// `--descriptor-set` decodes a protoc-emitted FDS without a live compiler at CLI runtime.
#[test]
fn descriptor_set_roundtrip_emits_package_markdown() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let fixture_dir = common::fixtures_dir();
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
    assert!(status.success(), "protoc descriptor_set_out failed");

    let out = tempfile::tempdir().expect("tempdir");
    let status = run_cli(
        &["--descriptor-set", fds.path().to_str().expect("utf8")],
        out.path(),
    )
    .status()
    .expect("spawn protobuf-mdbook");
    assert!(status.success(), "protobuf-mdbook --descriptor-set failed");

    common::assert_fixture_markdown_only(out.path());
}

/// Default buf compiler on a full module root (no per-file filter).
#[test]
fn buf_module_root_without_file_filter() {
    if !common::buf_available() {
        eprintln!("skip: buf not on PATH");
        return;
    }
    let out = tempfile::tempdir().expect("tempdir");
    let proto_root = common::examples_proto_dir();
    let status = run_cli(&[proto_root.to_str().expect("utf8")], out.path())
        .status()
        .expect("spawn protobuf-mdbook");
    assert!(status.success(), "protobuf-mdbook buf module root failed");

    protobuf_mdbook::link_check::assert_tree(out.path()).expect("buf module links");
}

#[test]
fn missing_output_flag_fails() {
    let fixture_dir = common::fixtures_dir();
    let out = run_cli_raw(&[
        "--compiler",
        "protoc",
        "-I",
        fixture_dir.to_str().expect("utf8"),
        "doc_rich.proto",
    ])
    .current_dir(&fixture_dir)
    .output()
    .expect("spawn");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("-o") || stderr.contains("--output"),
        "stderr: {stderr}"
    );
}

#[test]
fn request_and_inputs_conflict_fails() {
    let out = tempfile::tempdir().expect("tempdir");
    let fixture = common::fixtures_dir().join("doc_rich.proto");
    let output = run_cli_raw(&[
        "-o",
        out.path().to_str().expect("utf8"),
        "--request",
        fixture.to_str().expect("utf8"),
    ])
    .output()
    .expect("spawn");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--request"), "stderr: {stderr}");
}

#[test]
fn request_stdin_roundtrip() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let fixture_dir = common::fixtures_dir();
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

    let bytes = std::fs::read(fds.path()).expect("read fds");
    let set = FileDescriptorSet::decode_from_slice(&bytes).expect("decode fds");
    let req = CodeGeneratorRequest {
        proto_file: set.file,
        file_to_generate: vec!["doc_rich.proto".into()],
        parameter: Some("layout=package".into()),
        ..Default::default()
    };
    let wire = req.encode_to_vec();

    let out = tempfile::tempdir().expect("tempdir");
    let mut child = Command::new(cli_bin())
        .arg("-o")
        .arg(out.path())
        .arg("--request")
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&wire)
        .expect("write request");
    let status = child.wait().expect("wait");
    assert!(status.success(), "protobuf-mdbook --request failed");

    common::assert_fixture_markdown_only(out.path());
}
