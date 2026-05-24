//! Protobuf fence rendering and comment formatting.

use crate::options::EscapeTags;
use crate::render::cel_fence::split_message_cel_blocks;
use crate::render::markdown_doc::format_markdown_doc;

pub(crate) fn render_proto_fence(
    file_name: &str,
    entity_doc: Option<&str>,
    body: &str,
    escape_tags: EscapeTags,
) -> String {
    let mut out = String::new();
    if let Some(c) = entity_doc {
        push_markdown_doc(&mut out, c, escape_tags);
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

pub(crate) fn push_proto_fence_body(out: &mut String, body: &str) {
    out.push_str("```protobuf\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n\n");
}

fn push_cel_fence_body(out: &mut String, body: &str) {
    out.push_str("```cel\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("```\n\n");
}

/// Entity / RPC docs: verbatim Markdown by default; optional `escape_tags` transforms prose.
pub(crate) fn push_markdown_doc(out: &mut String, comment: &str, escape_tags: EscapeTags) {
    let formatted = format_markdown_doc(&dedent_comment(comment), escape_tags);
    out.push_str(&formatted);
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
pub(crate) fn push_inline_comment_lines(out: &mut String, comment: &str) {
    for line in comment.lines() {
        out.push_str("// ");
        out.push_str(line);
        out.push('\n');
    }
}

pub(crate) fn short_rpc_type(fqn: &str) -> String {
    fqn.rsplit('.').next().unwrap_or(fqn).to_string()
}

pub(crate) fn strip_leading_dot(s: &str) -> &str {
    s.strip_prefix('.').unwrap_or(s)
}
