//! Markdown rendering from descriptors.

pub mod cel_fence;
pub mod comments;
pub mod entity;
pub mod links;
pub mod package;
pub mod proto_syntax;
pub mod source;

use crate::options::{Layout, Options};
use crate::plugin_api::FileDescriptorProto;
use links::{EntityKind, EntityRef, LinkContext};
use std::collections::BTreeMap;

pub struct GeneratedDoc {
    pub path: String,
    pub content: String,
}

/// Markdown ATX heading (`level` hashes). Built at runtime for Rust 2024 `##` literal rules.
pub(crate) fn md_heading(level: usize, text: &str) -> String {
    let mut s = "#".repeat(level);
    s.push(' ');
    s.push_str(text);
    s.push('\n');
    s.push('\n');
    s
}

pub(crate) fn push_paragraph_break(out: &mut String) {
    out.push('\n');
    out.push('\n');
}

pub fn packages_map<'a>(
    proto_files: &'a [FileDescriptorProto],
    file_to_generate: &'a [String],
) -> BTreeMap<String, Vec<(&'a str, &'a FileDescriptorProto)>> {
    let mut by_package: BTreeMap<String, Vec<(&'a str, &'a FileDescriptorProto)>> = BTreeMap::new();
    for name in file_to_generate {
        let Some(file) = proto_files
            .iter()
            .find(|f| f.name.as_deref() == Some(name.as_str()))
        else {
            continue;
        };
        let pkg = file.package.clone().unwrap_or_default();
        by_package
            .entry(pkg)
            .or_default()
            .push((name.as_str(), file));
    }
    by_package
}

pub fn collect_entities_for_package(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
) -> Vec<EntityRef> {
    collect_entities(package, files)
}

pub fn build_link_context(
    by_package: &BTreeMap<String, Vec<(&str, &FileDescriptorProto)>>,
    opts: &Options,
) -> LinkContext {
    let entities: Vec<EntityRef> = by_package
        .iter()
        .flat_map(|(pkg, files)| collect_entities(pkg, files))
        .collect();
    LinkContext::new(opts.layout, &opts.book_root, &opts.markdown_root, entities)
}

pub fn render_all(
    proto_files: &[FileDescriptorProto],
    file_to_generate: &[String],
    opts: &Options,
    links: &LinkContext,
    source: &mut source::SourceCache,
) -> Vec<GeneratedDoc> {
    let by_package = packages_map(proto_files, file_to_generate);

    let mut docs = Vec::new();
    for (package, files) in &by_package {
        if package.is_empty() {
            continue;
        }
        match opts.layout {
            Layout::Package => {
                let (path, content) =
                    package::render_package_page(package, files, links, opts, source);
                docs.push(GeneratedDoc { path, content });
            }
            Layout::Entity => {
                for (path, content) in
                    entity::render_entity_pages(package, files, links, opts, false, source)
                {
                    docs.push(GeneratedDoc { path, content });
                }
            }
            Layout::Split => {
                for (path, content) in
                    entity::render_entity_pages(package, files, links, opts, true, source)
                {
                    docs.push(GeneratedDoc { path, content });
                }
            }
        }
    }

    docs
}

pub(crate) fn collect_entities(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for (_, file) in files {
        for msg in &file.message_type {
            if let Some(name) = &msg.name {
                out.push(EntityRef {
                    package: package.to_string(),
                    kind: EntityKind::Message,
                    name: name.clone(),
                });
            }
        }
        for en in &file.enum_type {
            if let Some(name) = &en.name {
                out.push(EntityRef {
                    package: package.to_string(),
                    kind: EntityKind::Enum,
                    name: name.clone(),
                });
            }
        }
        for svc in &file.service {
            if let Some(name) = &svc.name {
                out.push(EntityRef {
                    package: package.to_string(),
                    kind: EntityKind::Service,
                    name: name.clone(),
                });
            }
        }
    }
    out
}
