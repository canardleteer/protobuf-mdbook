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

/// Group `file_to_generate` by package; file order within each package follows request order.
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

/// Visit entities in link/SUMMARY order: files vec order, then message/enum/service descriptor order.
pub fn for_each_entity_in_files(
    files: &[(&str, &FileDescriptorProto)],
    mut f: impl FnMut(EntityKind, &str, &str, &FileDescriptorProto, usize),
) {
    for (proto_name, file) in files {
        for (i, msg) in file.message_type.iter().enumerate() {
            if let Some(name) = msg.name.as_deref() {
                f(EntityKind::Message, name, proto_name, file, i);
            }
        }
        for (i, en) in file.enum_type.iter().enumerate() {
            if let Some(name) = en.name.as_deref() {
                f(EntityKind::Enum, name, proto_name, file, i);
            }
        }
        for (i, svc) in file.service.iter().enumerate() {
            if let Some(name) = svc.name.as_deref() {
                f(EntityKind::Service, name, proto_name, file, i);
            }
        }
    }
}

/// Build cross-reference paths for all entities in `by_package`.
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

/// Render package or entity markdown for every package in `by_package` (BTreeMap key order).
pub fn render_all(
    by_package: &BTreeMap<String, Vec<(&str, &FileDescriptorProto)>>,
    opts: &Options,
    links: &LinkContext,
    source: &mut source::SourceCache,
) -> Vec<GeneratedDoc> {
    let mut docs = Vec::new();
    for (package, files) in by_package {
        if package.is_empty() {
            continue;
        }
        match opts.layout {
            Layout::Package => {
                let (path, content) =
                    package::render_package_page(package, files, links, opts, source);
                docs.push(GeneratedDoc { path, content });
            }
            Layout::Entity | Layout::Split => {
                for (path, content) in
                    entity::render_entity_pages(package, files, links, opts, source)
                {
                    docs.push(GeneratedDoc { path, content });
                }
            }
        }
    }

    docs
}

/// Collect entity refs for one package in link/SUMMARY order.
pub(crate) fn collect_entities(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
) -> Vec<EntityRef> {
    let mut out = Vec::new();
    for_each_entity_in_files(files, |kind, name, _, _, _| {
        out.push(EntityRef {
            package: package.to_string(),
            kind,
            name: name.to_string(),
        });
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, package: &str) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.into()),
            package: Some(package.into()),
            ..Default::default()
        }
    }

    #[test]
    fn packages_map_preserves_file_to_generate_order_within_package() {
        let a = file("pkg/a.proto", "acme.v1");
        let b = file("pkg/b.proto", "acme.v1");
        let proto_files = vec![a.clone(), b.clone()];

        let forward_inputs = ["pkg/a.proto".into(), "pkg/b.proto".into()];
        let forward = packages_map(&proto_files, &forward_inputs);
        let reverse_inputs = ["pkg/b.proto".into(), "pkg/a.proto".into()];
        let reverse = packages_map(&proto_files, &reverse_inputs);

        let forward_names: Vec<_> = forward["acme.v1"].iter().map(|(n, _)| *n).collect();
        let reverse_names: Vec<_> = reverse["acme.v1"].iter().map(|(n, _)| *n).collect();

        assert_eq!(forward_names, ["pkg/a.proto", "pkg/b.proto"]);
        assert_eq!(reverse_names, ["pkg/b.proto", "pkg/a.proto"]);
    }
}
