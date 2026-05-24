//! Plugin options from `request.parameter`.

use anyhow::{Result, bail};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Package,
    Entity,
    Split,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub init: bool,
    pub summary: bool,
    pub no_proto_highlight: bool,
    pub no_cel_highlight: bool,
    pub no_proto_markdown: bool,
    pub book_root: String,
    /// Directory for generated API markdown, relative to `book_root` (default `src/packages`).
    pub markdown_root: String,
    /// Path to generated SUMMARY when `summary` or `init` (default `src/SUMMARY.md`).
    pub summary_path: String,
    /// Book root directory or path to `book.toml` (`book=`); loads paths via mdbook-core when set.
    pub book: Option<String>,
    /// Protoc output root for validation with `book=` (`mdbook_out=`).
    pub mdbook_out: Option<String>,
    pub(crate) explicit_book_root: bool,
    pub(crate) explicit_markdown_root: bool,
    pub(crate) explicit_summary_path: bool,
    pub title: Option<String>,
    pub theme: bool,
    pub ignore_git: bool,
    pub layout: Layout,
    /// Extra directories to resolve `FileDescriptorProto.name` when reading source for spans.
    pub proto_search_path: Vec<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            init: false,
            summary: false,
            no_proto_highlight: false,
            no_cel_highlight: false,
            no_proto_markdown: false,
            book_root: ".".into(),
            markdown_root: default_markdown_root().into(),
            summary_path: default_summary_path().into(),
            book: None,
            mdbook_out: None,
            explicit_book_root: false,
            explicit_markdown_root: false,
            explicit_summary_path: false,
            title: None,
            theme: false,
            ignore_git: true,
            layout: Layout::Package,
            proto_search_path: Vec::new(),
        }
    }
}

impl Options {
    pub fn proto_search_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.proto_search_path.iter().cloned()
    }

    pub fn render_summary(&self) -> bool {
        self.summary || self.init
    }

    pub fn package_only_summary(&self) -> bool {
        self.init
    }

    /// Whether init should patch `theme/index.hbs` with inline protobuf Highlight.js grammar.
    pub fn proto_highlight(&self) -> bool {
        self.init && !self.no_proto_highlight
    }

    /// Whether init should patch `theme/index.hbs` with inline CEL Highlight.js grammar.
    pub fn cel_highlight(&self) -> bool {
        self.init && !self.no_cel_highlight
    }

    /// Absolute output path for a plugin-relative file: `{book_root}/{rel}` under `--mdbook_out`.
    pub fn output_path(&self, rel: &str) -> String {
        join_book_root(&self.book_root, rel)
    }
}

pub fn default_markdown_root() -> &'static str {
    "src/packages"
}

pub fn default_summary_path() -> &'static str {
    "src/SUMMARY.md"
}

pub fn parse_parameter(parameter: &Option<String>) -> Result<Options> {
    let mut opts = Options::default();
    let mut saw_markdown_only = false;

    let Some(param) = parameter else {
        return Ok(opts);
    };

    for opt in param.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(v) = opt.strip_prefix("book_root=") {
            opts.book_root = normalize_book_root(v);
            opts.explicit_book_root = true;
        } else if let Some(v) = opt.strip_prefix("markdown_root=") {
            opts.markdown_root = normalize_rel_path(v, default_markdown_root())?;
            opts.explicit_markdown_root = true;
        } else if let Some(v) = opt.strip_prefix("summary_path=") {
            opts.summary_path = normalize_rel_path(v, default_summary_path())?;
            opts.explicit_summary_path = true;
        } else if let Some(v) = opt.strip_prefix("book=") {
            opts.book = Some(v.to_string());
        } else if let Some(v) = opt.strip_prefix("mdbook_out=") {
            opts.mdbook_out = Some(v.to_string());
        } else if let Some(v) = opt.strip_prefix("title=") {
            opts.title = Some(v.to_string());
        } else if let Some(v) = opt.strip_prefix("ignore=") {
            opts.ignore_git = match v {
                "git" => true,
                "none" => false,
                other => bail!("unknown ignore value {other:?}; use git or none"),
            };
        } else if let Some(v) = opt.strip_prefix("proto_path=") {
            opts.proto_search_path = v
                .split(':')
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect();
        } else if let Some(v) = opt.strip_prefix("layout=") {
            opts.layout = match v {
                "package" => Layout::Package,
                "entity" => Layout::Entity,
                "split" => Layout::Split,
                other => bail!("unknown layout {other:?}; use package, entity, or split"),
            };
        } else {
            match opt {
                "init" => opts.init = true,
                "summary" => opts.summary = true,
                "theme" => opts.theme = true,
                "no_proto_highlight" => opts.no_proto_highlight = true,
                "no_cel_highlight" => opts.no_cel_highlight = true,
                "no_proto_markdown" => opts.no_proto_markdown = true,
                "markdown_only" => {
                    saw_markdown_only = true;
                }
                other => bail!("unknown plugin option: {other:?}"),
            }
        }
    }

    if saw_markdown_only {
        eprintln!(
            "protobuf-mdbook: `markdown_only` is deprecated (default output is markdown-only); use `init` for a full mdBook project"
        );
    }

    if opts.no_proto_highlight && !opts.init {
        bail!("`no_proto_highlight` is only valid with `init`");
    }

    if opts.no_cel_highlight && !opts.init {
        bail!("`no_cel_highlight` is only valid with `init`");
    }

    if !opts.init {
        if opts.title.is_some() {
            bail!("`title` is only valid with `init`");
        }
        if opts.theme {
            bail!("`theme` is only valid with `init`");
        }
        if !opts.ignore_git {
            bail!("`ignore=none` is only valid with `init`");
        }
    }

    Ok(opts)
}

fn normalize_book_root(root: &str) -> String {
    normalize_rel_path(root, ".").expect("book_root validated")
}

fn normalize_rel_path(path: &str, default: &str) -> Result<String> {
    let path = path.trim().trim_matches('/');
    if path.is_empty() {
        return Ok(default.to_string());
    }
    if path.contains("..") {
        bail!("path must not contain `..`: {path:?}");
    }
    Ok(path.replace('\\', "/"))
}

pub fn join_book_root(book_root: &str, rel: &str) -> String {
    let rel = rel.trim_start_matches('/');
    if book_root == "." {
        rel.to_string()
    } else {
        format!("{book_root}/{rel}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_without_init_is_valid() {
        let o = parse_parameter(&Some("summary,layout=entity".into())).unwrap();
        assert!(!o.init);
        assert!(o.summary);
        assert_eq!(o.layout, Layout::Entity);
    }

    #[test]
    fn title_requires_init() {
        let err = parse_parameter(&Some("title=My Book".into())).unwrap_err();
        assert!(format!("{err:#}").contains("title"));
    }

    #[test]
    fn parses_init_and_layout() {
        let o = parse_parameter(&Some("init,layout=entity".into())).unwrap();
        assert!(o.init);
        assert_eq!(o.layout, Layout::Entity);
    }

    #[test]
    fn markdown_only_is_deprecated_noop() {
        let o = parse_parameter(&Some("markdown_only,summary".into())).unwrap();
        assert!(!o.init);
        assert!(o.summary);
    }

    #[test]
    fn no_proto_highlight_requires_init() {
        let err = parse_parameter(&Some("no_proto_highlight".into())).unwrap_err();
        assert!(format!("{err:#}").contains("no_proto_highlight"));
    }

    #[test]
    fn parses_no_proto_markdown() {
        let o = parse_parameter(&Some("no_proto_markdown,summary".into())).unwrap();
        assert!(o.no_proto_markdown);
        assert!(o.summary);
    }

    #[test]
    fn proto_highlight_default_on_for_init() {
        let o = parse_parameter(&Some("init".into())).unwrap();
        assert!(o.proto_highlight());
    }

    #[test]
    fn proto_highlight_off_when_flag_set() {
        let o = parse_parameter(&Some("init,no_proto_highlight".into())).unwrap();
        assert!(!o.proto_highlight());
    }

    #[test]
    fn cel_highlight_default_on_for_init() {
        let o = parse_parameter(&Some("init".into())).unwrap();
        assert!(o.cel_highlight());
    }

    #[test]
    fn cel_highlight_off_when_flag_set() {
        let o = parse_parameter(&Some("init,no_cel_highlight".into())).unwrap();
        assert!(!o.cel_highlight());
        assert!(o.proto_highlight());
    }

    #[test]
    fn no_cel_highlight_requires_init() {
        let err = parse_parameter(&Some("no_cel_highlight".into())).unwrap_err();
        assert!(format!("{err:#}").contains("no_cel_highlight"));
    }

    #[test]
    fn parses_markdown_root_and_summary_path() {
        let o = parse_parameter(&Some(
            "markdown_root=content/api,summary_path=content/SUMMARY.md".into(),
        ))
        .unwrap();
        assert_eq!(o.markdown_root, "content/api");
        assert_eq!(o.summary_path, "content/SUMMARY.md");
        assert_eq!(
            o.output_path("content/api/acme.example.v1.md"),
            "content/api/acme.example.v1.md"
        );
        assert_eq!(
            o.output_path("content/api/acme.example.v1.md"),
            join_book_root(".", "content/api/acme.example.v1.md")
        );
    }

    #[test]
    fn book_root_prefixes_output_paths() {
        let o = parse_parameter(&Some("book_root=docs".into())).unwrap();
        assert_eq!(
            o.output_path("src/packages/acme.example.v1.md"),
            "docs/src/packages/acme.example.v1.md"
        );
    }

    #[test]
    fn parses_book_and_mdbook_out() {
        let o = parse_parameter(&Some("book=./api-book,mdbook_out=./api-book".into())).unwrap();
        assert_eq!(o.book.as_deref(), Some("./api-book"));
        assert_eq!(o.mdbook_out.as_deref(), Some("./api-book"));
    }
}
