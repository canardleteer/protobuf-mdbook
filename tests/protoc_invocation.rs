//! Integration tests: `protoc-gen-mdbook` plugin and mirrored `protobuf-mdbook` CLI.

mod common;

use common::{
    assert_fixture_init, assert_fixture_markdown_only, mirrored_fixture_backends, run_fixture,
};
use std::process::Command;

#[test]
fn default_emits_markdown_only() {
    for backend in mirrored_fixture_backends() {
        let out = run_fixture("", backend);
        assert_fixture_markdown_only(out.path());
    }
}

#[test]
fn init_writes_mdbook_tree() {
    for backend in mirrored_fixture_backends() {
        let out = run_fixture("init", backend);
        assert_fixture_init(out.path());
    }
}

#[test]
fn protoc_plugin_version_flag_prints_mdbook_pin() {
    let bin = env!("CARGO_BIN_EXE_protoc-gen-mdbook");
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .expect("spawn plugin");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("protoc-gen-mdbook"));
    assert!(
        stdout.contains(protobuf_mdbook::mdbook_version()),
        "stdout should include pinned mdbook version: {stdout}"
    );
}

#[test]
fn protobuf_mdbook_version_flag_prints_mdbook_pin() {
    let bin = env!("CARGO_BIN_EXE_protobuf-mdbook");
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .expect("spawn protobuf-mdbook");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("protobuf-mdbook"));
    assert!(
        stdout.contains(protobuf_mdbook::mdbook_version()),
        "stdout should include pinned mdbook version: {stdout}"
    );
}

#[test]
fn mdbook_protobuf_highlight_version_flag_prints_mdbook_pin() {
    let bin = env!("CARGO_BIN_EXE_mdbook-protobuf-highlight");
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .expect("spawn mdbook-protobuf-highlight");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("mdbook-protobuf-highlight"));
    assert!(
        stdout.contains(protobuf_mdbook::mdbook_version()),
        "stdout should include pinned mdbook version: {stdout}"
    );
}
