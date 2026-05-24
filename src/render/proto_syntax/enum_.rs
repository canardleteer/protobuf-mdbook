//! Enum definition synthesis.

use super::RenderContext;
use super::fence::{push_inline_comment_lines, render_proto_fence};
use crate::options::EscapeTags;
use crate::render::comments::CommentIndex;
use buffa_descriptor::generated::descriptor::EnumDescriptorProto;

pub fn synthesize_enum(
    file_name: &str,
    idx: &CommentIndex<'_>,
    ei: usize,
    en: &EnumDescriptorProto,
    ctx: Option<&RenderContext<'_>>,
) -> String {
    let name = en.name.as_deref().unwrap_or("Enum");
    let entity_doc = idx.leading_enum(ei);
    let mut body = format!("enum {name} {{\n");
    for (vi, val) in en.value.iter().enumerate() {
        if let Some(c) = idx.leading_enum_value(ei, vi) {
            push_inline_comment_lines(&mut body, c);
        }
        body.push_str(&format!(
            "  {} = {};\n",
            val.name.as_deref().unwrap_or("UNKNOWN"),
            val.number.unwrap_or(0)
        ));
    }
    body.push_str("}\n");
    let escape_tags = ctx.map(|c| c.escape_tags).unwrap_or(EscapeTags::Off);
    render_proto_fence(file_name, entity_doc, &body, escape_tags)
}
