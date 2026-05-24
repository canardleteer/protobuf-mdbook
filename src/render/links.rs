//! Layout-aware paths and cross-reference links.

use crate::options::{Layout, join_book_root};
use crate::plugin_api::codegen::split_proto_type_name;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Message,
    Enum,
    Service,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityRef {
    pub package: String,
    pub kind: EntityKind,
    pub name: String,
}

pub struct LinkContext {
    pub layout: Layout,
    pub book_root: String,
    pub markdown_root: String,
    entities: HashMap<EntityRef, PathBuf>,
}

impl LinkContext {
    pub fn new(
        layout: Layout,
        book_root: &str,
        markdown_root: &str,
        entities: impl IntoIterator<Item = EntityRef>,
    ) -> Self {
        let book_root = book_root.to_string();
        let markdown_root = markdown_root.to_string();
        let mut map = HashMap::new();
        for e in entities {
            let path = entity_rel_path(layout, &markdown_root, &e);
            map.insert(e, path);
        }
        Self {
            layout,
            book_root,
            markdown_root,
            entities: map,
        }
    }

    pub fn package_page_rel(&self, package: &str) -> PathBuf {
        package_page_rel(&self.markdown_root, package)
    }

    pub fn package_index_rel(&self, package: &str) -> PathBuf {
        package_index_rel(self.layout, &self.markdown_root, package)
    }

    pub fn entity_path(&self, package: &str, kind: EntityKind, name: &str) -> Option<&PathBuf> {
        self.entities.get(&EntityRef {
            package: package.to_string(),
            kind,
            name: name.to_string(),
        })
    }

    pub fn link_from(&self, from: &Path, package: &str, kind: EntityKind, name: &str) -> String {
        let Some(target) = self.entity_path(package, kind, name) else {
            return format!("`.{package}.{name}`");
        };
        match self.layout {
            Layout::Package => self.package_layout_link(from, target, name),
            Layout::Entity | Layout::Split => self.file_link(from, target),
        }
    }

    pub fn link_type(&self, from: &Path, fqn: &str) -> String {
        let Some((pkg, ident)) = split_proto_type_name(fqn) else {
            return format!("`{fqn}`");
        };
        if self.entity_path(pkg, EntityKind::Message, ident).is_some() {
            return self.link_from(from, pkg, EntityKind::Message, ident);
        }
        if self.entity_path(pkg, EntityKind::Enum, ident).is_some() {
            return self.link_from(from, pkg, EntityKind::Enum, ident);
        }
        format!("`{fqn}`")
    }

    fn file_link(&self, from: &Path, target: &Path) -> String {
        let from_dir = from.parent().unwrap_or(Path::new(""));
        let rel = relative_path(from_dir, target);
        let label = target.file_stem().unwrap_or_default().to_string_lossy();
        format!("[{label}]({rel})")
    }

    fn package_layout_link(&self, from: &Path, target: &Path, name: &str) -> String {
        if from == target {
            format!("[{name}](#{})", heading_slug(name))
        } else {
            let from_dir = from.parent().unwrap_or(Path::new(""));
            let rel = relative_path(from_dir, target);
            format!("[{name}]({rel}#{})", heading_slug(name))
        }
    }

    pub fn summary_link(&self, from: &Path, target: &Path, title: &str) -> String {
        let from_dir = from.parent().unwrap_or(Path::new(""));
        let rel = relative_path(from_dir, target);
        format!("[{title}]({rel})")
    }
}

pub fn with_book_root(book_root: &str, path: &Path) -> String {
    join_book_root(book_root, &path.to_string_lossy())
}

pub fn package_page_rel(markdown_root: &str, package: &str) -> PathBuf {
    PathBuf::from(format!("{markdown_root}/{package}.md"))
}

pub fn package_index_rel(layout: Layout, markdown_root: &str, package: &str) -> PathBuf {
    match layout {
        Layout::Package => package_page_rel(markdown_root, package),
        Layout::Entity | Layout::Split => PathBuf::from(format!(
            "{markdown_root}/{}/index.md",
            package.replace('.', "/")
        )),
    }
}

fn entity_rel_path(layout: Layout, markdown_root: &str, e: &EntityRef) -> PathBuf {
    let pkg_file = e.package.replace('.', "/");
    match layout {
        Layout::Package => package_page_rel(markdown_root, &e.package),
        Layout::Entity | Layout::Split => PathBuf::from(match e.kind {
            EntityKind::Message => format!("{markdown_root}/{pkg_file}/messages/{}.md", e.name),
            EntityKind::Enum => format!("{markdown_root}/{pkg_file}/enums/{}.md", e.name),
            EntityKind::Service => format!("{markdown_root}/{pkg_file}/services/{}.md", e.name),
        }),
    }
}

/// Heading anchor id compatible with mdBook HTML output (`mdbook-html` `id_from_content`).
pub fn heading_slug(name: &str) -> String {
    id_from_content(name)
}

/// Assigns mdBook-style unique heading ids in document order (appends `-1`, `-2`, … on collision).
pub fn unique_heading_ids(titles: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut used = HashSet::new();
    titles
        .into_iter()
        .map(|title| {
            let base = id_from_content(title.as_ref());
            unique_id(&base, &mut used)
        })
        .collect()
}

fn id_from_content(content: &str) -> String {
    content
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

fn unique_id(id: &str, used: &mut HashSet<String>) -> String {
    if used.insert(id.to_string()) {
        return id.to_string();
    }
    let mut counter = 1u32;
    loop {
        let candidate = format!("{id}-{counter}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

fn relative_path(from_dir: &Path, target: &Path) -> String {
    relative_path_from_dir(from_dir, target)
}

/// Relative POSIX path from `from_dir` to `target` (mdBook link form).
pub fn relative_path_from_dir(from_dir: &Path, target: &Path) -> String {
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
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    } else {
        parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mdbook_slug_for_pascal_case_message() {
        assert_eq!(
            heading_slug("GetOrganizationsResponse"),
            "getorganizationsresponse"
        );
    }

    /// Parity with mdBook HTML heading ids (local copy of crate-private `id_from_content`).
    #[test]
    fn id_from_content_matches_mdbook_behavior() {
        let cases = [
            ("GetOrganizationsResponse", "getorganizationsresponse"),
            ("中文標題 CJK title", "中文標題-cjk-title"),
            ("_-_12345", "_-_12345"),
        ];
        for (input, expected) in cases {
            assert_eq!(id_from_content(input), expected, "input: {input:?}");
        }
    }

    #[test]
    fn relative_path_from_dir_cases() {
        use std::path::PathBuf;

        let cases = [
            (
                Path::new("src"),
                Path::new("src/packages/acme.md"),
                "packages/acme.md",
            ),
            (
                Path::new("src/packages"),
                Path::new("src/packages/acme.md"),
                "acme.md",
            ),
            (
                Path::new("content/api"),
                Path::new("src/packages/acme.md"),
                "../../src/packages/acme.md",
            ),
        ];
        for (from, target, expected) in cases {
            assert_eq!(
                relative_path_from_dir(from, target),
                expected,
                "from={from:?} target={target:?}"
            );
            let summary_from = PathBuf::from("src/SUMMARY.md");
            let from_dir = summary_from.parent().unwrap();
            if from == from_dir {
                let via_summary = PathBuf::from(relative_path_from_dir(from_dir, target));
                assert_eq!(
                    via_summary.to_string_lossy(),
                    expected,
                    "link_path_for_summary parity"
                );
            }
        }
    }
}
