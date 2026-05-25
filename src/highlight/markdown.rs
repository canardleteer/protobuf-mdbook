//! Markdown fence transformation for build-time highlighting.

use crate::highlight::cel::highlight_cel;
use crate::highlight::cel_split::split_message_cel_blocks;
use crate::highlight::html::normalize_newlines;
use crate::highlight::protobuf::highlight_protobuf;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// Which languages to highlight during preprocessing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HighlightConfig {
    pub protobuf: bool,
    pub cel: bool,
}

impl HighlightConfig {
    pub fn all() -> Self {
        Self {
            protobuf: true,
            cel: true,
        }
    }
}

/// Transform chapter markdown, replacing handled fences with highlighted HTML.
pub fn transform_chapter(content: &str, config: HighlightConfig) -> Result<String> {
    if !config.protobuf && !config.cel {
        return Ok(content.to_string());
    }
    let content = normalize_newlines(content);
    transform_markdown(&content, config)
}

fn transform_markdown(content: &str, config: HighlightConfig) -> Result<String> {
    let mut replacements: Vec<(std::ops::Range<usize>, String)> = Vec::new();

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut in_target_fence = false;
    let mut fence_lang = String::new();
    let mut code_span = 0..0;
    let mut start_new_code_span = true;

    let events = Parser::new_ext(content, opts);
    for (event, span) in events.into_offset_iter() {
        if let Event::Start(Tag::CodeBlock(kind)) = &event {
            if let CodeBlockKind::Fenced(lang) = kind {
                fence_lang = lang.to_string();
                in_target_fence = should_highlight_fence(&fence_lang, config);
            }
            continue;
        }

        if !in_target_fence {
            continue;
        }

        if matches!(event, Event::Text(_)) {
            if start_new_code_span {
                code_span = span.clone();
                start_new_code_span = false;
            } else {
                code_span = code_span.start..span.end;
            }
            continue;
        }

        if matches!(event, Event::End(TagEnd::CodeBlock)) {
            in_target_fence = false;
            let fence_body = &content[code_span.clone()];
            let html = render_fence(&fence_lang, fence_body, config);
            replacements.push((span, html));
            start_new_code_span = true;
            fence_lang.clear();
        }
    }

    let mut out = content.to_string();
    for (span, block) in replacements.into_iter().rev() {
        let pre = &out[..span.start];
        let post = &out[span.end..];
        out = format!("{pre}\n{block}\n{post}");
    }
    Ok(out)
}

fn should_highlight_fence(lang: &str, config: HighlightConfig) -> bool {
    match lang {
        "protobuf" | "proto" => config.protobuf,
        "cel" | "google-cel" => config.cel,
        _ => false,
    }
}

fn render_fence(lang: &str, body: &str, config: HighlightConfig) -> String {
    let body = body.replace("\r\n", "\n");
    match lang {
        "protobuf" | "proto" if config.protobuf => render_protobuf_fence(&body, config),
        "cel" | "google-cel" if config.cel => highlight_cel(body.trim_end()),
        _ => body,
    }
}

fn render_protobuf_fence(body: &str, config: HighlightConfig) -> String {
    let trimmed = body.trim_end();
    let (proto_body, cel_blocks) = split_message_cel_blocks(trimmed);
    let mut parts = vec![highlight_protobuf(proto_body.trim_end())];
    if config.cel {
        for block in cel_blocks {
            parts.push(highlight_cel(&block));
        }
    } else {
        for block in cel_blocks {
            parts.push(crate::highlight::html::escape_for_pre("cel", &block));
        }
    }
    parts.join("\n\n")
}

/// Highlight raw source for golden / xtask checks (language: `protobuf`, `proto`, `cel`).
pub fn highlight_source(language: &str, source: &str) -> String {
    let source = normalize_newlines(source);
    let config = HighlightConfig::all();
    match language {
        "protobuf" | "proto" => render_protobuf_fence(&source, config),
        "cel" | "google-cel" => highlight_cel(source.trim_end()),
        _ => panic!("unsupported highlight language: {language}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_protobuf_fence_to_pre() {
        let md = "```protobuf\nsyntax = \"proto3\";\n```\n";
        let out = transform_chapter(md, HighlightConfig::all()).unwrap();
        assert!(out.contains("<pre class=\"protobuf-mdbook language-protobuf\">"));
        assert!(out.contains("hljs-keyword"));
        assert!(!out.contains("```protobuf"));
    }

    #[test]
    fn splits_and_highlights_message_cel() {
        let md = r#"```protobuf
message M {
  option (buf.validate.message).cel = {
    id: "m.rule"
    expression: "true"
  };
}
```"#;
        let out = transform_chapter(md, HighlightConfig::all()).unwrap();
        assert!(out.contains("language-protobuf"));
        assert!(out.contains("language-cel"));
        assert!(out.contains("m.rule"));
    }

    #[test]
    fn leaves_rust_fence_untouched() {
        let md = "```rust\nfn main() {}\n```\n";
        let out = transform_chapter(md, HighlightConfig::all()).unwrap();
        assert!(out.contains("```rust"));
    }
}
