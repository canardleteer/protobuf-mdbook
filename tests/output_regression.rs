//! Byte-identical output regression against checked-in golden fixtures.
//!
//! Refresh baselines: `cargo xtask update-golden`
//!
//! Generation uses [`Backend::ProtocPlugin`] only; CLI parity is covered by
//! mirrored integration tests in `link_check.rs` and `escape_tags.rs`.

mod common;

use common::{Backend, run_examples_in, run_fixture_proto_in};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

struct Scenario {
    name: &'static str,
    layout: &'static str,
    extra_opt: &'static str,
    /// When set, generate from this fixture proto instead of `examples/proto/`.
    fixture_proto: Option<&'static str>,
    /// When set, only these paths are compared (relative to output root).
    paths_only: Option<&'static [&'static str]>,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "package_default",
        layout: "package",
        extra_opt: "",
        fixture_proto: None,
        paths_only: None,
    },
    Scenario {
        name: "entity",
        layout: "entity",
        extra_opt: "",
        fixture_proto: None,
        paths_only: None,
    },
    Scenario {
        name: "split",
        layout: "split",
        extra_opt: "",
        fixture_proto: None,
        paths_only: None,
    },
    Scenario {
        name: "summary_companion",
        layout: "package",
        extra_opt: "summary",
        fixture_proto: None,
        paths_only: Some(&["src/SUMMARY.md", "src/packages/acme.example.v1.md"]),
    },
    Scenario {
        name: "summary_entity_flat",
        layout: "entity",
        extra_opt: "summary,no_proto_markdown",
        fixture_proto: None,
        paths_only: Some(&["src/SUMMARY.md"]),
    },
    Scenario {
        name: "summary_split_flat",
        layout: "split",
        extra_opt: "summary,no_proto_markdown",
        fixture_proto: None,
        paths_only: Some(&["src/SUMMARY.md"]),
    },
    Scenario {
        name: "escape_tags_backticks",
        layout: "",
        extra_opt: "layout=package,escape_tags",
        fixture_proto: Some("escape_tags_comments.proto"),
        paths_only: Some(&["src/packages/acme.example.tagdoc.md"]),
    },
    Scenario {
        name: "escape_tags_entities",
        layout: "",
        extra_opt: "layout=package,escape_tags=entities",
        fixture_proto: Some("escape_tags_comments.proto"),
        paths_only: Some(&["src/packages/acme.example.tagdoc.md"]),
    },
];

fn golden_dir() -> PathBuf {
    common::manifest_dir().join("tests/fixtures/golden")
}

fn normalize_newlines(content: String) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Collect relative path → content for all files under `root`.
fn collect_tree(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    collect_tree_inner(root, root, &mut out);
    out
}

fn collect_tree_inner(root: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            collect_tree_inner(root, &path, out);
        } else {
            let rel = path
                .strip_prefix(root)
                .expect("strip_prefix")
                .to_string_lossy()
                .replace('\\', "/");
            let content = normalize_newlines(fs::read_to_string(&path).expect("read file"));
            out.insert(rel, content);
        }
    }
}

fn generate_output(scenario: &Scenario) -> BTreeMap<String, String> {
    let out = tempfile::tempdir().expect("tempdir");
    if let Some(proto) = scenario.fixture_proto {
        run_fixture_proto_in(out.path(), proto, scenario.extra_opt, Backend::ProtocPlugin);
    } else {
        run_examples_in(
            out.path(),
            scenario.layout,
            scenario.extra_opt,
            Backend::ProtocPlugin,
        );
    }
    let tree = collect_tree(out.path());
    match scenario.paths_only {
        None => tree,
        Some(paths) => paths
            .iter()
            .map(|p| {
                let content = tree
                    .get(*p)
                    .unwrap_or_else(|| panic!("missing output path {p} in {}", scenario.name))
                    .clone();
                ((*p).to_string(), content)
            })
            .collect(),
    }
}

fn write_golden(scenario: &Scenario, tree: &BTreeMap<String, String>) {
    let dir = golden_dir().join(scenario.name);
    fs::create_dir_all(&dir).expect("create golden dir");
    for (rel, content) in tree {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, content).expect("write golden");
    }
}

fn read_golden(scenario: &Scenario) -> BTreeMap<String, String> {
    let dir = golden_dir().join(scenario.name);
    if !dir.is_dir() {
        panic!("missing golden dir: {}", dir.display());
    }
    collect_tree(&dir)
}

#[test]
fn output_regression_matches_golden() {
    let update = std::env::var("UPDATE_GOLDEN").ok().as_deref() == Some("1");

    for scenario in SCENARIOS {
        let actual = generate_output(scenario);
        if update {
            write_golden(scenario, &actual);
            eprintln!("updated golden: {}", scenario.name);
            continue;
        }
        let expected = read_golden(scenario);
        assert_eq!(expected, actual, "golden mismatch for {}", scenario.name);
    }
}
