//! Message definition synthesis.

use super::RenderContext;
use super::fence::{push_inline_comment_lines, render_proto_fence};
use crate::options::EscapeTags;
use crate::render::comments::{CommentIndex, path};
use crate::render::source::{SourceCache, push_indented_lines};
use buffa_descriptor::generated::descriptor::field_descriptor_proto::Type;
use buffa_descriptor::generated::descriptor::{DescriptorProto, FieldDescriptorProto};

pub fn synthesize_message_with_file(
    file_name: &str,
    idx: &CommentIndex<'_>,
    mi: usize,
    msg: &DescriptorProto,
    source: Option<&mut SourceCache>,
    ctx: Option<&RenderContext<'_>>,
) -> String {
    let name = msg.name.as_deref().unwrap_or("Message");
    let entity_doc = idx.leading_message(mi);
    let file_source =
        source.and_then(|cache| cache.load(file_name).map(|contents| contents.to_string()));
    let mut body = format!("message {name} {{\n");
    if let Some(src) = file_source.as_deref() {
        let opt_path = [path::FILE_MESSAGE, mi as i32, path::MSG_OPTIONS];
        if let Some(snippet) = idx.span_snippet(src, &opt_path) {
            push_indented_lines(&mut body, &snippet, "  ");
        }
    }
    synthesize_message_fields(&mut body, idx, mi, msg, file_source.as_deref());
    body.push_str("}\n");
    let escape_tags = ctx.map(|c| c.escape_tags).unwrap_or(EscapeTags::Off);
    render_proto_fence(file_name, entity_doc, &body, escape_tags)
}

fn synthesize_message_fields(
    body: &mut String,
    idx: &CommentIndex<'_>,
    mi: usize,
    msg: &DescriptorProto,
    file_source: Option<&str>,
) {
    let mut fi = 0;
    while fi < msg.field.len() {
        let field = &msg.field[fi];
        if let Some(oi) = field.oneof_index {
            if try_push_oneof_span(body, idx, mi, oi, file_source) {
                while fi < msg.field.len() && msg.field[fi].oneof_index == Some(oi) {
                    fi += 1;
                }
                continue;
            }
            let name = oneof_name(msg, oi);
            body.push_str(&format!("  oneof {name} {{\n"));
            while fi < msg.field.len() && msg.field[fi].oneof_index == Some(oi) {
                synthesize_message_field(body, idx, mi, fi, &msg.field[fi], "    ", file_source);
                fi += 1;
            }
            body.push_str("  }\n");
        } else {
            synthesize_message_field(body, idx, mi, fi, field, "  ", file_source);
            fi += 1;
        }
    }
}

fn synthesize_message_field(
    body: &mut String,
    idx: &CommentIndex<'_>,
    mi: usize,
    fi: usize,
    field: &FieldDescriptorProto,
    indent: &str,
    file_source: Option<&str>,
) {
    if let Some(c) = idx.leading_message_field(mi, fi) {
        push_inline_comment_lines(body, c);
    }
    let field_path = [path::FILE_MESSAGE, mi as i32, path::MSG_FIELD, fi as i32];
    if let Some(src) = file_source
        && let Some(snippet) = idx.span_snippet(src, &field_path)
    {
        push_indented_lines(body, &snippet, indent);
        return;
    }
    append_field(body, field, indent);
}

fn try_push_oneof_span(
    body: &mut String,
    idx: &CommentIndex<'_>,
    mi: usize,
    oi: i32,
    file_source: Option<&str>,
) -> bool {
    let Some(src) = file_source else {
        return false;
    };
    let oneof_path = [path::FILE_MESSAGE, mi as i32, path::MSG_ONEOF, oi];
    let Some(snippet) = idx.span_snippet(src, &oneof_path) else {
        return false;
    };
    if !snippet.contains("oneof") {
        return false;
    }
    push_indented_lines(body, &snippet, "  ");
    true
}

fn oneof_name(msg: &DescriptorProto, oi: i32) -> &str {
    msg.oneof_decl
        .get(oi as usize)
        .and_then(|o| o.name.as_deref())
        .unwrap_or("payload")
}

fn append_field(out: &mut String, field: &FieldDescriptorProto, indent: &str) {
    let label = match field.label {
        Some(
            buffa_descriptor::generated::descriptor::field_descriptor_proto::Label::LABEL_REPEATED,
        ) => "repeated ",
        _ => {
            if field.proto3_optional == Some(true) {
                "optional "
            } else {
                ""
            }
        }
    };
    let ty = field_type_name(field);
    let name = field.name.as_deref().unwrap_or("field");
    let number = field.number.unwrap_or(0);
    out.push_str(&format!("{indent}{label}{ty} {name} = {number};\n"));
}

fn field_type_name(field: &FieldDescriptorProto) -> String {
    if let Some(ref tn) = field.type_name {
        return super::fence::strip_leading_dot(tn).to_string();
    }
    scalar_type_name(field.r#type).unwrap_or_else(|| "bytes".into())
}

fn scalar_type_name(ty: Option<Type>) -> Option<String> {
    let ty = ty?;
    Some(
        match ty {
            Type::TYPE_DOUBLE => "double",
            Type::TYPE_FLOAT => "float",
            Type::TYPE_INT64 => "int64",
            Type::TYPE_UINT64 => "uint64",
            Type::TYPE_INT32 => "int32",
            Type::TYPE_FIXED64 => "fixed64",
            Type::TYPE_FIXED32 => "fixed32",
            Type::TYPE_BOOL => "bool",
            Type::TYPE_STRING => "string",
            Type::TYPE_GROUP => "group",
            Type::TYPE_MESSAGE => "message",
            Type::TYPE_BYTES => "bytes",
            Type::TYPE_UINT32 => "uint32",
            Type::TYPE_ENUM => "enum",
            Type::TYPE_SFIXED32 => "sfixed32",
            Type::TYPE_SFIXED64 => "sfixed64",
            Type::TYPE_SINT32 => "sint32",
            Type::TYPE_SINT64 => "sint64",
        }
        .into(),
    )
}
