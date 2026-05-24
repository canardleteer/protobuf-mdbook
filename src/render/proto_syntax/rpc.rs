//! Service and RPC synthesis.

use super::fence::{push_markdown_doc, push_proto_fence_body, short_rpc_type, strip_leading_dot};
use crate::options::EscapeTags;
use crate::plugin_api::codegen::rpc_kind;
use crate::render::comments::CommentIndex;
use crate::render::links::LinkContext;
use crate::render::md_heading;
use crate::render::proto_syntax::RenderContext;
use buffa_descriptor::generated::descriptor::method_options::IdempotencyLevel;
use buffa_descriptor::generated::descriptor::{
    MethodDescriptorProto, ServiceDescriptorProto, UninterpretedOption,
};

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
        let escape_tags = ctx.map(|c| c.escape_tags).unwrap_or(EscapeTags::Off);
        push_markdown_doc(&mut out, c, escape_tags);
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
        let escape_tags = ctx.map(|c| c.escape_tags).unwrap_or(EscapeTags::Off);
        push_markdown_doc(out, c, escape_tags);
    }
}

fn push_rpc_signature_line(
    out: &mut String,
    method: &MethodDescriptorProto,
    ctx: Option<&RenderContext<'_>>,
) {
    out.push_str(&format!("{}\n\n", rpc_signature_markdown(method, ctx)));
}

pub(crate) fn rpc_signature_markdown(
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
        .map(|links: &LinkContext| links.link_type(ctx.from_md, fqn))
        .unwrap_or_else(|| format!("`{short}`"))
}

pub(crate) fn synthesize_method_options_body(method: &MethodDescriptorProto) -> Option<String> {
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
