//! Per-entity pages (`layout=entity` and `layout=split`).

use crate::options::Options;
use crate::plugin_api::FileDescriptorProto;
use crate::render::comments::{CommentIndex, package_overview};
use crate::render::links::{EntityKind, LinkContext};
use crate::render::proto_syntax::{
    RenderContext, synthesize_enum, synthesize_message_with_file, synthesize_service,
};
use crate::render::source::SourceCache;
use crate::render::{for_each_entity_in_files, md_heading, push_paragraph_break};

/// Render package index plus one page per message, enum, and service (both entity layouts).
pub fn render_entity_pages(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
    links: &LinkContext,
    opts: &Options,
    source: &mut SourceCache,
) -> Vec<(String, String)> {
    let mut pages = Vec::new();

    {
        let index_rel = links.package_index_rel(package);
        let index_path = opts.output_path(index_rel.to_str().unwrap_or_default());
        let mut index = md_heading(1, package);
        if let Some(overview) = package_overview(files) {
            index.push_str(&overview);
            push_paragraph_break(&mut index);
        }
        index.push_str(&md_heading(2, "Contents"));
        let index_from = index_rel.as_path();
        for_each_entity_in_files(files, |kind, name, _, _, _| {
            let p = links.entity_path(package, kind, name).expect("entity");
            index.push_str("- ");
            index.push_str(&links.summary_link(index_from, p, name));
            index.push('\n');
        });
        pages.push((index_path, index));
    }

    for (proto_name, file) in files {
        let idx = CommentIndex::from_file(file);
        for (i, msg) in file.message_type.iter().enumerate() {
            let name = msg.name.as_deref().unwrap_or("Message");
            let rel = links
                .entity_path(package, EntityKind::Message, name)
                .expect("entity");
            let path = opts.output_path(rel.to_str().unwrap_or_default());
            let mut page = md_heading(1, name);
            page.push_str(&synthesize_message_with_file(
                proto_name,
                &idx,
                i,
                msg,
                Some(source),
            ));
            pages.push((path, page));
        }
        for (i, en) in file.enum_type.iter().enumerate() {
            let name = en.name.as_deref().unwrap_or("Enum");
            let rel = links
                .entity_path(package, EntityKind::Enum, name)
                .expect("entity");
            let path = opts.output_path(rel.to_str().unwrap_or_default());
            let mut page = md_heading(1, name);
            page.push_str(&synthesize_enum(proto_name, &idx, i, en));
            pages.push((path, page));
        }
        for (i, svc) in file.service.iter().enumerate() {
            let name = svc.name.as_deref().unwrap_or("Service");
            let rel = links
                .entity_path(package, EntityKind::Service, name)
                .expect("entity");
            let path = opts.output_path(rel.to_str().unwrap_or_default());
            let ctx = RenderContext {
                links: Some(links),
                from_md: rel.as_path(),
            };
            let page = synthesize_service(proto_name, &idx, i, svc, 1, Some(&ctx));
            pages.push((path, page));
        }
    }

    pages
}
