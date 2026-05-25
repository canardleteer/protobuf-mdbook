//! `protobuf-mdbook` library — generate mdBook trees from protobuf descriptors.

#![forbid(unsafe_code)]

mod plugin_api;

pub mod book_config;
pub mod examples;
pub mod highlight;
pub mod init;
pub mod input;
pub mod link_check;
pub mod options;
pub mod paths;
pub mod proto_deps;
pub mod proto_markdown;
pub mod render;
pub mod runner;
pub mod summary;

use crate::plugin_api::{
    CodeGeneratorRequest, CodeGeneratorResponse, CodeGeneratorResponseFile, FileDescriptorProto,
};
use anyhow::{Context, Result, bail};
use buffa::Message;
use options::{Options, parse_parameter};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::book_config::apply_book_config;

/// Inputs for documentation generation (plugin request or CLI-resolved descriptors).
#[derive(Clone, Debug)]
pub struct GenerateInput {
    pub proto_file: Vec<FileDescriptorProto>,
    pub file_to_generate: Vec<String>,
    /// Protoc plugin options (`request.parameter` / `--mdbook_opt=`).
    pub parameter: Option<String>,
    /// Pre-parsed options from the `protobuf-mdbook` CLI (takes precedence over `parameter`).
    pub options: Option<Options>,
    /// CLI-resolved search roots; protoc plugin leaves empty and uses `proto_path=` in parameter.
    pub proto_search_paths: Vec<PathBuf>,
}

impl From<CodeGeneratorRequest> for GenerateInput {
    fn from(req: CodeGeneratorRequest) -> Self {
        Self {
            proto_file: req.proto_file,
            file_to_generate: req.file_to_generate,
            parameter: req.parameter,
            options: None,
            proto_search_paths: Vec::new(),
        }
    }
}

/// mdBook version compiled into this plugin (`mdbook-core` pin in root `Cargo.toml`).
pub fn mdbook_version() -> &'static str {
    mdbook_core::MDBOOK_VERSION
}

/// Generate output files from descriptors and plugin options.
pub fn generate_from_input(input: &GenerateInput) -> Result<Vec<(String, String)>> {
    if input.file_to_generate.is_empty() {
        bail!("file_to_generate is empty");
    }

    let mut opts = match &input.options {
        Some(o) => o.clone(),
        None => parse_parameter(&input.parameter)?,
    };
    apply_book_config(&mut opts)?;
    if !input.proto_search_paths.is_empty() {
        opts.proto_search_path = input.proto_search_paths.clone();
    }

    let by_package = render::packages_map(&input.proto_file, &input.file_to_generate);
    let links = render::build_link_context(&by_package, &opts);
    let mut source = render::source::SourceCache::new(opts.proto_search_paths());
    let docs = render::render_all(&by_package, &opts, &links, &mut source);

    let mut file_map: HashMap<String, String> = HashMap::new();
    for doc in &docs {
        file_map.insert(doc.path.clone(), doc.content.clone());
    }

    let companions =
        proto_markdown::discover_companion_docs(&input.proto_file, &input.file_to_generate, &opts)?;
    for (path, content) in proto_markdown::read_companion_files(&companions, &opts)? {
        file_map.insert(path, content);
    }

    if let Some((path, content)) = summary::render_summary(&by_package, &opts, &links, &companions)
    {
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

    Ok(pairs)
}

/// Write generated `(relative_path, content)` pairs under `out_root`.
pub fn write_generated_files(out_root: &Path, pairs: &[(String, String)]) -> Result<()> {
    for (rel, content) in pairs {
        let path = out_root.join(rel.trim_start_matches('/'));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

/// Decode request bytes, generate documentation, encode response.
pub fn generate(request_bytes: &[u8]) -> Result<Vec<u8>> {
    let req = CodeGeneratorRequest::decode_from_slice(request_bytes)
        .map_err(|e| anyhow::anyhow!("decode CodeGeneratorRequest: {e}"))?;

    let pairs = generate_from_input(&req.into())?;

    let files: Vec<CodeGeneratorResponseFile> = pairs
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
