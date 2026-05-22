//! Integration test: stock `protoc` drives `protoc-gen-mdbook`.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn protoc_default_emits_markdown_only() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures");

    let out = tempfile::tempdir().expect("tempdir");
    let out_arg = format!("--mdbook_out={}", out.path().display());
    let plugin_arg = format!("--plugin=protoc-gen-mdbook={}", plugin.display());

    let status = Command::new(protoc)
        .args([
            "-I",
            fixture_dir.to_str().expect("utf8"),
            &plugin_arg,
            &out_arg,
            "doc_rich.proto",
        ])
        .status()
        .expect("spawn protoc");

    assert!(status.success(), "protoc exit: {status:?}");

    assert!(
        !out.path().join("book.toml").exists(),
        "default mode must not scaffold mdBook"
    );
    let package_md = out
        .path()
        .join("src")
        .join("packages")
        .join("acme.example.v1.md");
    assert!(package_md.is_file(), "expected {}", package_md.display());

    let pkg = std::fs::read_to_string(&package_md).expect("read package md");
    assert!(pkg.contains("EchoUnaryRequest"));
    assert!(pkg.contains("```protobuf"));
    assert!(pkg.contains("message EchoUnaryRequest"));
    assert!(!pkg.contains("| # | Name | Type |"));

    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("links resolve");
}

#[test]
fn protoc_init_writes_mdbook_tree() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures");

    let out = tempfile::tempdir().expect("tempdir");
    let out_arg = format!("--mdbook_out={}", out.path().display());
    let plugin_arg = format!("--plugin=protoc-gen-mdbook={}", plugin.display());

    let status = Command::new(protoc)
        .args([
            "-I",
            fixture_dir.to_str().expect("utf8"),
            &plugin_arg,
            &out_arg,
            "--mdbook_opt=init",
            "doc_rich.proto",
        ])
        .status()
        .expect("spawn protoc");

    assert!(status.success(), "protoc exit: {status:?}");

    let book_toml = out.path().join("book.toml");
    let readme = out.path().join("README.md");
    let summary = out.path().join("src").join("SUMMARY.md");
    let package_md = out
        .path()
        .join("src")
        .join("packages")
        .join("acme.example.v1.md");
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

    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("links resolve");
}

#[test]
fn version_flag_prints_mdbook_pin() {
    let bin = env!("CARGO_BIN_EXE_protoc-gen-mdbook");
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .expect("spawn plugin");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("protoc-gen-mdbook"));
    assert!(
        stdout.contains(protoc_gen_mdbook::mdbook_version()),
        "stdout should include pinned mdbook version: {stdout}"
    );
}
