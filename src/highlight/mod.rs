//! Build-time syntax highlighting for protobuf and CEL mdBook fences.

mod book_toml;
mod cel;
mod cel_split;
mod engine;
mod html;
mod markdown;
mod protobuf;

pub use book_toml::{
    PREPROCESSOR_COMMAND, PREPROCESSOR_NAME, THEME_CSS_REL, config_from_mdbook,
    configure_book_toml, install_book_toml, theme_css_content, write_theme_css_file,
};
pub use cel_split::split_message_cel_blocks;
pub use html::normalize_newlines;
pub use markdown::{HighlightConfig, highlight_source, transform_chapter};
