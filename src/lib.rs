//! `protoc-gen-mdbook` library — generate mdBook trees from protobuf descriptors.

#![forbid(unsafe_code)]

mod plugin_api;

pub mod book_config;
pub mod init;
pub mod link_check;
pub mod options;
pub mod proto_deps;
pub mod proto_markdown;
pub mod render;
pub mod summary;

use crate::plugin_api::{CodeGeneratorRequest, CodeGeneratorResponse, CodeGeneratorResponseFile};
use anyhow::{Result, bail};
use buffa::Message;
use options::parse_parameter;
use std::collections::HashMap;

use crate::book_config::apply_book_config;

/// mdBook version compiled into this plugin (`mdbook-core` pin in root `Cargo.toml`).
pub fn mdbook_version() -> &'static str {
    mdbook_core::MDBOOK_VERSION
}

/// Decode request bytes, generate documentation, encode response.
pub fn generate(request_bytes: &[u8]) -> Result<Vec<u8>> {
    let req = CodeGeneratorRequest::decode_from_slice(request_bytes)
        .map_err(|e| anyhow::anyhow!("decode CodeGeneratorRequest: {e}"))?;

    if req.file_to_generate.is_empty() {
        bail!("file_to_generate is empty");
    }

    let mut opts = parse_parameter(&req.parameter)?;
    apply_book_config(&mut opts)?;

    let by_package = render::packages_map(&req.proto_file, &req.file_to_generate);
    let links = render::build_link_context(&by_package, &opts);
    let mut source = render::source::SourceCache::new(opts.proto_search_paths());
    let docs = render::render_all(
        &req.proto_file,
        &req.file_to_generate,
        &opts,
        &links,
        &mut source,
    );

    let mut file_map: HashMap<String, String> = HashMap::new();
    for doc in &docs {
        file_map.insert(doc.path.clone(), doc.content.clone());
    }

    let companions =
        proto_markdown::discover_companion_docs(&req.proto_file, &req.file_to_generate, &opts)?;
    for (path, content) in proto_markdown::read_companion_files(&companions, &opts)? {
        file_map.insert(path, content);
    }

    if let Some((path, content)) = summary::render_summary(
        &req.proto_file,
        &req.file_to_generate,
        &opts,
        &links,
        &companions,
    ) {
        file_map.insert(path, content);
    }

    let pairs = if opts.init {
        let init_files = init::scaffold_init_tree(&opts)?;
        let doc_pairs: Vec<_> = file_map.into_iter().collect();
        init::merge_init_files(&opts, &opts.book_root, init_files, &doc_pairs)
    } else {
        let mut pairs: Vec<_> = file_map.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    };

    let file_map: HashMap<String, String> = pairs.into_iter().collect();

    let files: Vec<CodeGeneratorResponseFile> = file_map
        .into_iter()
        .map(|(name, content)| CodeGeneratorResponseFile {
            name: Some(name),
            insertion_point: None,
            content: Some(content),
            ..Default::default()
        })
        .collect();

    // `FEATURE_PROTO3_OPTIONAL` — required when inputs use proto3 `optional` (e.g. Buf registry APIs).
    const FEATURE_PROTO3_OPTIONAL: u64 = 1;

    let resp = CodeGeneratorResponse {
        error: None,
        supported_features: Some(FEATURE_PROTO3_OPTIONAL),
        file: files,
        ..Default::default()
    };

    Ok(resp.encode_to_vec())
}
