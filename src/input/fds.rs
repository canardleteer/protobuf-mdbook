//! Descriptor set loading and stdin request decoding.

use crate::input::ResolvedInput;
use crate::plugin_api::{CodeGeneratorRequest, FileDescriptorProto, decode_descriptor_message};
use anyhow::{Context, Result};
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use std::io::Read;
use std::path::Path;

pub fn read_request_stdin() -> Result<ResolvedInput> {
    let mut stdin = Vec::new();
    std::io::stdin()
        .read_to_end(&mut stdin)
        .context("read CodeGeneratorRequest from stdin")?;
    let req = decode_descriptor_message::<CodeGeneratorRequest>(&stdin)
        .map_err(|e| anyhow::anyhow!("decode CodeGeneratorRequest: {e}"))?;
    Ok(ResolvedInput {
        proto_file: req.proto_file,
        file_to_generate: req.file_to_generate,
        proto_search_paths: Vec::new(),
    })
}

pub fn load_descriptor_set(path: &Path) -> Result<(Vec<FileDescriptorProto>, Vec<String>)> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read descriptor set {}", path.display()))?;
    let set = decode_descriptor_message::<FileDescriptorSet>(&bytes)
        .map_err(|e| anyhow::anyhow!("decode FileDescriptorSet {}: {e}", path.display()))?;
    let files = set.file;
    let names: Vec<String> = files.iter().filter_map(|f| f.name.clone()).collect();
    Ok((files, names))
}
