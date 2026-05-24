//! Shared SUMMARY chapter link builders for flat and nav-tree layouts.

use crate::options::Layout;
use crate::plugin_api::FileDescriptorProto;
use crate::render::for_each_entity_in_files;
use crate::render::links::{EntityKind, LinkContext};
use crate::summary::render_md;
use mdbook_summary::{Link, SummaryItem};
use std::path::Path;

/// Target markdown path for a package page under the current layout.
pub fn package_target(links: &LinkContext, layout: Layout, package: &str) -> std::path::PathBuf {
    match layout {
        Layout::Package => links.package_page_rel(package),
        Layout::Entity | Layout::Split => links.package_index_rel(package),
    }
}

/// Entity sub-links for SUMMARY in link/SUMMARY order.
pub fn entity_summary_items(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
    links: &LinkContext,
    summary_from: &Path,
) -> Vec<SummaryItem> {
    let mut out = Vec::new();
    for_each_entity_in_files(files, |kind, name, _, _, _| {
        let p = links.entity_path(package, kind, name).expect("entity");
        let path = render_md::link_path_for_summary(summary_from, p);
        let title = match kind {
            EntityKind::Message => format!("Message {name}"),
            EntityKind::Enum => format!("Enum {name}"),
            EntityKind::Service => format!("Service {name}"),
        };
        out.push(SummaryItem::Link(Link::new(title, path)));
    });
    out
}

/// One flat-layout chapter link for a package (no companion tree).
pub fn flat_package_chapter(
    package: &str,
    files: &[(&str, &FileDescriptorProto)],
    layout: Layout,
    package_only: bool,
    links: &LinkContext,
    summary_from: &Path,
) -> SummaryItem {
    match layout {
        Layout::Package => {
            let target = links.package_page_rel(package);
            let path = render_md::link_path_for_summary(summary_from, &target);
            SummaryItem::Link(Link::new(package, path))
        }
        Layout::Split | Layout::Entity if package_only => {
            let target = links.package_index_rel(package);
            let path = render_md::link_path_for_summary(summary_from, &target);
            SummaryItem::Link(Link::new(package, path))
        }
        Layout::Split => {
            let target = links.package_index_rel(package);
            let path = render_md::link_path_for_summary(summary_from, &target);
            let mut link = Link::new(package, path);
            link.nested_items = entity_summary_items(package, files, links, summary_from);
            SummaryItem::Link(link)
        }
        Layout::Entity => {
            let mut link = Link::default();
            link.name = package.to_string();
            link.location = None;
            link.nested_items = entity_summary_items(package, files, links, summary_from);
            SummaryItem::Link(link)
        }
    }
}
