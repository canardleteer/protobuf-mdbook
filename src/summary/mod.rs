//! `SUMMARY.md` navigation for generated documentation.

mod nav_tree;
mod render_md;

use crate::init::DEFAULT_BOOK_TITLE;
use crate::options::{Layout, Options};
use crate::plugin_api::FileDescriptorProto;
use crate::proto_markdown::CompanionDoc;
use crate::render::links::{EntityKind, LinkContext};
use nav_tree::{NavInput, PackageAtDir, build_summary, package_rel_dir};
use render_md::{render_summary_markdown, validate_summary_warn};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Build SUMMARY navigation when `summary` or `init` is set.
pub fn render_summary(
    proto_files: &[FileDescriptorProto],
    file_to_generate: &[String],
    opts: &Options,
    links: &LinkContext,
    companions: &[CompanionDoc],
) -> Option<(String, String)> {
    if !opts.render_summary() {
        return None;
    }
    if opts.no_proto_markdown {
        return render_summary_proto_only(proto_files, file_to_generate, opts, links);
    }

    let package_only = opts.package_only_summary();
    let h1 = summary_h1_title(opts);

    let mut by_package: BTreeMap<String, Vec<&FileDescriptorProto>> = BTreeMap::new();
    let mut package_dirs: BTreeMap<String, PathBuf> = BTreeMap::new();
    for name in file_to_generate {
        let Some(file) = proto_files
            .iter()
            .find(|f| f.name.as_deref() == Some(name.as_str()))
        else {
            continue;
        };
        let pkg = file.package.clone().unwrap_or_default();
        if pkg.is_empty() {
            continue;
        }
        by_package.entry(pkg.clone()).or_default().push(file);
        package_dirs
            .entry(pkg)
            .or_insert_with(|| package_rel_dir(name));
    }

    let mut packages = BTreeMap::new();
    for (package, files) in &by_package {
        let rel_dir = package_dirs.get(package).cloned().unwrap_or_default();
        packages.insert(
            package.clone(),
            PackageAtDir {
                rel_dir,
                package,
                files,
            },
        );
    }

    let summary_from = Path::new(&opts.summary_path);
    let mut summary = build_summary(
        &h1,
        NavInput {
            companions,
            packages,
            summary_from,
            links,
        },
        opts.layout,
        package_only,
    );

    if !package_only {
        append_entity_lines(
            &mut summary.numbered_chapters,
            &by_package,
            opts.layout,
            links,
            summary_from,
        );
    }

    let md = render_summary_markdown(&summary);
    validate_summary_warn(&md);
    Some((opts.output_path(&opts.summary_path), md))
}

fn summary_h1_title(opts: &Options) -> String {
    if opts.init {
        opts.title
            .clone()
            .unwrap_or_else(|| DEFAULT_BOOK_TITLE.to_string())
    } else {
        DEFAULT_BOOK_TITLE.to_string()
    }
}

fn render_summary_proto_only(
    proto_files: &[FileDescriptorProto],
    file_to_generate: &[String],
    opts: &Options,
    links: &LinkContext,
) -> Option<(String, String)> {
    if !opts.render_summary() {
        return None;
    }

    let package_only = opts.package_only_summary();
    let h1 = summary_h1_title(opts);

    let mut by_package: BTreeMap<String, Vec<&FileDescriptorProto>> = BTreeMap::new();
    for name in file_to_generate {
        let Some(file) = proto_files
            .iter()
            .find(|f| f.name.as_deref() == Some(name.as_str()))
        else {
            continue;
        };
        let pkg = file.package.clone().unwrap_or_default();
        if !pkg.is_empty() {
            by_package.entry(pkg).or_default().push(file);
        }
    }

    let summary_from = Path::new(&opts.summary_path);
    let mut chapters = Vec::new();
    for (package, files) in &by_package {
        match opts.layout {
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
                push_entity_items(&mut link.nested_items, package, files, links, summary_from);
                chapters.push(mdbook_summary::SummaryItem::Link(link));
            }
            Layout::Entity => {
                let mut link = mdbook_summary::Link::default();
                link.name = package.to_string();
                link.location = None;
                push_entity_items(&mut link.nested_items, package, files, links, summary_from);
                chapters.push(mdbook_summary::SummaryItem::Link(link));
            }
        }
    }

    let mut summary = mdbook_summary::Summary::default();
    summary.title = Some(h1);
    summary.numbered_chapters = chapters;
    let md = render_summary_markdown(&summary);
    validate_summary_warn(&md);
    Some((opts.output_path(&opts.summary_path), md))
}

fn append_entity_lines(
    chapters: &mut [mdbook_summary::SummaryItem],
    by_package: &BTreeMap<String, Vec<&FileDescriptorProto>>,
    layout: Layout,
    links: &LinkContext,
    summary_from: &Path,
) {
    if matches!(layout, Layout::Package) {
        return;
    }
    for (package, files) in by_package {
        let target = links.package_index_rel(package);
        let path = render_md::link_path_for_summary(summary_from, &target);
        if let Some(mdbook_summary::SummaryItem::Link(link)) = chapters.iter_mut().find(|item| {
            if let mdbook_summary::SummaryItem::Link(l) = item {
                l.location.as_deref() == Some(path.as_path())
            } else {
                false
            }
        }) {
            push_entity_items(&mut link.nested_items, package, files, links, summary_from);
        }
    }
}

fn push_entity_items(
    out: &mut Vec<mdbook_summary::SummaryItem>,
    package: &str,
    files: &[&FileDescriptorProto],
    links: &LinkContext,
    summary_from: &Path,
) {
    for file in files {
        for msg in &file.message_type {
            if let Some(name) = &msg.name {
                let p = links
                    .entity_path(package, EntityKind::Message, name)
                    .expect("entity");
                let path = render_md::link_path_for_summary(summary_from, p);
                out.push(mdbook_summary::SummaryItem::Link(
                    mdbook_summary::Link::new(format!("Message {name}"), path),
                ));
            }
        }
        for en in &file.enum_type {
            if let Some(name) = &en.name {
                let p = links
                    .entity_path(package, EntityKind::Enum, name)
                    .expect("entity");
                let path = render_md::link_path_for_summary(summary_from, p);
                out.push(mdbook_summary::SummaryItem::Link(
                    mdbook_summary::Link::new(format!("Enum {name}"), path),
                ));
            }
        }
        for svc in &file.service {
            if let Some(name) = &svc.name {
                let p = links
                    .entity_path(package, EntityKind::Service, name)
                    .expect("entity");
                let path = render_md::link_path_for_summary(summary_from, p);
                out.push(mdbook_summary::SummaryItem::Link(
                    mdbook_summary::Link::new(format!("Service {name}"), path),
                ));
            }
        }
    }
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
        let empty: &[&FileDescriptorProto] = &[];
        let packages = BTreeMap::from([(
            "acme.example.v1".to_string(),
            PackageAtDir {
                rel_dir: PathBuf::from("acme/example/v1"),
                package: "acme.example.v1",
                files: empty,
            },
        )]);
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
        let proto_files = vec![file.clone()];
        let file_to_generate = vec!["acme/example/v1/a.proto".into()];
        let opts = Options {
            summary: true,
            layout: Layout::Entity,
            no_proto_markdown: true,
            ..Options::default()
        };
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let links = build_link_context(&by_package, &opts);
        let out =
            render_summary(&proto_files, &file_to_generate, &opts, &links, &[]).expect("summary");
        let md = out.1;
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("Message EchoUnaryRequest"));
        assert!(md.contains("Service EchoService"));
    }

    #[test]
    fn render_summary_split_layout_nests_entities() {
        let file = rich_file("acme.example.v1");
        let proto_files = vec![file.clone()];
        let file_to_generate = vec!["acme/example/v1/a.proto".into()];
        let opts = Options {
            summary: true,
            layout: Layout::Split,
            no_proto_markdown: true,
            ..Options::default()
        };
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let links = build_link_context(&by_package, &opts);
        let out =
            render_summary(&proto_files, &file_to_generate, &opts, &links, &[]).expect("summary");
        let md = out.1;
        mdbook_summary::parse_summary(&md).expect("valid SUMMARY");
        assert!(md.contains("[acme.example.v1]"));
        assert!(md.contains("Message EchoUnaryRequest"));
    }

    #[test]
    fn render_summary_with_companions_uses_nav_tree() {
        let companions = vec![companion("acme/example/v1", "README", "v1 README")];
        let file = dummy_file("acme.example.v1");
        let proto_files = vec![file.clone()];
        let file_to_generate = vec!["acme/example/v1/a.proto".into()];
        let opts = Options {
            summary: true,
            ..Options::default()
        };
        let mut by_package: BTreeMap<String, Vec<(&str, &FileDescriptorProto)>> = BTreeMap::new();
        by_package.insert(
            "acme.example.v1".into(),
            vec![("acme/example/v1/a.proto", &file)],
        );
        let links = build_link_context(&by_package, &opts);
        let out = render_summary(&proto_files, &file_to_generate, &opts, &links, &companions)
            .expect("summary");
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
