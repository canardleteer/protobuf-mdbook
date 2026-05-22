//! `SourceCodeInfo` comment lookup.

use crate::plugin_api::FileDescriptorProto;
use buffa_descriptor::generated::descriptor::source_code_info::Location;
use std::collections::HashMap;

/// Field numbers in parent descriptor messages (see `descriptor.proto`).
pub mod path {
    pub const FILE_MESSAGE: i32 = 4;
    pub const FILE_ENUM: i32 = 5;
    pub const FILE_SERVICE: i32 = 6;
    pub const MSG_FIELD: i32 = 2;
    pub const MSG_ONEOF: i32 = 8;
    pub const MSG_OPTIONS: i32 = 7;
    pub const ENUM_VALUE: i32 = 2;
    pub const SVC_METHOD: i32 = 2;
}

pub struct CommentIndex<'a> {
    by_path: HashMap<Vec<i32>, &'a Location>,
}

impl<'a> CommentIndex<'a> {
    pub fn from_file(file: &'a FileDescriptorProto) -> Self {
        let mut by_path = HashMap::new();
        if let Some(info) = file.source_code_info.as_option() {
            for loc in &info.location {
                if !loc.path.is_empty() {
                    by_path.insert(loc.path.clone(), loc);
                }
            }
        }
        Self { by_path }
    }

    pub fn leading(&self, path: &[i32]) -> Option<&str> {
        self.by_path
            .get(path)
            .and_then(|l| l.leading_comments.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    pub fn span_snippet(&self, source: &str, path: &[i32]) -> Option<String> {
        let loc = self.by_path.get(path)?;
        if loc.span.is_empty() {
            return None;
        }
        crate::render::source::extract_span_snippet(source, &loc.span)
    }

    /// Leading comments for a top-level message (`path`: `[4, message_index]`).
    pub fn leading_message(&self, mi: usize) -> Option<&str> {
        self.leading(&[path::FILE_MESSAGE, mi as i32])
    }

    /// Leading comments for a message field (`path`: `[4, mi, 2, fi]`).
    pub fn leading_message_field(&self, mi: usize, fi: usize) -> Option<&str> {
        self.leading(&[path::FILE_MESSAGE, mi as i32, path::MSG_FIELD, fi as i32])
    }

    /// Leading comments for an enum (`path`: `[5, enum_index]`).
    pub fn leading_enum(&self, ei: usize) -> Option<&str> {
        self.leading(&[path::FILE_ENUM, ei as i32])
    }

    /// Leading comments for an enum value (`path`: `[5, ei, 2, vi]`).
    pub fn leading_enum_value(&self, ei: usize, vi: usize) -> Option<&str> {
        self.leading(&[path::FILE_ENUM, ei as i32, path::ENUM_VALUE, vi as i32])
    }

    /// Leading comments for a service (`path`: `[6, service_index]`).
    pub fn leading_service(&self, si: usize) -> Option<&str> {
        self.leading(&[path::FILE_SERVICE, si as i32])
    }

    /// Leading comments for an RPC (`path`: `[6, si, 2, mi]` — `2` is the `method` field).
    pub fn leading_method(&self, si: usize, mi: usize) -> Option<&str> {
        self.leading(&[path::FILE_SERVICE, si as i32, path::SVC_METHOD, mi as i32])
    }

    pub fn trailing(&self, path: &[i32]) -> Option<&str> {
        self.by_path
            .get(path)
            .and_then(|l| l.trailing_comments.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

/// Package overview: first alphabetically earliest file in the package with a package comment.
pub fn package_overview(files: &[(&str, &FileDescriptorProto)]) -> Option<String> {
    let mut sorted = files.to_vec();
    sorted.sort_by_key(|(name, _)| *name);
    for (_, file) in sorted {
        let idx = CommentIndex::from_file(file);
        if let Some(c) = idx.leading(&[2]) {
            return Some(c.to_string());
        }
    }
    None
}
