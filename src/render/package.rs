//! Package-rollup markdown (`layout=package`).

use crate::options::Options;
use crate::plugin_api::FileDescriptorProto;
use crate::render::comments::{CommentIndex, package_overview};
use crate::render::links::LinkContext;
use crate::render::proto_syntax::{
    RenderContext, synthesize_enum, synthesize_message_with_file, synthesize_service,
};
use crate::render::source::SourceCache;
use crate::render::{md_heading, push_paragraph_break};

/// Package pages: `## Services` / `## Messages and enums`; services and messages at `###`.
const SECTION_LEVEL: usize = 2;
const ENTITY_LEVEL: usize = 3;

pub fn render_package_page(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
    links: &LinkContext,
    opts: &Options,
    source: &mut SourceCache,
) -> (String, String) {
    let rel = links.package_page_rel(package);
    let path = opts.output_path(rel.to_str().unwrap_or_default());
    let ctx = RenderContext {
        links: Some(links),
        from_md: rel.as_path(),
    };

    let mut out = String::new();
    out.push_str(&md_heading(1, package));

    if let Some(overview) = package_overview(files) {
        out.push_str(&overview);
        push_paragraph_break(&mut out);
    }

    let mut has_services = false;
    for (_, file) in files {
        if !file.service.is_empty() {
            has_services = true;
            break;
        }
    }
    if has_services {
        out.push_str(&md_heading(SECTION_LEVEL, "Services"));
        for (fname, file) in files {
            let idx = CommentIndex::from_file(file);
            for (i, svc) in file.service.iter().enumerate() {
                out.push_str(&synthesize_service(
                    fname,
                    &idx,
                    i,
                    svc,
                    ENTITY_LEVEL,
                    Some(&ctx),
                ));
            }
        }
    }

    let mut has_messages_enums = false;
    for (_, file) in files {
        if !file.message_type.is_empty() || !file.enum_type.is_empty() {
            has_messages_enums = true;
            break;
        }
    }
    if has_messages_enums {
        out.push_str(&md_heading(SECTION_LEVEL, "Messages and enums"));
        for (fname, file) in files {
            let idx = CommentIndex::from_file(file);
            for (i, msg) in file.message_type.iter().enumerate() {
                let name = msg.name.as_deref().unwrap_or("Message");
                out.push_str(&md_heading(ENTITY_LEVEL, name));
                out.push_str(&synthesize_message_with_file(
                    fname,
                    &idx,
                    i,
                    msg,
                    Some(source),
                ));
            }
            for (i, en) in file.enum_type.iter().enumerate() {
                let name = en.name.as_deref().unwrap_or("Enum");
                out.push_str(&md_heading(ENTITY_LEVEL, name));
                out.push_str(&synthesize_enum(fname, &idx, i, en));
            }
        }
    }

    (path, out)
}
