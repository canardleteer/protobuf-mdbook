//! Plugin options from `request.parameter`.

mod table;

use anyhow::{Result, bail};
use std::path::PathBuf;

use table::{
    apply_parsed, build_options_from_cli as build_from_cli, is_markdown_only, parse_token,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    #[default]
    Package,
    Entity,
    Split,
}

/// How to rewrite HTML-like `<tag>` tokens in leading-comment prose for mdBook.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EscapeTags {
    #[default]
    Off,
    Backticks,
    Entities,
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
    pub ignore_git: bool,
    pub layout: Layout,
    /// Extra directories to resolve `FileDescriptorProto.name` when reading source for spans.
    pub proto_search_path: Vec<PathBuf>,
    /// Rewrite HTML-like angle-bracket tokens in leading-comment prose.
    pub escape_tags: EscapeTags,
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
            ignore_git: true,
            layout: Layout::Package,
            proto_search_path: Vec::new(),
            escape_tags: EscapeTags::Off,
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

    /// Whether init wires protobuf highlighting (`mdbook-protobuf-highlight` in `book.toml`).
    pub fn proto_highlight(&self) -> bool {
        self.init && !self.no_proto_highlight
    }

    /// Whether init wires CEL highlighting (`mdbook-protobuf-highlight` in `book.toml`).
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

    for token in param.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let parsed = parse_token(token)?;
        if is_markdown_only(&parsed) {
            saw_markdown_only = true;
        }
        apply_parsed(&mut opts, parsed);
    }

    validate_options(&opts, saw_markdown_only)?;
    Ok(opts)
}

/// Init-only and deprecated-option checks after all tokens are applied.
pub fn validate_options(opts: &Options, saw_markdown_only: bool) -> Result<()> {
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
        if !opts.ignore_git {
            bail!("`ignore=none` is only valid with `init`");
        }
    }

    Ok(())
}

pub use crate::paths::join_book_root;

/// Fields collected from native `protobuf-mdbook` clap flags.
#[derive(Clone, Debug)]
pub struct CliOptionsInput {
    pub init: bool,
    pub summary: bool,
    pub layout: Layout,
    pub book_root: Option<String>,
    pub book: Option<String>,
    pub markdown_root: Option<String>,
    pub summary_path: Option<String>,
    pub title: Option<String>,
    pub ignore_git: bool,
    pub no_proto_highlight: bool,
    pub no_cel_highlight: bool,
    pub no_proto_markdown: bool,
    pub escape_tags: EscapeTags,
}

impl Default for CliOptionsInput {
    fn default() -> Self {
        Self {
            init: false,
            summary: false,
            layout: Layout::Package,
            book_root: None,
            book: None,
            markdown_root: None,
            summary_path: None,
            title: None,
            ignore_git: true,
            no_proto_highlight: false,
            no_cel_highlight: false,
            no_proto_markdown: false,
            escape_tags: EscapeTags::Off,
        }
    }
}

/// Build validated [`Options`] from native CLI flags.
pub fn build_options_from_cli(input: CliOptionsInput) -> Result<Options> {
    let opts = build_from_cli(input)?;
    validate_options(&opts, false)?;
    Ok(opts)
}

/// Map parsed options to native `protobuf-mdbook` argv tokens (excluding `-o`, `-I`, inputs).
pub use table::options_to_cli_args;

/// Parse a protoc-style option string and emit equivalent CLI argv tokens.
pub fn parameter_to_cli_args(param: &str) -> Result<Vec<String>> {
    let opts = parse_parameter(&Some(param.to_string()))?;
    Ok(options_to_cli_args(&opts))
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

    #[test]
    fn escape_tags_defaults_off() {
        let o = parse_parameter(&None).unwrap();
        assert_eq!(o.escape_tags, EscapeTags::Off);
    }

    #[test]
    fn parses_escape_tags_bare_flag() {
        let o = parse_parameter(&Some("escape_tags".into())).unwrap();
        assert_eq!(o.escape_tags, EscapeTags::Backticks);
    }

    #[test]
    fn parses_escape_tags_backticks_and_entities() {
        let backticks = parse_parameter(&Some("escape_tags=backticks".into())).unwrap();
        assert_eq!(backticks.escape_tags, EscapeTags::Backticks);
        let entities = parse_parameter(&Some("escape_tags=entities".into())).unwrap();
        assert_eq!(entities.escape_tags, EscapeTags::Entities);
    }

    #[test]
    fn unknown_escape_tags_value_errors() {
        let err = parse_parameter(&Some("escape_tags=foo".into())).unwrap_err();
        assert!(format!("{err:#}").contains("escape_tags"));
    }

    #[test]
    fn parameter_to_cli_args_round_trip() {
        let param = "init,layout=entity,title=My Book,no_proto_highlight";
        let args = parameter_to_cli_args(param).unwrap();
        assert!(args.contains(&"--init".to_string()));
        assert!(args.contains(&"--layout".to_string()));
        assert!(args.contains(&"entity".to_string()));
        assert!(args.contains(&"--title".to_string()));
        assert!(args.contains(&"My Book".to_string()));
        assert!(args.contains(&"--no-proto-highlight".to_string()));
    }

    #[test]
    fn options_to_cli_args_escape_tags_entities() {
        let opts = parse_parameter(&Some("escape_tags=entities".into())).unwrap();
        let args = options_to_cli_args(&opts);
        assert_eq!(args, vec!["--escape-tags", "entities"]);
    }

    #[test]
    fn build_options_from_cli_matches_parse_parameter() {
        let input = CliOptionsInput {
            summary: true,
            layout: Layout::Entity,
            markdown_root: Some("content/api".into()),
            ..CliOptionsInput::default()
        };
        let from_cli = build_options_from_cli(input).unwrap();
        let from_param = parse_parameter(&Some(
            "summary,layout=entity,markdown_root=content/api".into(),
        ))
        .unwrap();
        assert_eq!(from_cli.summary, from_param.summary);
        assert_eq!(from_cli.layout, from_param.layout);
        assert_eq!(from_cli.markdown_root, from_param.markdown_root);
        assert_eq!(
            from_cli.explicit_markdown_root,
            from_param.explicit_markdown_root
        );
    }
}
