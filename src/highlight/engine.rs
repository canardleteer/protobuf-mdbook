//! Shared HTML span emission for hljs-compatible highlighting.

pub(crate) fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

pub(crate) fn span(class: &str, text: &str) -> String {
    format!(r#"<span class="{class}">{}</span>"#, escape_html(text))
}

pub(crate) fn plain(text: &str) -> String {
    escape_html(text)
}

pub(crate) fn highlight_with_spans(spans: &[(Option<&str>, &str)]) -> String {
    let mut out = String::new();
    for (class, text) in spans {
        match class {
            Some(c) => out.push_str(&span(c, text)),
            None => out.push_str(&plain(text)),
        }
    }
    out
}
