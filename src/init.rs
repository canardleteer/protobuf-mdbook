//! mdBook init via `mdbook-driver::BookBuilder` in a temp directory.

use crate::options::{Options, join_book_root};
use anyhow::{Context, Result};
use mdbook_core::config::Config;
use mdbook_driver::MDBook;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

pub const DEFAULT_BOOK_TITLE: &str = "Protobuf documentation";

/// Vendored Highlight.js 10.1.1 protobuf grammar (`assets/highlightjs/protobuf-10.js`).
const PROTO_HIGHLIGHT_JS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/highlightjs/protobuf-10.js"
));

/// Repo-authored Highlight.js 10.1.1 CEL grammar (`assets/highlightjs/cel-10.js`).
const CEL_HIGHLIGHT_JS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/highlightjs/cel-10.js"
));

const SYNTAX_HIGHLIGHT_BEGIN: &str = "protobuf-mdbook: syntax highlight begin";
const SYNTAX_HIGHLIGHT_END: &str = "protobuf-mdbook: syntax highlight end";
const LEGACY_SYNTAX_HIGHLIGHT_BEGIN: &str = "protoc-gen-mdbook: syntax highlight begin";

fn index_has_syntax_highlight_block(index: &str) -> bool {
    index.contains(SYNTAX_HIGHLIGHT_BEGIN) || index.contains(LEGACY_SYNTAX_HIGHLIGHT_BEGIN)
}

fn book_toml_has_syntax_highlight_comment(book: &str) -> bool {
    book.contains("protobuf-mdbook: syntax highlighting")
        || book.contains("protoc-gen-mdbook: syntax highlighting")
}

/// Paths from mdBook init that should not appear in plugin output (replaced by generated docs).
const MDBOOK_DEFAULT_SUMMARY: &str = "src/SUMMARY.md";
const MDBOOK_DEFAULT_CHAPTER: &str = "src/chapter_1.md";

fn init_stub_paths(opts: &Options) -> Vec<String> {
    let mut paths = vec![
        MDBOOK_DEFAULT_SUMMARY.to_string(),
        MDBOOK_DEFAULT_CHAPTER.to_string(),
    ];
    if opts.summary_path != MDBOOK_DEFAULT_SUMMARY {
        paths.push(opts.summary_path.clone());
    }
    paths
}

const THEME_HIGHLIGHT_JS: &str = r#"<script src="{{ resource "highlight.js" }}"></script>"#;
const THEME_BOOK_JS: &str = r#"<script src="{{ resource "book.js" }}"></script>"#;

const BOOK_TOML_HIGHLIGHT_ENABLED: &str = r#"
# --- protobuf-mdbook: syntax highlighting (enabled at init) ---
# Generated API pages use ```protobuf fences; Protovalidate message-level CEL rules also
# emit ```cel fences. Init patches theme/index.hbs with inline <script> grammars after
# highlight.js and before book.js (mdBook does not bundle arbitrary theme/*.js via
# {{ resource }} — only inlined scripts in index.hbs are reliable).
#
# To disable: delete the "protobuf-mdbook: syntax highlight" block in theme/index.hbs and
# optional theme/highlight-*.js reference copies. On a future init:
#   no_proto_highlight — skip protobuf grammar only
#   no_cel_highlight   — skip CEL grammar only (protobuf can stay on)
# Re-init does not replace an existing highlight block when markers are already present.
# See the plugin repository README (Syntax highlighting) for custom themes and limitations.
# Attribution: assets/highlightjs/NOTICE (protobuf: BSD-3-Clause; cel: repo-authored).
# --- end protobuf-mdbook syntax highlighting ---
"#;

const BOOK_TOML_HIGHLIGHT_DISABLED: &str = r#"
# --- protobuf-mdbook: syntax highlighting (disabled at init) ---
# Pass init without no_proto_highlight / no_cel_highlight, or patch theme/index.hbs yourself.
# --- end protobuf-mdbook syntax highlighting ---
"#;

/// Inline Highlight.js grammars for `theme/index.hbs`.
fn syntax_highlight_index_hbs_snippet(opts: &Options) -> String {
    let mut parts = Vec::new();
    if opts.proto_highlight() {
        parts.push(PROTO_HIGHLIGHT_JS.trim());
    }
    if opts.cel_highlight() {
        parts.push(CEL_HIGHLIGHT_JS.trim());
    }
    let body = parts.join("\n");
    format!(
        "        <!-- {SYNTAX_HIGHLIGHT_BEGIN} -->\n        <script>\n{body}\n        </script>\n        <!-- {SYNTAX_HIGHLIGHT_END} -->\n"
    )
}

pub fn scaffold_init_tree(opts: &Options) -> Result<HashMap<String, Vec<u8>>> {
    let temp = tempfile::tempdir().context("tempdir for mdbook init")?;
    let root = temp.path();

    let mut cfg = Config::default();
    if let Some(title) = &opts.title {
        cfg.book.title = Some(title.clone());
    } else {
        cfg.book.title = Some(DEFAULT_BOOK_TITLE.into());
    }

    let mut builder = MDBook::init(root);
    if opts.ignore_git {
        builder.create_gitignore(true);
    }
    // Init always includes mdBook's default theme assets (same as `copy_theme(true)`).
    builder.copy_theme(true);
    builder.with_config(cfg);
    builder.build().context("BookBuilder::build")?;

    let mut files = read_tree(root)?;
    let stubs = init_stub_paths(opts);
    files.retain(|k, _| !stubs.iter().any(|stub| stub == k));
    Ok(files)
}

fn read_tree(root: &Path) -> Result<HashMap<String, Vec<u8>>> {
    let mut files = HashMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .context("strip_prefix")?
            .to_path_buf();
        let key = rel.to_string_lossy().replace('\\', "/");
        let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        files.insert(key, data);
    }
    Ok(files)
}

pub fn merge_init_files(
    opts: &Options,
    book_root: &str,
    init_files: HashMap<String, Vec<u8>>,
    docs: &[(String, String)],
) -> Vec<(String, String)> {
    let mut out: HashMap<String, String> = init_files
        .into_iter()
        .filter_map(|(k, v)| {
            String::from_utf8(v)
                .ok()
                .map(|s| (prefix_book_root(book_root, &k), s))
        })
        .collect();

    if opts.proto_highlight() || opts.cel_highlight() {
        inject_syntax_highlighting(opts, book_root, &mut out);
    } else {
        append_book_toml_highlight_comment(book_root, &mut out, false);
    }

    for (path, content) in docs {
        out.insert(path.clone(), content.clone());
    }

    let readme_path = join_book_root(book_root, "README.md");
    out.insert(readme_path, init_readme_content(opts));

    let mut pairs: Vec<_> = out.into_iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

fn inject_syntax_highlighting(opts: &Options, book_root: &str, out: &mut HashMap<String, String>) {
    if opts.proto_highlight() {
        let highlight_path = join_book_root(book_root, "theme/highlight-protobuf.js");
        out.insert(highlight_path, PROTO_HIGHLIGHT_JS.to_string());
    }
    if opts.cel_highlight() {
        let highlight_path = join_book_root(book_root, "theme/highlight-cel.js");
        out.insert(highlight_path, CEL_HIGHLIGHT_JS.to_string());
    }

    let index_key = join_book_root(book_root, "theme/index.hbs");
    if let Some(index) = out.get_mut(&index_key)
        && !index_has_syntax_highlight_block(index)
        && let Some(pos) = index.find(THEME_HIGHLIGHT_JS)
    {
        let insert_at = pos + THEME_HIGHLIGHT_JS.len();
        index.insert(insert_at, '\n');
        index.insert_str(insert_at + 1, &syntax_highlight_index_hbs_snippet(opts));
        debug_assert!(index.contains(THEME_BOOK_JS));
    }

    append_book_toml_highlight_comment(book_root, out, true);
}

fn append_book_toml_highlight_comment(
    book_root: &str,
    out: &mut HashMap<String, String>,
    enabled: bool,
) {
    let book_key = join_book_root(book_root, "book.toml");
    let comment = if enabled {
        BOOK_TOML_HIGHLIGHT_ENABLED
    } else {
        BOOK_TOML_HIGHLIGHT_DISABLED
    };
    match out.get_mut(&book_key) {
        Some(book) => {
            if !book_toml_has_syntax_highlight_comment(book) {
                book.push_str(comment);
            }
        }
        None => {
            out.insert(book_key, comment.trim_start().to_string());
        }
    }
}

fn prefix_book_root(book_root: &str, rel: &str) -> String {
    join_book_root(book_root, rel)
}

/// Starter README beside `book.toml` (init mode only).
pub fn init_readme_content(opts: &Options) -> String {
    let mdbook_ver = crate::mdbook_version();
    let highlight_section = if opts.proto_highlight() || opts.cel_highlight() {
        let mut lines = vec![
            "## Syntax highlighting".to_string(),
            String::new(),
            "Init patches `theme/index.hbs` with Highlight.js 10.1.1 grammars between \
             `highlight.js` and `book.js`. See the **protobuf-mdbook** repository README \
             (**Syntax highlighting**) for custom themes, re-init behavior, and CEL limitations."
                .to_string(),
        ];
        if opts.proto_highlight() {
            lines.push(
                "- **Protobuf:** ` ```protobuf ` fences (disable: `no_proto_highlight`).".into(),
            );
        }
        if opts.cel_highlight() {
            lines.push(
                "- **CEL:** ` ```cel ` fences for Protovalidate message-level rules (disable: \
                 `no_cel_highlight`)."
                    .into(),
            );
        }
        lines.push(String::new());
        lines.join("\n")
    } else {
        r#"## Syntax highlighting

Disabled at init (`no_proto_highlight` and/or `no_cel_highlight`). See the plugin repository
README for how to add grammars manually.

"#
        .to_string()
    };

    format!(
        r#"# Generated mdBook project

This file was created by **protobuf-mdbook** when you passed `init`. You can edit or delete it.

## Next steps

1. Customize `book.toml`, `{summary_path}`, themes, and preprocessors to taste.
2. API reference pages live under `{markdown_root}/` (set via `markdown_root=` if you relocate them).
3. Preview locally (install an **mdbook** CLI whose major.minor matches the plugin pin):

   ```bash
   mdbook serve
   mdbook build
   ```

   The generator reports its pinned mdBook version:

   ```bash
   protobuf-mdbook --version
   # or: protoc-gen-mdbook --version
   ```

   Expected pin: **{mdbook_ver}** (also declared in the crate `Cargo.toml`).

{highlight_section}## Diagrams (` ```mermaid ` fences)

If any generated pages include ` ```mermaid ` blocks (from protobuf comments), configure
[mdbook-mermaid](https://github.com/badboy/mdbook-mermaid) yourself in `book.toml`. Using
Mermaid in your protos is optional; rendering it in the book is your setup.

## Doc linting

- [rumdl](https://github.com/rvben/rumdl) — Markdown style
- [lychee](https://github.com/lycheeiver/lychee) — link checking

## Regenerating API pages

After the first `init` run, call the plugin **without** `init` so `book.toml`, this README,
your SUMMARY, and theme files are preserved. Only `{markdown_root}/**/*.md` are refreshed.
Use the same `markdown_root=` and `summary_path=` as your book layout when regenerating.
"#,
        summary_path = opts.summary_path,
        markdown_root = opts.markdown_root,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        SYNTAX_HIGHLIGHT_BEGIN, THEME_BOOK_JS, THEME_HIGHLIGHT_JS, init_readme_content,
        inject_syntax_highlighting,
    };
    use crate::options::parse_parameter;
    use std::collections::HashMap;

    #[test]
    fn init_readme_mentions_mermaid_rumdl_and_lychee() {
        let opts = parse_parameter(&Some("init".into())).unwrap();
        let readme = init_readme_content(&opts);
        assert!(readme.contains("```mermaid"));
        assert!(readme.contains("mdbook-mermaid"));
        assert!(readme.contains("rumdl"));
        assert!(readme.contains("lychee"));
        assert!(readme.contains("without") && readme.contains("init"));
        assert!(readme.contains("highlight-protobuf") || readme.contains("Protobuf"));
        assert!(readme.contains("CEL") || readme.contains("cel"));
        assert!(readme.contains("src/packages"));
    }

    #[test]
    fn init_readme_no_highlight_when_disabled() {
        let opts =
            parse_parameter(&Some("init,no_proto_highlight,no_cel_highlight".into())).unwrap();
        let readme = init_readme_content(&opts);
        assert!(readme.contains("no_proto_highlight"));
        assert!(readme.contains("no_cel_highlight"));
    }

    #[test]
    fn init_readme_reflects_custom_paths() {
        let opts = parse_parameter(&Some(
            "init,markdown_root=content/api,summary_path=content/SUMMARY.md".into(),
        ))
        .unwrap();
        let readme = init_readme_content(&opts);
        assert!(readme.contains("content/api"));
        assert!(readme.contains("content/SUMMARY.md"));
    }

    #[test]
    fn inject_skips_when_legacy_highlight_markers_present() {
        let opts = parse_parameter(&Some("init".into())).unwrap();
        let mut out = HashMap::from([(
            "theme/index.hbs".to_string(),
            format!(
                "head\n{THEME_HIGHLIGHT_JS}\n<!-- protoc-gen-mdbook: syntax highlight begin -->\n<!-- protoc-gen-mdbook: syntax highlight end -->\n{THEME_BOOK_JS}\n"
            ),
        )]);
        inject_syntax_highlighting(&opts, ".", &mut out);
        let index = out.get("theme/index.hbs").expect("index.hbs");
        assert!(index.contains("protoc-gen-mdbook: syntax highlight begin"));
        assert!(!index.contains("protobuf-mdbook: syntax highlight begin"));
    }

    #[test]
    fn inject_patches_index_hbs_with_inline_grammars() {
        let opts = parse_parameter(&Some("init".into())).unwrap();
        let mut out = HashMap::from([(
            "theme/index.hbs".to_string(),
            format!("head\n{THEME_HIGHLIGHT_JS}\n{THEME_BOOK_JS}\n"),
        )]);
        inject_syntax_highlighting(&opts, ".", &mut out);
        let index = out.get("theme/index.hbs").expect("index.hbs");
        assert!(index.contains(SYNTAX_HIGHLIGHT_BEGIN));
        assert!(index.contains("<script>"));
        assert!(index.contains("hljs.registerLanguage(\"protobuf\""));
        assert!(index.contains("hljs.registerLanguage(\"cel\""));
        assert!(!index.contains(r#"resource "highlight-protobuf.js""#));
        let hl = index.find(THEME_HIGHLIGHT_JS).expect("highlight.js");
        let marker = index.find(SYNTAX_HIGHLIGHT_BEGIN).expect("marker");
        let bk = index.find(THEME_BOOK_JS).expect("book.js");
        assert!(hl < marker && marker < bk);
        assert!(out.contains_key("theme/highlight-protobuf.js"));
        assert!(out.contains_key("theme/highlight-cel.js"));
    }

    #[test]
    fn inject_cel_only_when_no_proto_highlight() {
        let opts = parse_parameter(&Some("init,no_proto_highlight".into())).unwrap();
        let mut out = HashMap::from([(
            "theme/index.hbs".to_string(),
            format!("head\n{THEME_HIGHLIGHT_JS}\n{THEME_BOOK_JS}\n"),
        )]);
        inject_syntax_highlighting(&opts, ".", &mut out);
        let index = out.get("theme/index.hbs").expect("index.hbs");
        assert!(!index.contains("hljs.registerLanguage(\"protobuf\""));
        assert!(index.contains("hljs.registerLanguage(\"cel\""));
        assert!(!out.contains_key("theme/highlight-protobuf.js"));
        assert!(out.contains_key("theme/highlight-cel.js"));
    }

    #[test]
    fn inject_proto_only_when_no_cel_highlight() {
        let opts = parse_parameter(&Some("init,no_cel_highlight".into())).unwrap();
        let mut out = HashMap::from([(
            "theme/index.hbs".to_string(),
            format!("head\n{THEME_HIGHLIGHT_JS}\n{THEME_BOOK_JS}\n"),
        )]);
        inject_syntax_highlighting(&opts, ".", &mut out);
        let index = out.get("theme/index.hbs").expect("index.hbs");
        assert!(index.contains("hljs.registerLanguage(\"protobuf\""));
        assert!(!index.contains("hljs.registerLanguage(\"cel\""));
        assert!(out.contains_key("theme/highlight-protobuf.js"));
        assert!(!out.contains_key("theme/highlight-cel.js"));
    }
}
