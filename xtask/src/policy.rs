//! Repository-owned seams for check order, image, and coverage.

use crate::{CheckStep, ImageEngine};

pub const CHECK_ORDER: [CheckStep; 10] = [
    CheckStep::Toolchain,
    CheckStep::BufLint,
    CheckStep::Fmt,
    CheckStep::Check,
    CheckStep::Clippy,
    CheckStep::Test,
    CheckStep::BuildPlugin,
    CheckStep::HighlightRust,
    CheckStep::BookInit,
    CheckStep::BookLinks,
];

pub const IMAGE_AUTO_ORDER: [ImageEngine; 2] = [ImageEngine::Docker, ImageEngine::Buildah];
pub const IMAGE_FILE: &str = "Dockerfile";
pub const IMAGE_CONTEXT: &str = ".";
pub const IMAGE_TAG: &str = "protobuf-mdbook:local";
pub const IMAGE_PLATFORM: &str = "linux/amd64";
pub const IMAGE_SMOKE_PROGRAM: &str = "/protoc-gen-mdbook";
pub const IMAGE_SMOKE_ARGS: &[&str] = &["--version"];

pub const LLVM_COV_REPORT: &str = "target/coverage/llvm-cov/html/index.html";
pub const TARPAULIN_REPORT: &str = "target/coverage/tarpaulin/tarpaulin-report.html";
