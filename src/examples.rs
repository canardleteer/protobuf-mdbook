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

/// Same list as [`EXAMPLE_PROTO_INPUTS`].
pub fn example_proto_inputs() -> &'static [&'static str] {
    EXAMPLE_PROTO_INPUTS
}

/// Comma-separated plugin options for example runs (`layout=…`, `proto_path=.`).
pub fn format_examples_mdbook_opt(layout: &str, extra_opt: &str) -> String {
    let mut opt = format!("layout={layout},proto_path=.");
    if !extra_opt.is_empty() {
        opt.push(',');
        opt.push_str(extra_opt);
    }
    opt
}
