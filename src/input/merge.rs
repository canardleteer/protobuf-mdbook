//! Merge descriptor sets and filter generation targets.

use crate::plugin_api::FileDescriptorProto;
use std::collections::BTreeMap;

pub fn merge_proto_files(into: &mut Vec<FileDescriptorProto>, files: Vec<FileDescriptorProto>) {
    let mut by_name: BTreeMap<String, FileDescriptorProto> = into
        .drain(..)
        .filter_map(|f| f.name.clone().map(|n| (n, f)))
        .collect();
    for f in files {
        if let Some(name) = f.name.clone() {
            by_name.insert(name, f);
        }
    }
    *into = by_name.into_values().collect();
}

pub fn filter_file_to_generate(
    proto_file: &[FileDescriptorProto],
    wanted: &[String],
) -> Vec<String> {
    let known: BTreeMap<_, _> = proto_file
        .iter()
        .filter_map(|f| f.name.as_deref().map(|n| (n, ())))
        .collect();
    wanted
        .iter()
        .filter(|n| known.contains_key(n.as_str()))
        .cloned()
        .collect()
}
