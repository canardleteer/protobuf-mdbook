//! Directory tree for SUMMARY with minimal subchaptering.

use crate::options::Layout;
use crate::plugin_api::FileDescriptorProto;
use crate::proto_markdown::{CompanionDoc, module_path_from_companion_output};
use crate::render::links::LinkContext;
use crate::summary::render_md::{SUMMARY_MAX_DEPTH, link_path_for_summary};
use mdbook_summary::{Link, Summary, SummaryItem};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct NavInput<'a> {
    pub companions: &'a [CompanionDoc],
    pub packages: BTreeMap<String, PackageAtDir<'a>>,
    pub summary_from: &'a Path,
    pub links: &'a LinkContext,
}

pub struct PackageAtDir<'a> {
    pub rel_dir: PathBuf,
    pub package: &'a str,
    pub files: &'a [(&'a str, &'a FileDescriptorProto)],
}

#[derive(Default)]
struct DirNode {
    companions: Vec<LinkEntry>,
    packages: Vec<LinkEntry>,
    children: BTreeMap<String, DirNode>,
}

#[derive(Clone)]
struct LinkEntry {
    title: String,
    path: PathBuf,
    stem: String,
    /// Protobuf-style path (`acme.example.v1`) for companions; package name for generated pages.
    module_path: Option<String>,
    is_package: bool,
    entity_items: Vec<SummaryItem>,
}

/// Build companion + package nav tree; entity sub-links attach at package link creation when requested.
pub fn build_summary(
    h1_title: &str,
    input: NavInput<'_>,
    layout: Layout,
    package_only: bool,
) -> Summary {
    let attach_entities = !package_only && !matches!(layout, Layout::Package);
    let mut root = DirNode::default();
    for doc in input.companions {
        let target = PathBuf::from(&input.links.markdown_root).join(&doc.output_rel);
        let path = link_path_for_summary(input.summary_from, &target);
        insert_at(
            &mut root,
            path_segments(&doc.source_dir),
            LinkEntry {
                title: doc.title.clone(),
                path,
                stem: doc.stem.clone(),
                module_path: module_path_from_companion_output(&doc.output_rel, &doc.stem),
                is_package: false,
                entity_items: Vec::new(),
            },
            InsertKind::Companion,
        );
    }
    for info in input.packages.values() {
        let target = package_target(input.links, layout, info.package);
        let path = link_path_for_summary(input.summary_from, &target);
        let entity_items = if attach_entities {
            super::entity_summary_items(info.package, info.files, input.links, input.summary_from)
        } else {
            Vec::new()
        };
        insert_at(
            &mut root,
            path_segments(&info.rel_dir),
            LinkEntry {
                title: info.package.to_string(),
                path,
                stem: String::new(),
                module_path: Some(info.package.to_string()),
                is_package: true,
                entity_items,
            },
            InsertKind::Package,
        );
    }

    let collapsed = collapse_linear_chains(root);
    let chapters = emit_dir(&collapsed, 0);

    let mut summary = Summary::default();
    summary.title = Some(h1_title.to_string());
    summary.numbered_chapters = chapters;
    summary
}

enum InsertKind {
    Companion,
    Package,
}

/// Normal path components for companion/package directory keys.
fn path_segments(rel_dir: &Path) -> Vec<String> {
    rel_dir
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn insert_at(node: &mut DirNode, segments: Vec<String>, entry: LinkEntry, kind: InsertKind) {
    if segments.is_empty() {
        match kind {
            InsertKind::Companion => node.companions.push(entry),
            InsertKind::Package => node.packages.push(entry),
        }
        return;
    }
    let head = segments[0].clone();
    let tail = segments[1..].to_vec();
    insert_at(node.children.entry(head).or_default(), tail, entry, kind);
}

fn package_target(links: &LinkContext, layout: Layout, package: &str) -> PathBuf {
    match layout {
        Layout::Package => links.package_page_rel(package),
        Layout::Entity | Layout::Split => links.package_index_rel(package),
    }
}

/// Collapse nodes with no companions, no packages, and exactly one child.
fn collapse_linear_chains(mut node: DirNode) -> DirNode {
    loop {
        if !node.companions.is_empty() || !node.packages.is_empty() || node.children.len() != 1 {
            break;
        }
        let (_key, child) = node.children.into_iter().next().expect("one child");
        node = child;
    }
    let children = node
        .children
        .into_iter()
        .map(|(k, c)| (k, collapse_linear_chains(c)))
        .collect();
    DirNode {
        companions: node.companions,
        packages: node.packages,
        children,
    }
}

fn emit_dir(node: &DirNode, depth: usize) -> Vec<SummaryItem> {
    if depth >= SUMMARY_MAX_DEPTH {
        return emit_flat(node);
    }

    let mut out = Vec::new();

    if node.companions.is_empty() && node.packages.is_empty() {
        for child in node.children.values() {
            out.extend(emit_dir(child, depth));
        }
        return out;
    }

    let mut group: Vec<LinkEntry> = node.companions.clone();
    sort_companions(&mut group);
    group.extend(node.packages.clone());

    if group.len() == 1 {
        out.push(link_item(&summary_link_entry(&group[0], false)));
    } else if !group.is_empty() {
        let first = &group[0];
        let mut link = link_item(&summary_link_entry(first, false));
        if let SummaryItem::Link(ref mut parent) = link {
            for entry in &group[1..] {
                parent
                    .nested_items
                    .push(link_item(&summary_link_entry(entry, true)));
            }
        }
        out.push(link);
    }

    let child_depth = depth.saturating_add(1);
    for child in node.children.values() {
        out.extend(emit_dir(child, child_depth));
    }

    out
}

fn emit_flat(node: &DirNode) -> Vec<SummaryItem> {
    let mut items = Vec::new();
    for c in &node.companions {
        items.push(link_item(&summary_link_entry(c, false)));
    }
    for child in node.children.values() {
        items.extend(emit_flat(child));
    }
    for p in &node.packages {
        items.push(link_item(&summary_link_entry(p, false)));
    }
    items
}

fn summary_link_entry(entry: &LinkEntry, is_nested: bool) -> LinkEntry {
    LinkEntry {
        title: format_summary_title(entry, is_nested),
        path: entry.path.clone(),
        stem: entry.stem.clone(),
        module_path: entry.module_path.clone(),
        is_package: entry.is_package,
        entity_items: entry.entity_items.clone(),
    }
}

/// SUMMARY link titles: module path prefix for section companions, bare titles at corpus root and for nested subchapters, package name only for generated module pages.
fn format_summary_title(entry: &LinkEntry, is_nested: bool) -> String {
    if is_nested || entry.is_package {
        return entry.title.clone();
    }
    let Some(ref module_path) = entry.module_path else {
        return entry.title.clone();
    };
    // Top-level companion dirs (`acme/`) — no protobuf package segment.
    if !module_path.contains('.') {
        return entry.title.clone();
    }
    if entry.title == *module_path {
        return entry.title.clone();
    }
    format!("{module_path} - {}", entry.title)
}

/// README first, then other companion pages alphabetically by filename stem.
fn sort_companions(companions: &mut [LinkEntry]) {
    companions.sort_by(|a, b| {
        companion_stem_rank(&a.stem)
            .cmp(&companion_stem_rank(&b.stem))
            .then_with(|| a.stem.cmp(&b.stem))
    });
}

fn companion_stem_rank(stem: &str) -> u8 {
    if stem.eq_ignore_ascii_case("readme") {
        0
    } else {
        1
    }
}

/// Emit one SUMMARY link, including entity sub-links on package entries.
fn link_item(entry: &LinkEntry) -> SummaryItem {
    let mut link = Link::new(entry.title.clone(), &entry.path);
    link.nested_items = entry.entity_items.clone();
    SummaryItem::Link(link)
}

/// Parent directory of a proto file, used for nav-tree placement.
pub fn package_rel_dir(proto_name: &str) -> PathBuf {
    Path::new(proto_name)
        .parent()
        .unwrap_or(Path::new(""))
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{Layout, Options};
    use crate::proto_markdown::CompanionDoc;
    use crate::render::build_link_context;
    use std::collections::BTreeMap;

    #[test]
    fn format_summary_title_rules() {
        let section = |title: &str, module_path: &str| LinkEntry {
            title: title.into(),
            path: PathBuf::from("packages/x.md"),
            stem: "README".into(),
            module_path: Some(module_path.into()),
            is_package: false,
            entity_items: Vec::new(),
        };
        let package = |pkg: &str| LinkEntry {
            title: pkg.into(),
            path: PathBuf::from("packages/x.md"),
            stem: String::new(),
            module_path: Some(pkg.into()),
            is_package: true,
            entity_items: Vec::new(),
        };

        assert_eq!(
            format_summary_title(&section("Acme APIs", "acme"), false),
            "Acme APIs"
        );
        assert_eq!(
            format_summary_title(&section("Example services", "acme.example"), false),
            "acme.example - Example services"
        );
        assert_eq!(
            format_summary_title(&section("acme.example.v1 README", "acme.example.v1"), false),
            "acme.example.v1 - acme.example.v1 README"
        );
        assert_eq!(
            format_summary_title(&section("Moving to v2", "acme.example.v1"), true),
            "Moving to v2"
        );
        assert_eq!(
            format_summary_title(&package("acme.example.v1"), true),
            "acme.example.v1"
        );
        assert_eq!(
            format_summary_title(&package("acme.example.v2"), false),
            "acme.example.v2"
        );
    }

    #[test]
    fn asymmetric_sparse_tree_summary_shape() {
        let companions = vec![CompanionDoc {
            output_rel: "a.b.NOTES.md".into(),
            title: "Notes".into(),
            source_dir: PathBuf::from("a/b"),
            stem: "NOTES".into(),
            source_path: PathBuf::from("/unused"),
        }];
        // `a.b` has a dot → section companion is prefixed in SUMMARY.
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        let f1 = FileDescriptorProto {
            name: Some("a/b/c/d/v2/e.proto".into()),
            package: Some("shallow.v2".into()),
            ..Default::default()
        };
        let f2 = FileDescriptorProto {
            name: Some("a/b/c/d/e/f/g/h/v1/stuff.proto".into()),
            package: Some("deep.v1".into()),
            ..Default::default()
        };
        let f3 = FileDescriptorProto {
            name: Some("a/b/c/d/e/f/g/h/v2/stuff.proto".into()),
            package: Some("deep.v2".into()),
            ..Default::default()
        };
        by_package.insert("shallow.v2".into(), vec![("a/b/c/d/v2/e.proto", &f1)]);
        by_package.insert(
            "deep.v1".into(),
            vec![("a/b/c/d/e/f/g/h/v1/stuff.proto", &f2)],
        );
        by_package.insert(
            "deep.v2".into(),
            vec![("a/b/c/d/e/f/g/h/v2/stuff.proto", &f3)],
        );
        let opts = Options::default();
        let links = build_link_context(&by_package, &opts);
        let packages = BTreeMap::from([
            (
                "shallow.v2".into(),
                PackageAtDir {
                    rel_dir: package_rel_dir("a/b/c/d/v2/e.proto"),
                    package: "shallow.v2",
                    files: by_package["shallow.v2"].as_slice(),
                },
            ),
            (
                "deep.v1".into(),
                PackageAtDir {
                    rel_dir: package_rel_dir("a/b/c/d/e/f/g/h/v1/stuff.proto"),
                    package: "deep.v1",
                    files: by_package["deep.v1"].as_slice(),
                },
            ),
            (
                "deep.v2".into(),
                PackageAtDir {
                    rel_dir: package_rel_dir("a/b/c/d/e/f/g/h/v2/stuff.proto"),
                    package: "deep.v2",
                    files: by_package["deep.v2"].as_slice(),
                },
            ),
        ]);
        let summary = build_summary(
            "Protobuf documentation",
            NavInput {
                companions: &companions,
                packages,
                summary_from: Path::new("src/SUMMARY.md"),
                links: &links,
            },
            Layout::Package,
            true,
        );
        let md = crate::summary::render_md::render_summary_markdown(&summary);
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("Notes"));
        assert!(md.contains("shallow.v2"));
        assert!(md.contains("deep.v1"));
        assert!(md.contains("deep.v2"));
    }
}
