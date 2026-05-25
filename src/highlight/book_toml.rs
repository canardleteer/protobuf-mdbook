//! `book.toml` preprocessor wiring for init and `mdbook-protobuf-highlight install`.

use crate::highlight::markdown::HighlightConfig;
use anyhow::{Context, Result, bail};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, Value};

pub const PREPROCESSOR_NAME: &str = "protobuf-highlight";
pub const PREPROCESSOR_COMMAND: &str = "mdbook-protobuf-highlight";

/// Relative path under the book root for highlight layout CSS (no HLJS panel on `<pre>`).
pub const THEME_CSS_REL: &str = "theme/protobuf-highlight.css";

const HIGHLIGHT_COMMENT_BEGIN: &str = "# --- protobuf-mdbook: syntax highlighting";

/// CSS shipped with init / `mdbook-protobuf-highlight install` (padding only; colors from mdBook `highlight.css`).
pub fn theme_css_content() -> &'static str {
    include_str!("../../assets/theme/protobuf-highlight.css")
}

/// Patch or append preprocessor config in `book.toml` content.
pub fn configure_book_toml(book_toml: &mut String, config: HighlightConfig) -> Result<()> {
    let mut doc = book_toml
        .parse::<DocumentMut>()
        .context("parse book.toml")?;
    if config.protobuf || config.cel {
        ensure_preprocessor(&mut doc, config)?;
        ensure_theme_css(&mut doc)?;
    } else {
        remove_preprocessor(&mut doc);
    }
    *book_toml = doc.to_string();
    if !book_toml.contains(HIGHLIGHT_COMMENT_BEGIN) {
        append_highlight_comment(book_toml, config.protobuf || config.cel);
    }
    Ok(())
}

/// Write preprocessor config to `book.toml` on disk (install subcommand).
pub fn install_book_toml(book_root: &Path, config: HighlightConfig) -> Result<()> {
    let path = book_root.join("book.toml");
    if !path.is_file() {
        bail!("missing book.toml at {}", path.display());
    }
    if config.protobuf || config.cel {
        write_theme_css_file(book_root)?;
    }
    let mut content = std::fs::read_to_string(&path).context("read book.toml")?;
    configure_book_toml(&mut content, config)?;
    std::fs::write(&path, &content).context("write book.toml")?;
    Ok(())
}

/// Write `theme/protobuf-highlight.css` under the book root.
pub fn write_theme_css_file(book_root: &Path) -> Result<()> {
    let path = book_root.join(THEME_CSS_REL);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&path, theme_css_content())
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn ensure_preprocessor(doc: &mut DocumentMut, config: HighlightConfig) -> Result<()> {
    let preprocessor = doc
        .entry("preprocessor")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("preprocessor table")?;
    let section = preprocessor
        .entry(PREPROCESSOR_NAME)
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("preprocessor.protobuf-highlight table")?;
    section.insert("command", Item::Value(Value::from(PREPROCESSOR_COMMAND)));
    section.insert("protobuf", Item::Value(Value::from(config.protobuf)));
    section.insert("cel", Item::Value(Value::from(config.cel)));
    Ok(())
}

fn ensure_theme_css(doc: &mut DocumentMut) -> Result<()> {
    let output = doc
        .entry("output")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("output table")?;
    let html = output
        .entry("html")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("output.html table")?;
    let path = THEME_CSS_REL;
    let mut append = true;
    if let Some(Item::Value(Value::Array(arr))) = html.get("additional-css") {
        append = !arr.iter().any(|v| v.as_str().is_some_and(|s| s == path));
    }
    if append {
        let arr = html
            .entry("additional-css")
            .or_insert(Item::Value(Value::Array(toml_edit::Array::new())))
            .as_array_mut()
            .context("additional-css array")?;
        arr.push(path);
    }
    Ok(())
}

fn remove_preprocessor(doc: &mut DocumentMut) {
    if let Some(preprocessor) = doc.get_mut("preprocessor").and_then(Item::as_table_mut) {
        preprocessor.remove(PREPROCESSOR_NAME);
        if preprocessor.is_empty() {
            doc.as_table_mut().remove("preprocessor");
        }
    }
}

fn append_highlight_comment(book_toml: &mut String, enabled: bool) {
    if book_toml.contains(HIGHLIGHT_COMMENT_BEGIN) {
        return;
    }
    let comment = if enabled {
        r#"
# --- protobuf-mdbook: syntax highlighting (enabled at init) ---
# Build-time highlighting via mdbook-protobuf-highlight preprocessor.
# Generated pages use ```protobuf fences; Protovalidate message-level CEL also
# emits ```cel fences (or is split from protobuf fences at mdbook build).
# Toggle in [preprocessor.protobuf-highlight]: protobuf = false / cel = false
# Disable entirely: no_proto_highlight and no_cel_highlight at init.
# --- end protobuf-mdbook syntax highlighting ---
"#
    } else {
        r#"
# --- protobuf-mdbook: syntax highlighting (disabled at init) ---
# Run mdbook-protobuf-highlight install or re-init without no_*_highlight flags.
# --- end protobuf-mdbook syntax highlighting ---
"#
    };
    if !book_toml.ends_with('\n') {
        book_toml.push('\n');
    }
    book_toml.push_str(comment);
}

/// Read preprocessor toggles from mdBook config (defaults true when section present).
pub fn config_from_mdbook(ctx: &mdbook_preprocessor::PreprocessorContext) -> HighlightConfig {
    let protobuf = ctx
        .config
        .get::<bool>(&format!("preprocessor.{PREPROCESSOR_NAME}.protobuf"))
        .ok()
        .flatten()
        .unwrap_or(true);
    let cel = ctx
        .config
        .get::<bool>(&format!("preprocessor.{PREPROCESSOR_NAME}.cel"))
        .ok()
        .flatten()
        .unwrap_or(true);
    HighlightConfig { protobuf, cel }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_preprocessor_section() {
        let mut book = "[book]\ntitle = \"t\"\n".to_string();
        configure_book_toml(
            &mut book,
            HighlightConfig {
                protobuf: true,
                cel: true,
            },
        )
        .unwrap();
        assert!(book.contains("[preprocessor.protobuf-highlight]"));
        assert!(book.contains("command = \"mdbook-protobuf-highlight\""));
        assert!(book.contains("protobuf = true"));
        assert!(book.contains("cel = true"));
        assert!(book.contains("theme/protobuf-highlight.css"));
    }

    #[test]
    fn disables_proto_only() {
        let mut book = "[book]\ntitle = \"t\"\n".to_string();
        configure_book_toml(
            &mut book,
            HighlightConfig {
                protobuf: false,
                cel: true,
            },
        )
        .unwrap();
        assert!(book.contains("protobuf = false"));
        assert!(book.contains("cel = true"));
    }

    #[test]
    fn removes_preprocessor_when_both_disabled() {
        let mut book = "[book]\ntitle = \"t\"\n".to_string();
        configure_book_toml(
            &mut book,
            HighlightConfig {
                protobuf: false,
                cel: false,
            },
        )
        .unwrap();
        assert!(!book.contains("[preprocessor.protobuf-highlight]"));
    }
}
