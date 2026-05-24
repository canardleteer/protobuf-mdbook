//! `SUMMARY.md` navigation for generated documentation.

mod nav_tree;
mod render_md;

use crate::init::DEFAULT_BOOK_TITLE;
use crate::options::{Layout, Options};
use crate::plugin_api::FileDescriptorProto;
use crate::proto_markdown::CompanionDoc;
use crate::render::for_each_entity_in_files;
use crate::render::links::{EntityKind, LinkContext};
use nav_tree::{NavInput, PackageAtDir, build_summary, package_rel_dir};
use render_md::{render_summary_markdown, validate_summary_warn};
use std::collections::BTreeMap;
use std::path::Path;

/// Build SUMMARY navigation when `summary` or `init` is set.
pub fn render_summary(
    by_package: &BTreeMap<String, Vec<(&str, &FileDescriptorProto)>>,
    opts: &Options,
    links: &LinkContext,
    companions: &[CompanionDoc],
) -> Option<(String, String)> {
    if !opts.render_summary() {
        return None;
    }

    let package_only = opts.package_only_summary();
    let h1 = summary_h1_title(opts);
    let summary_from = Path::new(&opts.summary_path);

    let summary = if opts.no_proto_markdown {
        build_flat_summary(
            by_package,
            &h1,
            opts.layout,
            package_only,
            links,
            summary_from,
        )
    } else {
        let packages = packages_nav_input(by_package);
        build_summary(
            &h1,
            NavInput {
                companions,
                packages,
                summary_from,
                links,
            },
            opts.layout,
            package_only,
        )
    };

    let md = render_summary_markdown(&summary);
    validate_summary_warn(&md);
    Some((opts.output_path(&opts.summary_path), md))
}

/// H1 title for generated SUMMARY.md.
fn summary_h1_title(opts: &Options) -> String {
    if opts.init {
        opts.title
            .clone()
            .unwrap_or_else(|| DEFAULT_BOOK_TITLE.to_string())
    } else {
        DEFAULT_BOOK_TITLE.to_string()
    }
}

/// Map packages to filesystem-relative dirs for nav-tree insertion (first proto path wins per package).
fn packages_nav_input<'a>(
    by_package: &'a BTreeMap<String, Vec<(&'a str, &'a FileDescriptorProto)>>,
) -> BTreeMap<String, PackageAtDir<'a>> {
    let mut out = BTreeMap::new();
    for (package, files) in by_package {
        if package.is_empty() {
            continue;
        }
        let rel_dir = files
            .first()
            .map(|(name, _)| package_rel_dir(name))
            .unwrap_or_default();
        out.insert(
            package.clone(),
            PackageAtDir {
                rel_dir,
                package,
                files,
            },
        );
    }
    out
}

/// Flat SUMMARY chapters when companion markdown is disabled (`no_proto_markdown`).
fn build_flat_summary(
    by_package: &BTreeMap<String, Vec<(&str, &FileDescriptorProto)>>,
    h1: &str,
    layout: Layout,
    package_only: bool,
    links: &LinkContext,
    summary_from: &Path,
) -> mdbook_summary::Summary {
    let chapters =
        build_flat_summary_chapters(by_package, layout, package_only, links, summary_from);
    let mut summary = mdbook_summary::Summary::default();
    summary.title = Some(h1.to_string());
    summary.numbered_chapters = chapters;
    summary
}

/// One top-level chapter per package for flat SUMMARY layouts.
fn build_flat_summary_chapters(
    by_package: &BTreeMap<String, Vec<(&str, &FileDescriptorProto)>>,
    layout: Layout,
    package_only: bool,
    links: &LinkContext,
    summary_from: &Path,
) -> Vec<mdbook_summary::SummaryItem> {
    let mut chapters = Vec::new();
    for (package, files) in by_package {
        match layout {
            Layout::Package => {
                let target = links.package_page_rel(package);
                let path = render_md::link_path_for_summary(summary_from, &target);
                chapters.push(mdbook_summary::SummaryItem::Link(
                    mdbook_summary::Link::new(package, path),
                ));
            }
            Layout::Split | Layout::Entity if package_only => {
                let target = links.package_index_rel(package);
                let path = render_md::link_path_for_summary(summary_from, &target);
                chapters.push(mdbook_summary::SummaryItem::Link(
                    mdbook_summary::Link::new(package, path),
                ));
            }
            Layout::Split => {
                let target = links.package_index_rel(package);
                let path = render_md::link_path_for_summary(summary_from, &target);
                let mut link = mdbook_summary::Link::new(package, path);
                link.nested_items = entity_summary_items(package, files, links, summary_from);
                chapters.push(mdbook_summary::SummaryItem::Link(link));
            }
            Layout::Entity => {
                let mut link = mdbook_summary::Link::default();
                link.name = package.to_string();
                link.location = None;
                link.nested_items = entity_summary_items(package, files, links, summary_from);
                chapters.push(mdbook_summary::SummaryItem::Link(link));
            }
        }
    }
    chapters
}

/// Entity sub-links for SUMMARY in link/SUMMARY order.
pub(crate) fn entity_summary_items(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
    links: &LinkContext,
    summary_from: &Path,
) -> Vec<mdbook_summary::SummaryItem> {
    let mut out = Vec::new();
    for_each_entity_in_files(files, |kind, name, _, _, _| {
        let p = links.entity_path(package, kind, name).expect("entity");
        let path = render_md::link_path_for_summary(summary_from, p);
        let title = match kind {
            EntityKind::Message => format!("Message {name}"),
            EntityKind::Enum => format!("Enum {name}"),
            EntityKind::Service => format!("Service {name}"),
        };
        out.push(mdbook_summary::SummaryItem::Link(
            mdbook_summary::Link::new(title, path),
        ));
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Layout;
    use crate::options::Options;
    use crate::proto_markdown::{CompanionDoc, companion_output_name};
    use crate::render::build_link_context;
    use std::path::PathBuf;

    fn companion(rel_dir: &str, stem: &str, title: &str) -> CompanionDoc {
        let source_dir = PathBuf::from(rel_dir);
        CompanionDoc {
            output_rel: companion_output_name(&source_dir, stem),
            title: title.to_string(),
            source_dir,
            stem: stem.to_string(),
            source_path: PathBuf::from("/unused"),
        }
    }

    #[test]
    fn summary_parses_with_companion_tree() {
        let companions = vec![
            companion("acme", "README", "Acme APIs"),
            companion("acme/example", "README", "Example services"),
            companion("acme/example/v1", "README", "acme.example.v1 README"),
            companion("acme/example/v1", "MOVING-TO-V2", "Moving to v2"),
        ];
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        let file = dummy_file("acme.example.v1");
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let opts = Options {
            summary: true,
            init: true,
            ..Options::default()
        };
        let links = build_link_context(&by_package, &opts);
        let packages = packages_nav_input(&by_package);
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
        let md = render_summary_markdown(&summary);
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("[Acme APIs]"));
        assert!(md.contains("[acme.example - Example services]"));
        assert!(md.contains("[acme.example.v1 - acme.example.v1 README]"));
        assert!(md.contains("[Moving to v2]"));
        assert!(!md.contains("example/v1"));
        assert!(!md.contains("example —"));
        assert!(md.contains("[acme.example.v1](packages/acme.example.v1.md)"));
    }

    #[test]
    fn render_summary_entity_layout_lists_entities() {
        let file = rich_file("acme.example.v1");
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let opts = Options {
            summary: true,
            layout: Layout::Entity,
            no_proto_markdown: true,
            ..Options::default()
        };
        let links = build_link_context(&by_package, &opts);
        let out = render_summary(&by_package, &opts, &links, &[]).expect("summary");
        let md = out.1;
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("Message EchoUnaryRequest"));
        assert!(md.contains("Service EchoService"));
    }

    #[test]
    fn render_summary_split_layout_nests_entities() {
        let file = rich_file("acme.example.v1");
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let opts = Options {
            summary: true,
            layout: Layout::Split,
            no_proto_markdown: true,
            ..Options::default()
        };
        let links = build_link_context(&by_package, &opts);
        let out = render_summary(&by_package, &opts, &links, &[]).expect("summary");
        let md = out.1;
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("[acme.example.v1]"));
        assert!(md.contains("Message EchoUnaryRequest"));
    }

    #[test]
    fn render_summary_with_companions_uses_nav_tree() {
        let companions = vec![companion("acme/example/v1", "README", "v1 README")];
        let file = dummy_file("acme.example.v1");
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let opts = Options {
            summary: true,
            ..Options::default()
        };
        let links = build_link_context(&by_package, &opts);
        let out = render_summary(&by_package, &opts, &links, &companions).expect("summary");
        let md = out.1;
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("[acme.example.v1 - v1 README]"));
        assert!(md.contains("[acme.example.v1](packages/acme.example.v1.md)"));
    }

    fn rich_file(pkg: &str) -> FileDescriptorProto {
        use buffa_descriptor::generated::descriptor::{DescriptorProto, ServiceDescriptorProto};
        FileDescriptorProto {
            name: Some("acme/example/v1/a.proto".into()),
            package: Some(pkg.into()),
            message_type: vec![DescriptorProto {
                name: Some("EchoUnaryRequest".into()),
                ..Default::default()
            }],
            service: vec![ServiceDescriptorProto {
                name: Some("EchoService".into()),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn dummy_file(pkg: &str) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some("acme/example/v1/a.proto".into()),
            package: Some(pkg.into()),
            ..Default::default()
        }
    }
}
