//! HTML wrappers for build-time highlighted blocks.

use crate::highlight::engine::escape_html;

/// Wrap highlighted inner HTML in a `<pre>` that mdBook `book.js` will not re-highlight.
pub fn wrap_pre(language: &str, inner_html: &str) -> String {
    format!("<pre class=\"protobuf-mdbook language-{language}\">{inner_html}</pre>")
}

/// Normalize line endings to LF (CRLF / lone CR from Windows or editors).
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Escape raw fence content for insertion into HTML when highlighting is disabled.
pub fn escape_for_pre(language: &str, source: &str) -> String {
    wrap_pre(language, &escape_html(source))
}
