//! Shared helpers for protoc-plugin vs `protobuf-mdbook` CLI integration tests.

#![allow(dead_code)]

pub use protobuf_mdbook::examples::EXAMPLE_PROTO_INPUTS;
use protobuf_mdbook::examples::format_examples_mdbook_opt;
use protobuf_mdbook::input::Compiler;
use protobuf_mdbook::runner::{Driver, RunSpec, run_generation};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    ProtocPlugin,
    ProtobufMdbook,
}

impl Backend {
    fn driver(self) -> Driver {
        match self {
            Backend::ProtocPlugin => Driver::protoc_plugin(
                protoc_bin_vendored::protoc_bin_path().expect("vendored protoc"),
                PathBuf::from(env!("CARGO_BIN_EXE_protoc-gen-mdbook")),
            ),
            Backend::ProtobufMdbook => Driver::cli(
                PathBuf::from(env!("CARGO_BIN_EXE_protobuf-mdbook")),
                Compiler::Buf,
            ),
        }
    }

    fn fixture_driver(self) -> Driver {
        match self {
            Backend::ProtocPlugin => self.driver(),
            Backend::ProtobufMdbook => Driver::cli(
                PathBuf::from(env!("CARGO_BIN_EXE_protobuf-mdbook")),
                Compiler::Protoc,
            ),
        }
    }
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
    run_examples_proto_in(out, EXAMPLE_PROTO_INPUTS, layout, extra_opt, backend);
}

pub fn run_examples_proto_in(
    out: &Path,
    proto_inputs: &[&str],
    layout: &str,
    extra_opt: &str,
    backend: Backend,
) {
    let proto_dir = examples_proto_dir();
    let deps = ensure_proto_deps_export();
    let opt = format_examples_mdbook_opt(layout, extra_opt);
    let inputs: Vec<PathBuf> = proto_inputs.iter().map(|rel| PathBuf::from(*rel)).collect();
    let search_paths = vec![PathBuf::from("."), deps];
    let spec = RunSpec {
        out,
        mdbook_opt: &opt,
        inputs: &inputs,
        search_paths: &search_paths,
        cwd: Some(&proto_dir),
    };
    run_generation(&spec, &backend.driver()).unwrap_or_else(|e| {
        panic!("examples generation layout={layout} opt={extra_opt} ({backend:?}): {e:#}")
    });
}

pub fn run_examples(layout: &str, extra_opt: &str, backend: Backend) -> tempfile::TempDir {
    let out = tempfile::tempdir().expect("tempdir");
    run_examples_in(out.path(), layout, extra_opt, backend);
    out
}

pub fn run_fixture_proto_in(out: &Path, proto_file: &str, extra_opt: &str, backend: Backend) {
    let fixture_dir = fixtures_dir();
    let inputs = vec![PathBuf::from(proto_file)];
    let search_paths = vec![fixture_dir.clone()];
    let spec = RunSpec {
        out,
        mdbook_opt: extra_opt,
        inputs: &inputs,
        search_paths: &search_paths,
        cwd: Some(&fixture_dir),
    };
    run_generation(&spec, &backend.fixture_driver()).unwrap_or_else(|e| {
        panic!("fixture proto={proto_file} opt={extra_opt} ({backend:?}): {e:#}")
    });
}

pub fn run_fixture_in(out: &Path, extra_opt: &str, backend: Backend) {
    run_fixture_proto_in(out, "doc_rich.proto", extra_opt, backend);
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
    run_examples_proto_in(out, &["acme/example/v1/echo.proto"], "package", "", backend);
}

/// Build a descriptor set for `doc_rich.proto` via vendored protoc.
pub fn build_fixture_fds() -> tempfile::NamedTempFile {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let fixture_dir = fixtures_dir();
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
    fds
}

/// Run a test body for each backend returned by `mirrored_backends()`.
#[macro_export]
macro_rules! mirrored_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            for backend in $crate::common::mirrored_backends() {
                $body(backend);
            }
        }
    };
}
