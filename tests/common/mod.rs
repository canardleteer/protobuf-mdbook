//! Shared helpers for protoc-plugin vs `protobuf-mdbook` CLI integration tests.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const EXAMPLE_PROTO_INPUTS: &[&str] = protobuf_mdbook::examples::EXAMPLE_PROTO_INPUTS;

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    ProtocPlugin,
    ProtobufMdbook,
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn examples_proto_dir() -> PathBuf {
    protobuf_mdbook::examples::examples_proto_dir()
}

pub fn fixtures_dir() -> PathBuf {
    manifest_dir().join("tests/fixtures")
}

/// Whether `buf` is on PATH (stdout/stderr discarded so parallel tests stay quiet).
pub fn buf_available() -> bool {
    Command::new("buf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

/// Backends for `examples/proto` tests (`protobuf-mdbook` uses default buf compiler).
pub fn mirrored_backends() -> Vec<Backend> {
    let mut backends = vec![Backend::ProtocPlugin];
    if buf_available() {
        backends.push(Backend::ProtobufMdbook);
    } else {
        eprintln!("mirrored_backends: skip protobuf-mdbook (buf not on PATH)");
    }
    backends
}

/// Backends for loose fixture protos (`protobuf-mdbook --compiler protoc`).
pub fn mirrored_fixture_backends() -> Vec<Backend> {
    vec![Backend::ProtocPlugin, Backend::ProtobufMdbook]
}

pub fn ensure_proto_deps_export() -> PathBuf {
    let proto_dir = examples_proto_dir();
    let export_dir = manifest_dir().join("target/proto-deps");
    protobuf_mdbook::proto_deps::ensure_proto_deps_export(&proto_dir, &export_dir, false)
        .expect("ensure proto deps export")
}

pub fn run_examples_in(out: &Path, layout: &str, extra_opt: &str, backend: Backend) {
    match backend {
        Backend::ProtocPlugin => run_protoc_examples_in(out, layout, extra_opt),
        Backend::ProtobufMdbook => run_cli_examples_in(out, layout, extra_opt),
    }
}

pub fn run_examples(layout: &str, extra_opt: &str, backend: Backend) -> tempfile::TempDir {
    let out = tempfile::tempdir().expect("tempdir");
    run_examples_in(out.path(), layout, extra_opt, backend);
    out
}

fn run_protoc_examples_in(out: &Path, layout: &str, extra_opt: &str) {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let proto_dir = examples_proto_dir();
    let deps = ensure_proto_deps_export();

    let opt = protobuf_mdbook::examples::format_examples_mdbook_opt(layout, extra_opt);

    let mut cmd = Command::new(protoc);
    cmd.current_dir(&proto_dir)
        .arg("-I")
        .arg(".")
        .arg("-I")
        .arg(deps.to_str().expect("utf8 proto-deps path"))
        .arg(format!("--plugin=protoc-gen-mdbook={}", plugin.display()))
        .arg(format!("--mdbook_out={}", out.display()))
        .arg(format!("--mdbook_opt={opt}"));
    for rel in EXAMPLE_PROTO_INPUTS {
        cmd.arg(rel);
    }
    let status = cmd.status().expect("spawn protoc");
    assert!(status.success(), "protoc layout={layout} opt={extra_opt}");
}

fn run_cli_examples_in(out: &Path, layout: &str, extra_opt: &str) {
    let cli: PathBuf = env!("CARGO_BIN_EXE_protobuf-mdbook").into();
    let proto_dir = examples_proto_dir();

    let opt = protobuf_mdbook::examples::format_examples_mdbook_opt(layout, extra_opt);

    let mut cmd = Command::new(cli);
    cmd.current_dir(&proto_dir)
        .arg("-o")
        .arg(out)
        .arg("--opt")
        .arg(&opt);
    for rel in EXAMPLE_PROTO_INPUTS {
        cmd.arg(rel);
    }
    let status = cmd.status().expect("spawn protobuf-mdbook");
    assert!(
        status.success(),
        "protobuf-mdbook layout={layout} opt={extra_opt}"
    );
}

pub fn run_fixture_protoc(out: &Path, extra_opt: &str) {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let fixture_dir = fixtures_dir();
    let plugin_arg = format!("--plugin=protoc-gen-mdbook={}", plugin.display());
    let out_arg = format!("--mdbook_out={}", out.display());

    let mut cmd = Command::new(protoc);
    cmd.args([
        "-I",
        fixture_dir.to_str().expect("utf8"),
        &plugin_arg,
        &out_arg,
    ]);
    if !extra_opt.is_empty() {
        cmd.arg(format!("--mdbook_opt={extra_opt}"));
    }
    cmd.arg("doc_rich.proto");

    let status = cmd.status().expect("spawn protoc");
    assert!(status.success(), "protoc fixture opt={extra_opt}");
}

pub fn run_fixture_cli(out: &Path, extra_opt: &str) {
    let cli: PathBuf = env!("CARGO_BIN_EXE_protobuf-mdbook").into();
    let fixture_dir = fixtures_dir();

    let mut cmd = Command::new(cli);
    cmd.arg("-o")
        .arg(out)
        .args(["--compiler", "protoc", "-I"])
        .arg(&fixture_dir)
        .arg("doc_rich.proto")
        .current_dir(&fixture_dir);
    if !extra_opt.is_empty() {
        cmd.args(["--opt", extra_opt]);
    }
    let status = cmd.status().expect("spawn protobuf-mdbook");
    assert!(status.success(), "protobuf-mdbook fixture opt={extra_opt}");
}

pub fn run_fixture_in(out: &Path, extra_opt: &str, backend: Backend) {
    match backend {
        Backend::ProtocPlugin => run_fixture_protoc(out, extra_opt),
        Backend::ProtobufMdbook => run_fixture_cli(out, extra_opt),
    }
}

pub fn run_fixture(extra_opt: &str, backend: Backend) -> tempfile::TempDir {
    let out = tempfile::tempdir().expect("tempdir");
    run_fixture_in(out.path(), extra_opt, backend);
    out
}

pub fn assert_fixture_markdown_only(out: &Path) {
    assert!(
        !out.join("book.toml").exists(),
        "default mode must not scaffold mdBook"
    );
    let package_md = out.join("src").join("packages").join("acme.example.v1.md");
    assert!(package_md.is_file(), "expected {}", package_md.display());

    let pkg = std::fs::read_to_string(&package_md).expect("read package md");
    assert!(pkg.contains("EchoUnaryRequest"));
    assert!(pkg.contains("```protobuf"));
    assert!(pkg.contains("message EchoUnaryRequest"));
    assert!(!pkg.contains("| # | Name | Type |"));

    protobuf_mdbook::link_check::assert_tree(out).expect("links resolve");
}

pub fn assert_fixture_init(out: &Path) {
    let book_toml = out.join("book.toml");
    let readme = out.join("README.md");
    let summary = out.join("src").join("SUMMARY.md");
    let package_md = out.join("src").join("packages").join("acme.example.v1.md");
    assert!(book_toml.is_file(), "expected {}", book_toml.display());
    assert!(readme.is_file(), "expected {}", readme.display());
    assert!(summary.is_file(), "expected {}", summary.display());
    assert!(package_md.is_file(), "expected {}", package_md.display());

    let sum = std::fs::read_to_string(&summary).expect("read SUMMARY.md");
    assert!(sum.contains("acme.example.v1"));
    assert!(
        sum.starts_with("# Protobuf documentation"),
        "init SUMMARY H1 uses book title"
    );
    assert!(
        !sum.contains("chapter_1"),
        "SUMMARY should list packages, not init stub"
    );

    let book = std::fs::read_to_string(&book_toml).expect("book.toml");
    assert!(
        book.contains("Protobuf documentation"),
        "default init title when title= omitted"
    );

    let pkg = std::fs::read_to_string(&package_md).expect("read package md");
    let svc = pkg
        .split("### EchoService")
        .nth(1)
        .expect("EchoService section");
    let mermaid = svc.find("```mermaid").expect("mermaid");
    let sig = svc.find("**EchoUnary**").expect("EchoUnary signature");
    assert!(mermaid < sig, "service mermaid before RPC signatures");
    assert!(svc.contains("EchoUnary RPC docs"));
    assert!(!svc.contains("service EchoService"));

    protobuf_mdbook::link_check::assert_tree(out).expect("links resolve");
}

pub fn run_single_echo_package(out: &Path, backend: Backend) {
    match backend {
        Backend::ProtocPlugin => {
            let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
            let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
            let proto_dir = examples_proto_dir();
            let deps = ensure_proto_deps_export();
            let status = Command::new(protoc)
                .current_dir(&proto_dir)
                .args([
                    "-I",
                    ".",
                    "-I",
                    deps.to_str().expect("utf8 proto-deps path"),
                    &format!("--plugin=protoc-gen-mdbook={}", plugin.display()),
                    &format!("--mdbook_out={}", out.display()),
                    "--mdbook_opt=layout=package",
                    "acme/example/v1/echo.proto",
                ])
                .status()
                .expect("spawn protoc");
            assert!(status.success());
        }
        Backend::ProtobufMdbook => {
            let cli: PathBuf = env!("CARGO_BIN_EXE_protobuf-mdbook").into();
            let proto_dir = examples_proto_dir();
            let status = Command::new(cli)
                .current_dir(&proto_dir)
                .args([
                    "-o",
                    out.to_str().expect("utf8"),
                    "--opt",
                    "layout=package,proto_path=.",
                    "acme/example/v1/echo.proto",
                ])
                .status()
                .expect("spawn protobuf-mdbook");
            assert!(status.success());
        }
    }
}
