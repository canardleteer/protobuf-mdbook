//! Render `mdbook_summary::Summary` to SUMMARY.md markdown.

use crate::render::links::relative_path_from_dir;
use mdbook_summary::{Link, Summary, SummaryItem};
use std::path::{Path, PathBuf};

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
    let from_dir = summary_from.parent().unwrap_or(Path::new(""));
    PathBuf::from(relative_path_from_dir(from_dir, target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_summary::{SummaryItem, parse_summary};

    #[derive(Debug, PartialEq, Eq)]
    struct LinkShape {
        name: String,
        location: Option<String>,
        nested: Vec<LinkShape>,
    }

    fn link_shapes(items: &[SummaryItem]) -> Vec<LinkShape> {
        items
            .iter()
            .filter_map(|item| {
                if let SummaryItem::Link(link) = item {
                    Some(LinkShape {
                        name: link.name.clone(),
                        location: link
                            .location
                            .as_ref()
                            .map(|p| p.to_string_lossy().into_owned()),
                        nested: link_shapes(&link.nested_items),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

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

    #[test]
    fn render_parse_round_trip_preserves_structure() {
        let mut nested = Link::new("Nested", "src/nested.md");
        nested
            .nested_items
            .push(SummaryItem::Link(Link::new("Deep", "src/deep.md")));

        let mut top = Link::new("Chapter", "src/chapter.md");
        top.nested_items.push(SummaryItem::Link(nested));

        let mut summary = Summary::default();
        summary.title = Some("Book title".into());
        summary.numbered_chapters = vec![SummaryItem::Link(top)];

        let md = render_summary_markdown(&summary);
        let parsed = parse_summary(&md).expect("parse rendered SUMMARY");

        assert_eq!(parsed.title.as_deref(), Some("Book title"));
        assert_eq!(
            link_shapes(&parsed.numbered_chapters),
            link_shapes(&summary.numbered_chapters)
        );
    }
}
