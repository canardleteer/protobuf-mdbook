//! Load mdBook `book.toml` via `mdbook-core` to infer plugin output paths.

use crate::options::Options;
use anyhow::{Context, Result, bail};
use mdbook_core::config::Config;
use std::path::{Path, PathBuf};

/// Resolve a `book=` argument to the directory containing `book.toml`.
pub fn resolve_book_root(book_arg: &str) -> Result<PathBuf> {
    let path = PathBuf::from(book_arg.trim());
    if path.file_name().and_then(|n| n.to_str()) == Some("book.toml") {
        Ok(path
            .parent()
            .context("book.toml path has no parent")?
            .to_path_buf())
    } else {
        Ok(path)
    }
}

/// Load `book.toml` from a book root directory.
pub fn load_mdbook_config(book_root: &Path) -> Result<Config> {
    let config_path = book_root.join("book.toml");
    Config::from_disk(&config_path).with_context(|| format!("load {}", config_path.display()))
}

/// Paths inferred from mdBook `[book] src` (SUMMARY is always `{src}/SUMMARY.md` in mdBook).
pub fn inferred_paths(src: &Path) -> (String, String) {
    let src = src.to_string_lossy().replace('\\', "/");
    let src = src.trim_matches('/');
    let summary_path = format!("{src}/SUMMARY.md");
    let markdown_root = format!("{src}/packages");
    (markdown_root, summary_path)
}

/// Apply `book=` / `mdbook_out=` after base option parsing.
pub fn apply_book_config(opts: &mut Options) -> Result<()> {
    let Some(book_arg) = opts.book.as_deref() else {
        return Ok(());
    };

    if opts.init {
        return Ok(());
    }

    let book_root = resolve_book_root(book_arg)?;
    let config_path = book_root.join("book.toml");
    if !config_path.is_file() {
        bail!(
            "`book={book_arg}` but {} does not exist; run with `init` to scaffold or fix the path",
            config_path.display()
        );
    }

    let config = load_mdbook_config(&book_root)?;
    let (markdown_root, summary_path) = inferred_paths(&config.book.src);

    if !opts.explicit_markdown_root {
        opts.markdown_root = markdown_root;
    }
    if !opts.explicit_summary_path {
        opts.summary_path = summary_path;
    }
    if !opts.explicit_book_root {
        opts.book_root = ".".into();
    }

    if let Some(mdbook_out) = opts.mdbook_out.as_deref() {
        validate_mdbook_out(mdbook_out, &book_root)?;
    }

    Ok(())
}

fn validate_mdbook_out(mdbook_out: &str, book_root: &Path) -> Result<()> {
    let out = PathBuf::from(mdbook_out);
    if paths_equal(&out, book_root) {
        return Ok(());
    }
    eprintln!(
        "protoc-gen-mdbook: warning: `mdbook_out={}` does not match book root `{}`; \
         point `--mdbook_out` at the book root or omit `book=`",
        out.display(),
        book_root.display()
    );
    Ok(())
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Markdown root directory for an on-disk book (for xtask / tooling).
pub fn markdown_root_dir(book_root: &Path) -> Result<PathBuf> {
    let config = load_mdbook_config(book_root)?;
    let (markdown_root, _) = inferred_paths(&config.book.src);
    Ok(book_root.join(markdown_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::parse_parameter;

    #[test]
    fn inferred_paths_default_src() {
        let (md, sum) = inferred_paths(Path::new("src"));
        assert_eq!(md, "src/packages");
        assert_eq!(sum, "src/SUMMARY.md");
    }

    #[test]
    fn inferred_paths_custom_src() {
        let (md, sum) = inferred_paths(Path::new("content"));
        assert_eq!(md, "content/packages");
        assert_eq!(sum, "content/SUMMARY.md");
    }

    #[test]
    fn resolve_book_root_from_toml_file() {
        let root = resolve_book_root("/tmp/my-book/book.toml").unwrap();
        assert_eq!(root, PathBuf::from("/tmp/my-book"));
    }

    #[test]
    fn apply_infers_paths_when_book_set() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("book.toml"),
            "[book]\ntitle = \"T\"\nsrc = \"content\"\n",
        )
        .unwrap();
        let book = dir.path().to_string_lossy().to_string();
        let mut opts = parse_parameter(&Some(format!("book={book}"))).unwrap();
        apply_book_config(&mut opts).unwrap();
        assert_eq!(opts.markdown_root, "content/packages");
        assert_eq!(opts.summary_path, "content/SUMMARY.md");
    }
}
