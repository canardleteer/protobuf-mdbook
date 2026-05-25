//! mdBook init via `mdbook-driver::BookBuilder` in a temp directory.

use crate::highlight::{HighlightConfig, THEME_CSS_REL, configure_book_toml, theme_css_content};
use crate::options::{Options, join_book_root};
use anyhow::{Context, Result};
use mdbook_core::config::Config;
use mdbook_driver::MDBook;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

pub const DEFAULT_BOOK_TITLE: &str = "Protobuf documentation";

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

fn highlight_config_from_opts(opts: &Options) -> HighlightConfig {
    HighlightConfig {
        protobuf: opts.proto_highlight(),
        cel: opts.cel_highlight(),
    }
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
                .map(|s| (join_book_root(book_root, &k), s))
        })
        .collect();

    configure_init_highlighting(opts, book_root, &mut out);

    for (path, content) in docs {
        out.insert(path.clone(), content.clone());
    }

    let readme_path = join_book_root(book_root, "README.md");
    out.insert(readme_path, init_readme_content(opts));

    let mut pairs: Vec<_> = out.into_iter().collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

fn configure_init_highlighting(opts: &Options, book_root: &str, out: &mut HashMap<String, String>) {
    let book_key = join_book_root(book_root, "book.toml");
    let config = highlight_config_from_opts(opts);
    if config.protobuf || config.cel {
        let css_key = join_book_root(book_root, THEME_CSS_REL);
        out.insert(css_key, theme_css_content().to_string());
    }
    match out.get_mut(&book_key) {
        Some(book) => {
            if let Err(e) = configure_book_toml(book, config) {
                eprintln!("protobuf-mdbook: warning: book.toml highlight config: {e:#}");
            }
        }
        None => {
            let mut book = String::new();
            if let Err(e) = configure_book_toml(&mut book, config) {
                eprintln!("protobuf-mdbook: warning: book.toml highlight config: {e:#}");
            } else {
                out.insert(book_key, book);
            }
        }
    }
}

/// Starter README beside `book.toml` (init mode only).
pub fn init_readme_content(opts: &Options) -> String {
    let mdbook_ver = crate::mdbook_version();
    let highlight_section = if opts.proto_highlight() || opts.cel_highlight() {
        let mut lines = vec![
            "## Syntax highlighting".to_string(),
            String::new(),
            "Init configures the **mdbook-protobuf-highlight** preprocessor in `book.toml`. \
             At `mdbook build`, ` ```protobuf ` and ` ```cel ` fences become pre-highlighted \
             HTML compatible with mdBook themes. See the **protobuf-mdbook** repository README \
             (**Syntax highlighting**) for toggles and standalone install."
                .to_string(),
        ];
        if opts.proto_highlight() {
            lines.push(
                "- **Protobuf:** ` ```protobuf ` fences (disable at init: `no_proto_highlight`)."
                    .into(),
            );
        }
        if opts.cel_highlight() {
            lines.push(
                "- **CEL:** ` ```cel ` fences for Protovalidate rules (disable at init: \
                 `no_cel_highlight`)."
                    .into(),
            );
        }
        lines.push(String::new());
        lines.join("\n")
    } else {
        r#"## Syntax highlighting

Disabled at init (`no_proto_highlight` and/or `no_cel_highlight`). Run
`mdbook-protobuf-highlight install` or see the plugin repository README.

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

   All three binaries report the pinned mdBook version:

   ```bash
   protobuf-mdbook --version
   # or: protoc-gen-mdbook --version
   # or: mdbook-protobuf-highlight --version
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
    use super::{init_readme_content, merge_init_files};
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
        assert!(readme.contains("mdbook-protobuf-highlight") || readme.contains("Protobuf"));
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
    fn init_configures_preprocessor_in_book_toml() {
        let opts = parse_parameter(&Some("init".into())).unwrap();
        let init_files =
            HashMap::from([("book.toml".to_string(), b"[book]\ntitle = \"t\"\n".to_vec())]);
        let pairs = merge_init_files(&opts, ".", init_files, &[]);
        let book = pairs
            .iter()
            .find(|(p, _)| p == "book.toml")
            .map(|(_, c)| c.as_str())
            .expect("book.toml");
        assert!(book.contains("[preprocessor.protobuf-highlight]"));
        assert!(book.contains("mdbook-protobuf-highlight"));
        assert!(
            !pairs
                .iter()
                .any(|(p, _)| p.contains("highlight-protobuf.js"))
        );
    }

    #[test]
    fn init_no_proto_highlight_sets_toml_flag() {
        let opts = parse_parameter(&Some("init,no_proto_highlight".into())).unwrap();
        let init_files =
            HashMap::from([("book.toml".to_string(), b"[book]\ntitle = \"t\"\n".to_vec())]);
        let pairs = merge_init_files(&opts, ".", init_files, &[]);
        let book = pairs
            .iter()
            .find(|(p, _)| p == "book.toml")
            .unwrap()
            .1
            .as_str();
        assert!(book.contains("protobuf = false"));
        assert!(book.contains("cel = true"));
    }

    #[test]
    fn init_both_disabled_omits_preprocessor() {
        let opts =
            parse_parameter(&Some("init,no_proto_highlight,no_cel_highlight".into())).unwrap();
        let init_files =
            HashMap::from([("book.toml".to_string(), b"[book]\ntitle = \"t\"\n".to_vec())]);
        let pairs = merge_init_files(&opts, ".", init_files, &[]);
        let book = pairs
            .iter()
            .find(|(p, _)| p == "book.toml")
            .unwrap()
            .1
            .as_str();
        assert!(!book.contains("[preprocessor.protobuf-highlight]"));
    }
}
