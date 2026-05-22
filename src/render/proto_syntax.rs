//! Synthesize protobuf source snippets from descriptors (mdBook `protobuf` fences).
//!
//! **Comment policy:** `SourceCodeInfo` *leading* comments on the entity (message,
//! enum, service, RPC) are emitted as Markdown *before* the fence. Leading comments
//! on fields and enum values stay as `//` lines inside the synthesized block.

use crate::plugin_api::codegen::rpc_kind;
use crate::render::cel_fence::split_message_cel_blocks;
use crate::render::comments::{CommentIndex, path};
use crate::render::links::LinkContext;
use crate::render::md_heading;
use crate::render::source::{SourceCache, push_indented_lines};
use buffa_descriptor::generated::descriptor::field_descriptor_proto::Type;
use buffa_descriptor::generated::descriptor::method_options::IdempotencyLevel;
use buffa_descriptor::generated::descriptor::{
    DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, MethodDescriptorProto,
    ServiceDescriptorProto, UninterpretedOption,
};
use std::path::Path;

pub struct RenderContext<'a> {
    pub links: Option<&'a LinkContext>,
    pub from_md: &'a Path,
}

pub fn synthesize_message_with_file(
    file_name: &str,
    idx: &CommentIndex<'_>,
    mi: usize,
    msg: &DescriptorProto,
    source: Option<&mut SourceCache>,
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
    render_proto_fence(file_name, entity_doc, &body)
}

/// Enum: leading comment as Markdown summary, then a `protobuf` fence for the definition.
pub fn synthesize_enum(
    file_name: &str,
    idx: &CommentIndex<'_>,
    ei: usize,
    en: &EnumDescriptorProto,
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
    render_proto_fence(file_name, entity_doc, &body)
}

/// Render one service: heading, file path, service doc, then each RPC (signature, options, doc).
pub fn synthesize_service(
    file_name: &str,
    idx: &CommentIndex<'_>,
    si: usize,
    svc: &ServiceDescriptorProto,
    service_heading_level: usize,
    ctx: Option<&RenderContext<'_>>,
) -> String {
    let name = svc.name.as_deref().unwrap_or("Service");
    let mut out = String::new();

    out.push_str(&md_heading(service_heading_level, name));
    if !file_name.is_empty() {
        out.push_str(&format!("*`{file_name}`*\n\n"));
    }
    if let Some(c) = idx.leading_service(si) {
        push_markdown_doc(&mut out, c);
    }

    for (mi, method) in svc.method.iter().enumerate() {
        push_rpc_section(&mut out, idx, si, mi, method, ctx);
    }

    out
}

fn push_rpc_section(
    out: &mut String,
    idx: &CommentIndex<'_>,
    si: usize,
    mi: usize,
    method: &MethodDescriptorProto,
    ctx: Option<&RenderContext<'_>>,
) {
    push_rpc_signature_line(out, method, ctx);
    if let Some(body) = synthesize_method_options_body(method) {
        push_proto_fence_body(out, &body);
    }
    if let Some(c) = idx.leading_method(si, mi) {
        push_markdown_doc(out, c);
    }
}

/// RPC signature as bold prose (not a heading — avoids polluting the mdBook TOC).
fn push_rpc_signature_line(
    out: &mut String,
    method: &MethodDescriptorProto,
    ctx: Option<&RenderContext<'_>>,
) {
    out.push_str(&format!("{}\n\n", rpc_signature_markdown(method, ctx)));
}

fn rpc_signature_markdown(
    method: &MethodDescriptorProto,
    ctx: Option<&RenderContext<'_>>,
) -> String {
    let name = method.name.as_deref().unwrap_or("Rpc");
    let input_fqn = method
        .input_type
        .as_deref()
        .unwrap_or(".google.protobuf.Empty");
    let output_fqn = method
        .output_type
        .as_deref()
        .unwrap_or(".google.protobuf.Empty");
    let kind = rpc_kind(method);
    let (in_kw, out_kw) = match kind {
        crate::plugin_api::codegen::RpcKind::Unary => ("", ""),
        crate::plugin_api::codegen::RpcKind::ClientStreaming => ("stream ", ""),
        crate::plugin_api::codegen::RpcKind::ServerStreaming => ("", "stream "),
        crate::plugin_api::codegen::RpcKind::BidiStreaming => ("stream ", "stream "),
    };

    let input = type_link(ctx, input_fqn);
    let output = type_link(ctx, output_fqn);
    format!("**{name}** ( {in_kw}{input} ) returns ( {out_kw}{output} )")
}

fn type_link(ctx: Option<&RenderContext<'_>>, fqn: &str) -> String {
    let short = short_rpc_type(strip_leading_dot(fqn));
    let Some(ctx) = ctx else {
        return format!("`{short}`");
    };
    ctx.links
        .map(|links| links.link_type(ctx.from_md, fqn))
        .unwrap_or_else(|| format!("`{short}`"))
}

fn synthesize_method_options_body(method: &MethodDescriptorProto) -> Option<String> {
    let opts = method.options.as_option()?;
    let mut lines = Vec::new();
    if opts.deprecated == Some(true) {
        lines.push("option deprecated = true;".to_string());
    }
    if let Some(level) = opts.idempotency_level
        && let Some(name) = idempotency_level_name(level)
    {
        lines.push(format!("option idempotency_level = {name};"));
    }
    for uo in &opts.uninterpreted_option {
        if let Some(line) = format_uninterpreted_option(uo) {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn idempotency_level_name(level: IdempotencyLevel) -> Option<&'static str> {
    match level {
        IdempotencyLevel::NO_SIDE_EFFECTS => Some("NO_SIDE_EFFECTS"),
        IdempotencyLevel::IDEMPOTENT => Some("IDEMPOTENT"),
        IdempotencyLevel::IDEMPOTENCY_UNKNOWN => None,
    }
}

fn format_uninterpreted_option(opt: &UninterpretedOption) -> Option<String> {
    let mut name = String::new();
    for part in &opt.name {
        let piece = &part.name_part;
        if part.is_extension {
            name.push('(');
            name.push_str(piece);
            name.push(')');
        } else if !name.is_empty() {
            name.push('.');
            name.push_str(piece);
        } else {
            name.push_str(piece);
        }
    }
    if name.is_empty() {
        return None;
    }
    let value = opt
        .identifier_value
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| opt.positive_int_value.map(|n| n.to_string()))
        .or_else(|| opt.negative_int_value.map(|n| n.to_string()))
        .or_else(|| opt.double_value.map(|n| n.to_string()))
        .or_else(|| {
            opt.string_value
                .as_ref()
                .and_then(|b| std::str::from_utf8(b).ok().map(|s| format!("\"{s}\"")))
        })
        .or_else(|| opt.aggregate_value.clone())?;
    Some(format!("option {name} = {value};"))
}

fn render_proto_fence(file_name: &str, entity_doc: Option<&str>, body: &str) -> String {
    let mut out = String::new();
    if let Some(c) = entity_doc {
        push_markdown_doc(&mut out, c);
    }
    if !file_name.is_empty() {
        out.push_str(&format!("*`{file_name}`*\n\n"));
    }
    let (body, cel_blocks) = split_message_cel_blocks(body);
    push_proto_fence_body(&mut out, &body);
    for block in cel_blocks {
        out.push_str("**Protovalidate (CEL)**\n\n");
        push_cel_fence_body(&mut out, &block);
    }
    out
}

fn push_cel_fence_body(out: &mut String, body: &str) {
    out.push_str("```cel\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n\n");
}

fn push_proto_fence_body(out: &mut String, body: &str) {
    out.push_str("```protobuf\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n\n");
}

fn short_rpc_type(fqn: &str) -> String {
    fqn.rsplit('.').next().unwrap_or(fqn).to_string()
}

fn strip_leading_dot(s: &str) -> &str {
    s.strip_prefix('.').unwrap_or(s)
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

/// Prefer a source span for the whole `oneof` block when `SourceCodeInfo` provides one.
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
        return strip_leading_dot(tn).to_string();
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

/// Entity / RPC docs: verbatim Markdown (not inside a code fence).
fn push_markdown_doc(out: &mut String, comment: &str) {
    out.push_str(&dedent_comment(comment));
    out.push_str("\n\n");
}

/// Strip a common leading indent from block comments (protoc often preserves `  //` spacing).
fn dedent_comment(comment: &str) -> String {
    let lines: Vec<&str> = comment.trim().lines().collect();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                let mut s: String = l.chars().skip(min_indent).collect();
                // Protoc often leaves one space after stripping `//` on indented RPC comments.
                if let Some(rest) = s.strip_prefix(' ') {
                    s = rest.to_string();
                }
                s
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Field / enum-value docs: `//` lines inside a synthesized proto block.
fn push_inline_comment_lines(out: &mut String, comment: &str) {
    for line in comment.lines() {
        out.push_str("// ");
        out.push_str(line);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa_descriptor::generated::descriptor::{MethodDescriptorProto, MethodOptions};

    #[test]
    fn rpc_signature_server_streaming() {
        let m = MethodDescriptorProto {
            name: Some("EchoServerStream".into()),
            input_type: Some(".acme.example.v1.EchoServerStreamRequest".into()),
            output_type: Some(".acme.example.v1.EchoServerStreamResponse".into()),
            client_streaming: Some(false),
            server_streaming: Some(true),
            ..Default::default()
        };
        let line = rpc_signature_markdown(&m, None);
        assert!(line.contains("**EchoServerStream**"));
        assert!(line.contains("returns ( stream "));
    }

    #[test]
    fn method_options_idempotency() {
        let m = MethodDescriptorProto {
            options: buffa_descriptor::generated::descriptor::MethodOptions {
                idempotency_level: Some(IdempotencyLevel::NO_SIDE_EFFECTS),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let body = synthesize_method_options_body(&m).expect("options");
        assert!(body.contains("option idempotency_level = NO_SIDE_EFFECTS;"));
    }

    #[test]
    fn entity_doc_outside_fence() {
        let md = render_proto_fence(
            "foo.proto",
            Some("```mermaid\nflowchart LR\n  A --> B\n```"),
            "message X {}\n",
        );
        assert!(md.contains("```mermaid"));
        let fence_start = md.find("```protobuf").expect("fence");
        let mermaid_start = md.find("```mermaid").expect("mermaid");
        assert!(mermaid_start < fence_start);
        assert!(!md[fence_start..].contains("```mermaid"));
    }

    #[test]
    fn message_oneof_block_from_descriptor() {
        use buffa_descriptor::generated::descriptor::{FieldDescriptorProto, OneofDescriptorProto};

        let msg = DescriptorProto {
            name: Some("RelayConnectRequest".into()),
            oneof_decl: vec![OneofDescriptorProto {
                name: Some("payload".into()),
                ..Default::default()
            }],
            field: vec![
                FieldDescriptorProto {
                    name: Some("open".into()),
                    number: Some(1),
                    type_name: Some(".acme.example.v1.RelayOpen".into()),
                    oneof_index: Some(0),
                    ..Default::default()
                },
                FieldDescriptorProto {
                    name: Some("frame".into()),
                    number: Some(2),
                    type_name: Some(".acme.example.v1.RelayFrame".into()),
                    oneof_index: Some(0),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let file = crate::plugin_api::FileDescriptorProto::default();
        let idx = CommentIndex::from_file(&file);
        let md = synthesize_message_with_file("gateway.proto", &idx, 0, &msg, None);
        assert!(md.contains("oneof payload"));
        assert!(md.contains("RelayOpen open = 1;"));
        let oneof_pos = md.find("oneof payload").expect("oneof");
        let open_pos = md.find("RelayOpen open").expect("open");
        assert!(oneof_pos < open_pos);
    }

    #[test]
    fn service_rpc_order_signature_options_doc() {
        let file = crate::plugin_api::FileDescriptorProto::default();
        let idx = CommentIndex::from_file(&file);
        let m = MethodDescriptorProto {
            name: Some("GetCommits".into()),
            input_type: Some(".pkg.GetCommitsRequest".into()),
            output_type: Some(".pkg.GetCommitsResponse".into()),
            options: MethodOptions {
                idempotency_level: Some(IdempotencyLevel::NO_SIDE_EFFECTS),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let svc = ServiceDescriptorProto {
            name: Some("CommitService".into()),
            method: vec![m],
            ..Default::default()
        };
        let md = synthesize_service("commit.proto", &idx, 0, &svc, 3, None);
        let sig = md.find("**GetCommits**").expect("signature line");
        let opt = md.find("option idempotency_level").expect("options");
        assert!(sig < opt);
        assert!(!md.contains("service CommitService"));
    }
}
