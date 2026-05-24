//! `escape_tags` option: leading-comment prose escaping (protoc plugin and CLI).

mod common;

use common::{mirrored_fixture_backends, run_fixture_proto_in};
use std::fs;

const PROTO: &str = "escape_tags_comments.proto";
const PACKAGE_MD: &str = "src/packages/acme.example.tagdoc.md";

#[test]
fn escape_tags_backticks_in_prose() {
    for backend in mirrored_fixture_backends() {
        let out = tempfile::tempdir().expect("tempdir");
        run_fixture_proto_in(out.path(), PROTO, "layout=package,escape_tags", backend);
        let pkg = fs::read_to_string(out.path().join(PACKAGE_MD)).expect("package md");
        assert!(
            pkg.contains("`<environment>`"),
            "backticks in prose ({backend:?})"
        );
        assert!(
            pkg.contains("`<zMist>`") && pkg.contains("`<zMistsMap>`"),
            "nested tags ({backend:?})"
        );
        assert!(
            pkg.contains("// Field note from <sensor>"),
            "fence comment stays bare ({backend:?})"
        );
        let before_fence = pkg.split("```protobuf").next().expect("prose");
        assert!(
            before_fence.contains("from `<environment>`"),
            "backtick-wrapped environment ({backend:?})"
        );
        assert!(
            !before_fence.contains("from <environment>"),
            "no bare environment in prose ({backend:?})"
        );
        protobuf_mdbook::link_check::assert_tree(out.path()).expect("links");
    }
}

#[test]
fn escape_tags_entities_in_prose() {
    for backend in mirrored_fixture_backends() {
        let out = tempfile::tempdir().expect("tempdir");
        run_fixture_proto_in(
            out.path(),
            PROTO,
            "layout=package,escape_tags=entities",
            backend,
        );
        let pkg = fs::read_to_string(out.path().join(PACKAGE_MD)).expect("package md");
        assert!(
            pkg.contains("&lt;environment&gt;"),
            "entities in prose ({backend:?})"
        );
        assert!(
            pkg.contains("// Field note from <sensor>"),
            "fence comment stays bare ({backend:?})"
        );
        protobuf_mdbook::link_check::assert_tree(out.path()).expect("links");
    }
}
