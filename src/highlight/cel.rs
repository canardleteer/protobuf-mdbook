//! Rust highlighter for CEL, mirroring repo-authored HLJS 10.1.1 rules.

use crate::highlight::engine::highlight_with_spans;
use regex::Regex;
use std::sync::LazyLock;

static CEL_KEYWORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(true|false|null|in|has|this|size|type|dyn|uint|int|double|string|bytes|duration|timestamp|exists|all|map|filter|id|message|expression)\b",
    )
    .expect("cel keywords")
});

static CEL_LINE_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//[^\n\r]*").expect("cel line comment"));

static CEL_BLOCK_COMMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").expect("cel block comment"));

static CEL_STRING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\])*""#).expect("cel string"));

static CEL_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b0[xX][\da-fA-F]+\b|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?u?\b").expect("cel number")
});

static CEL_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)\b(id|message|expression)\s*:").expect("cel title"));

static CEL_FUNCTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([a-zA-Z_][\w]*)\s*\(").expect("cel function"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    BlockComment,
    LineComment,
    String,
    Number,
    Title,
    Function,
    Keyword,
}

struct Match<'a> {
    start: usize,
    end: usize,
    kind: TokenKind,
    text: &'a str,
}

pub fn highlight_cel_inner(source: &str) -> String {
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
            TokenKind::Title => "hljs-title",
            TokenKind::Function => "hljs-function",
            TokenKind::Keyword => "hljs-keyword",
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
        &CEL_BLOCK_COMMENT,
        TokenKind::BlockComment,
        &mut out,
    );
    push_regex_matches(source, &CEL_LINE_COMMENT, TokenKind::LineComment, &mut out);
    push_regex_matches(source, &CEL_STRING, TokenKind::String, &mut out);
    push_regex_matches(source, &CEL_NUMBER, TokenKind::Number, &mut out);
    for cap in CEL_TITLE.captures_iter(source) {
        if let Some(title) = cap.get(1) {
            out.push(Match {
                start: title.start(),
                end: title.end(),
                kind: TokenKind::Title,
                text: title.as_str(),
            });
        }
    }
    for cap in CEL_FUNCTION.captures_iter(source) {
        if let Some(name) = cap.get(1) {
            out.push(Match {
                start: name.start(),
                end: name.end(),
                kind: TokenKind::Function,
                text: name.as_str(),
            });
        }
    }
    push_regex_matches(source, &CEL_KEYWORDS, TokenKind::Keyword, &mut out);
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
        TokenKind::Title => 2,
        TokenKind::Function => 3,
        TokenKind::Number => 4,
        TokenKind::Keyword => 5,
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

pub fn highlight_cel(source: &str) -> String {
    crate::highlight::html::wrap_pre("cel", &highlight_cel_inner(source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_cel_title_line() {
        let html = highlight_cel_inner("id: \"foo\"\nexpression: \"true\"");
        assert!(html.contains(r#"<span class="hljs-title">id</span>"#));
        assert!(html.contains(r#"<span class="hljs-title">expression</span>"#));
    }
}
