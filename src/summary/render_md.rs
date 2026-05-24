//! Render `mdbook_summary::Summary` to SUMMARY.md markdown.

use mdbook_summary::{Link, Summary, SummaryItem};
use std::path::Path;

/// Bullet levels below the H1 title.
pub const SUMMARY_MAX_DEPTH: usize = 4;

pub fn render_summary_markdown(summary: &Summary) -> String {
    let mut out = String::new();
    if let Some(title) = &summary.title {
        out.push_str("# ");
        out.push_str(title);
        out.push_str("\n\n");
    }
    for item in &summary.prefix_chapters {
        push_item(&mut out, item, 0);
    }
    for item in &summary.numbered_chapters {
        push_item(&mut out, item, 0);
    }
    for item in &summary.suffix_chapters {
        push_item(&mut out, item, 0);
    }
    out
}

fn push_item(out: &mut String, item: &SummaryItem, depth: usize) {
    match item {
        SummaryItem::Link(link) => {
            push_link(out, link, depth);
        }
        SummaryItem::Separator => {
            out.push_str("---\n\n");
        }
        SummaryItem::PartTitle(title) => {
            out.push_str("# ");
            out.push_str(title);
            out.push_str("\n\n");
        }
        _ => {}
    }
}

fn push_link(out: &mut String, link: &Link, depth: usize) {
    let indent = "  ".repeat(depth);
    out.push_str(&indent);
    out.push_str("- [");
    out.push_str(&link.name);
    out.push_str("](");
    if let Some(loc) = &link.location {
        out.push_str(&loc.to_string_lossy());
    }
    out.push_str(")\n");
    for nested in &link.nested_items {
        if let SummaryItem::Link(n) = nested {
            push_link(out, n, depth + 1);
        } else {
            push_item(out, nested, depth + 1);
        }
    }
}

/// Warn on stderr if mdBook cannot parse the rendered SUMMARY (still returns `text`).
pub fn validate_summary_warn(text: &str) {
    if let Err(e) = mdbook_summary::parse_summary(text) {
        eprintln!("protobuf-mdbook: warning: generated SUMMARY.md failed parse_summary: {e}");
    }
}

/// Relative link path from `summary_path` to `target` under markdown_root/book layout.
pub fn link_path_for_summary(summary_from: &Path, target: &Path) -> PathBuf {
    use std::path::PathBuf;
    let from_dir = summary_from.parent().unwrap_or(Path::new(""));
    let from_parts: Vec<_> = from_dir.components().collect();
    let target_parts: Vec<_> = target.components().collect();
    let mut i = 0;
    while i < from_parts.len() && i < target_parts.len() && from_parts[i] == target_parts[i] {
        i += 1;
    }
    let ups = from_parts.len().saturating_sub(i);
    let mut parts: Vec<String> = (0..ups).map(|_| "..".to_string()).collect();
    for c in &target_parts[i..] {
        parts.push(c.as_os_str().to_string_lossy().into_owned());
    }
    if parts.is_empty() {
        target
            .file_name()
            .map(|s| PathBuf::from(s.to_string_lossy().into_owned()))
            .unwrap_or_default()
    } else {
        PathBuf::from(parts.join("/"))
    }
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_summary::SummaryItem;

    #[test]
    fn rendered_summary_parses() {
        let mut link = Link::new("Chapter", "src/chapter.md");
        link.nested_items
            .push(Link::new("Nested", "src/nested.md").into());
        let mut summary = Summary::default();
        summary.title = Some("Book".into());
        summary.numbered_chapters = vec![SummaryItem::Link(link)];
        let md = render_summary_markdown(&summary);
        mdbook_summary::parse_summary(&md).expect("round-trip");
        assert!(md.starts_with("# Book\n"));
    }
}
