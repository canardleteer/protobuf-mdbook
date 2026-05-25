//! Rust highlighter for protobuf/proto, mirroring vendored HLJS 10.1.1 rules.

use crate::highlight::engine::highlight_with_spans;
use regex::Regex;
use std::sync::LazyLock;

static PROTO_KEYWORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(package|import|option|optional|required|repeated|group|oneof|syntax|edition|extend|reserved|map|weak|public)\b",
    )
    .expect("proto keywords")
});

static PROTO_BUILTINS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(double|float|int32|int64|uint32|uint64|sint32|sint64|fixed32|fixed64|sfixed32|sfixed64|bool|string|bytes)\b",
    )
    .expect("proto builtins")
});

static PROTO_LITERALS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(true|false)\b").expect("proto literals"));

static PROTO_LINE_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//[^\n\r]*").expect("proto line comment"));

static PROTO_BLOCK_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").expect("proto block comment"));

static PROTO_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\])*""#).expect("proto string"));

static PROTO_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0[xX][\da-fA-F]+\b|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b").expect("proto number")
});

static PROTO_CLASS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(message|enum|service)\s+([A-Za-z_][\w.]*)").expect("proto class")
});

static PROTO_RPC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\brpc\s+([A-Za-z_][\w]*)\s*\(").expect("proto rpc"));

static PROTO_OPTION_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*([A-Z_][A-Z0-9_]*)\s*=").expect("proto option name"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    BlockComment,
    LineComment,
    String,
    Number,
    ClassKeyword,
    ClassTitle,
    RpcKeyword,
    RpcName,
    OptionName,
    Keyword,
    BuiltIn,
    Literal,
}

struct Match<'a> {
    start: usize,
    end: usize,
    kind: TokenKind,
    text: &'a str,
}

/// Highlight protobuf source into inner HTML (spans only, no wrapper).
pub fn highlight_protobuf_inner(source: &str) -> String {
    let matches = resolve_matches(collect_matches(source));
    let mut spans: Vec<(Option<&str>, &str)> = Vec::new();
    let mut pos = 0;
    for m in matches {
        if m.start < pos {
            continue;
        }
        if m.start > pos {
            spans.push((None, &source[pos..m.start]));
        }
        let class = match m.kind {
            TokenKind::BlockComment | TokenKind::LineComment => "hljs-comment",
            TokenKind::String => "hljs-string",
            TokenKind::Number => "hljs-number",
            TokenKind::ClassKeyword => "hljs-keyword",
            TokenKind::ClassTitle => "hljs-title",
            TokenKind::RpcKeyword => "hljs-keyword",
            TokenKind::RpcName => "hljs-title",
            TokenKind::OptionName => "hljs-symbol",
            TokenKind::Keyword => "hljs-keyword",
            TokenKind::BuiltIn => "hljs-built_in",
            TokenKind::Literal => "hljs-literal",
        };
        spans.push((Some(class), m.text));
        pos = m.end;
    }
    if pos < source.len() {
        spans.push((None, &source[pos..]));
    }
    highlight_with_spans(&spans)
}

fn collect_matches(source: &str) -> Vec<Match<'_>> {
    let mut out = Vec::new();
    push_regex_matches(
        source,
        &PROTO_BLOCK_COMMENT,
        TokenKind::BlockComment,
        &mut out,
    );
    push_regex_matches(
        source,
        &PROTO_LINE_COMMENT,
        TokenKind::LineComment,
        &mut out,
    );
    push_regex_matches(source, &PROTO_STRING, TokenKind::String, &mut out);
    push_regex_matches(source, &PROTO_NUMBER, TokenKind::Number, &mut out);
    for cap in PROTO_CLASS.captures_iter(source) {
        if let Some(kw) = cap.get(1) {
            out.push(Match {
                start: kw.start(),
                end: kw.end(),
                kind: TokenKind::ClassKeyword,
                text: kw.as_str(),
            });
        }
        if let Some(title) = cap.get(2) {
            out.push(Match {
                start: title.start(),
                end: title.end(),
                kind: TokenKind::ClassTitle,
                text: title.as_str(),
            });
        }
    }
    for cap in PROTO_RPC.captures_iter(source) {
        if let Some(kw) = cap.get(0) {
            let name = cap.get(1).unwrap();
            out.push(Match {
                start: kw.start(),
                end: name.start(),
                kind: TokenKind::RpcKeyword,
                text: kw.as_str().trim_end(),
            });
            out.push(Match {
                start: name.start(),
                end: name.end(),
                kind: TokenKind::RpcName,
                text: name.as_str(),
            });
        }
    }
    push_regex_matches(source, &PROTO_OPTION_NAME, TokenKind::OptionName, &mut out);
    push_regex_matches(source, &PROTO_KEYWORDS, TokenKind::Keyword, &mut out);
    push_regex_matches(source, &PROTO_BUILTINS, TokenKind::BuiltIn, &mut out);
    push_regex_matches(source, &PROTO_LITERALS, TokenKind::Literal, &mut out);
    out
}

fn push_regex_matches<'a>(source: &'a str, re: &Regex, kind: TokenKind, out: &mut Vec<Match<'a>>) {
    for m in re.find_iter(source) {
        out.push(Match {
            start: m.start(),
            end: m.end(),
            kind,
            text: m.as_str(),
        });
    }
}

fn priority(kind: TokenKind) -> u8 {
    match kind {
        TokenKind::BlockComment | TokenKind::LineComment => 0,
        TokenKind::String => 1,
        TokenKind::ClassTitle | TokenKind::RpcName => 2,
        TokenKind::ClassKeyword | TokenKind::RpcKeyword | TokenKind::OptionName => 3,
        TokenKind::Number => 4,
        TokenKind::BuiltIn | TokenKind::Literal => 5,
        TokenKind::Keyword => 6,
    }
}

fn resolve_matches(mut matches: Vec<Match<'_>>) -> Vec<Match<'_>> {
    matches.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| priority(a.kind).cmp(&priority(b.kind)))
            .then_with(|| b.end.cmp(&a.end))
    });
    let mut resolved = Vec::new();
    let mut cursor = 0;
    for m in matches {
        if m.start >= cursor {
            cursor = m.end;
            resolved.push(m);
        }
    }
    resolved
}

/// Full `<pre>` block for protobuf content.
pub fn highlight_protobuf(source: &str) -> String {
    crate::highlight::html::wrap_pre("protobuf", &highlight_protobuf_inner(source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_syntax_keyword() {
        let html = highlight_protobuf_inner(r#"syntax = "proto3";"#);
        assert!(html.contains(r#"<span class="hljs-keyword">syntax</span>"#));
        assert!(html.contains(r#"<span class="hljs-string">&quot;proto3&quot;</span>"#));
    }

    #[test]
    fn highlights_message_title() {
        let html = highlight_protobuf_inner("message Foo {");
        assert!(html.contains(r#"<span class="hljs-keyword">message</span>"#));
        assert!(html.contains(r#"<span class="hljs-title">Foo</span>"#));
    }

    #[test]
    fn line_comment_excludes_crlf() {
        let html = highlight_protobuf_inner("  // inline comment\r\n");
        assert!(html.contains(r#"<span class="hljs-comment">// inline comment</span>"#));
        assert!(!html.contains("// inline comment\r"));
    }
}
