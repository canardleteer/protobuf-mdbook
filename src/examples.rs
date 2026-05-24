//! Shared helpers for `examples/proto/` used by integration tests and xtask.

use std::path::PathBuf;

/// Canonical example `.proto` inputs (excludes vendored `buf/validate/validate.proto`).
pub const EXAMPLE_PROTO_INPUTS: &[&str] = &[
    "acme/example/v1/echo.proto",
    "acme/example/v1/gateway.proto",
    "acme/example/v2/types.proto",
    "acme/example/v2/catalog.proto",
    "acme/example/v2/services.proto",
    "acme/example/v3alpha1/types.proto",
    "acme/example/v3alpha1/pipeline.proto",
    "acme/example/v3alpha1/services.proto",
];

/// Path to the Buf module under `examples/proto/`.
pub fn examples_proto_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/proto")
}

/// Flags for building comma-separated `--mdbook_opt` strings.
#[derive(Clone, Debug, Default)]
pub struct MdbookOptFlags {
    pub summary: bool,
    pub init: bool,
    /// When set, append `book=` and `mdbook_out=` for guided book refresh.
    pub with_book_paths: Option<(String, String)>,
    /// Append `proto_path=.` (integration tests on examples/proto).
    pub include_proto_path: bool,
}

/// Comma-separated plugin options for example runs.
pub fn format_mdbook_opt(layout: &str, extra_opt: &str, flags: MdbookOptFlags) -> String {
    let mut parts = Vec::new();
    if flags.init {
        parts.push("init".to_string());
    }
    parts.push(format!("layout={layout}"));
    if flags.summary {
        parts.push("summary".into());
    }
    if flags.include_proto_path {
        parts.push("proto_path=.".into());
    }
    if let Some((book, mdbook_out)) = flags.with_book_paths {
        parts.push(format!("book={book}"));
        parts.push(format!("mdbook_out={mdbook_out}"));
    }
    if !extra_opt.is_empty() {
        parts.push(extra_opt.to_string());
    }
    parts.join(",")
}

/// Comma-separated plugin options for example runs (`layout=…`, `proto_path=.`).
pub fn format_examples_mdbook_opt(layout: &str, extra_opt: &str) -> String {
    format_mdbook_opt(
        layout,
        extra_opt,
        MdbookOptFlags {
            include_proto_path: true,
            ..MdbookOptFlags::default()
        },
    )
}
