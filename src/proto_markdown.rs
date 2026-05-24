//! Discover and copy hand-written `.md` beside included `.proto` files.

use crate::options::Options;
use crate::plugin_api::FileDescriptorProto;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// A companion markdown file copied into `{markdown_root}/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanionDoc {
    /// Path relative to `markdown_root` (e.g. `acme.example.v1.README.md`).
    pub output_rel: String,
    /// Link title for SUMMARY (first `#` heading or stem).
    pub title: String,
    /// Source directory relative to corpus root (`acme/example/v1`).
    pub source_dir: PathBuf,
    pub stem: String,
    /// Absolute path resolved at discovery (used when copying bytes).
    pub source_path: PathBuf,
}

/// Search roots for companion discovery; defaults to `"."` when opts list is empty.
fn search_roots(opts: &Options) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = opts.proto_search_paths().collect();
    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }
    roots
}

/// Discover companion markdown for `file_to_generate` protos.
pub fn discover_companion_docs(
    proto_files: &[FileDescriptorProto],
    file_to_generate: &[String],
    opts: &Options,
) -> Result<Vec<CompanionDoc>> {
    if opts.no_proto_markdown {
        return Ok(Vec::new());
    }

    let roots = search_roots(opts);
    let proto_dirs = collect_proto_dirs(proto_files, file_to_generate, &roots)?;
    if proto_dirs.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = BTreeMap::new();

    for proto_dir in &proto_dirs {
        let mut dir = proto_dir.clone();
        loop {
            if !dir.as_os_str().is_empty() {
                collect_md_in_dir(&dir, &roots, &mut seen)?;
            }
            if dir.as_os_str().is_empty() {
                break;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    Ok(seen.into_values().collect())
}

/// Read companion files from disk; returns `(output_path, content)` pairs for the response map.
pub fn read_companion_files(
    docs: &[CompanionDoc],
    opts: &Options,
) -> Result<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(docs.len());
    for doc in docs {
        let content = std::fs::read_to_string(&doc.source_path)
            .with_context(|| format!("read companion markdown {}", doc.source_path.display()))?;
        let path = opts.output_path(&format!("{}/{}", opts.markdown_root, doc.output_rel));
        out.push((path, content));
    }
    Ok(out)
}

fn collect_proto_dirs(
    proto_files: &[FileDescriptorProto],
    file_to_generate: &[String],
    search_roots: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for name in file_to_generate {
        let rel = Path::new(name);
        if rel.components().any(|c| matches!(c, Component::ParentDir)) {
            continue;
        }
        let parent = rel.parent().unwrap_or(Path::new("")).to_path_buf();
        let _file = proto_files
            .iter()
            .find(|f| f.name.as_deref() == Some(name.as_str()));
        let resolved = resolve_proto_dir(&parent, search_roots);
        dirs.push(resolved.unwrap_or(parent));
    }
    dirs.sort();
    dirs.dedup();
    Ok(dirs)
}

fn resolve_proto_dir(rel_dir: &Path, search_roots: &[PathBuf]) -> Option<PathBuf> {
    for root in search_roots {
        let candidate = root.join(rel_dir);
        if candidate.is_dir() {
            return Some(normalize_rel_dir(rel_dir));
        }
    }
    Some(normalize_rel_dir(rel_dir))
}

fn normalize_rel_dir(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_owned()),
            _ => None,
        })
        .collect()
}

fn collect_md_in_dir(
    dir: &Path,
    search_roots: &[PathBuf],
    seen: &mut BTreeMap<String, CompanionDoc>,
) -> Result<()> {
    let fs_dir = search_roots
        .iter()
        .map(|r| r.join(dir))
        .find(|p| p.is_dir());
    let Some(fs_dir) = fs_dir else {
        return Ok(());
    };

    for entry in std::fs::read_dir(&fs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem.starts_with('.') {
            continue;
        }
        let rel_dir = normalize_rel_dir(dir);
        let output_rel = companion_output_name(&rel_dir, stem);
        if seen.contains_key(&output_rel) {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let title = title_from_markdown(stem, &content);
        seen.insert(
            output_rel.clone(),
            CompanionDoc {
                output_rel,
                title,
                source_dir: rel_dir,
                stem: stem.to_string(),
                source_path: path,
            },
        );
    }
    Ok(())
}

/// Dot-separated module path implied by a companion output filename.
///
/// `acme.example.v1.README.md` with stem `README` → `acme.example.v1`.
pub fn module_path_from_companion_output(output_rel: &str, stem: &str) -> Option<String> {
    let base = output_rel.strip_suffix(".md")?;
    let suffix = format!(".{stem}");
    base.strip_suffix(&suffix).map(str::to_string)
}

pub fn companion_output_name(rel_dir: &Path, stem: &str) -> String {
    let dot_path = rel_dir
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(".");
    if dot_path.is_empty() {
        format!("{stem}.md")
    } else {
        format!("{dot_path}.{stem}.md")
    }
}

fn title_from_markdown(stem: &str, content: &str) -> String {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('#') {
            let title = rest.trim_start_matches('#').trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    humanize_stem(stem)
}

fn humanize_stem(stem: &str) -> String {
    if stem.eq_ignore_ascii_case("readme") {
        return "README".to_string();
    }
    stem.replace(['-', '_'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn module_path_from_companion_output_parses() {
        assert_eq!(
            module_path_from_companion_output("acme.example.v1.README.md", "README"),
            Some("acme.example.v1".into())
        );
        assert_eq!(
            module_path_from_companion_output("acme.README.md", "README"),
            Some("acme".into())
        );
    }

    #[test]
    fn companion_output_name_dots() {
        let dir = Path::new("acme/example/v1");
        assert_eq!(
            companion_output_name(dir, "README"),
            "acme.example.v1.README.md"
        );
    }

    #[test]
    fn discovers_intermediate_and_leaf_companions() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("acme/example/v1")).unwrap();
        fs::create_dir_all(root.join("acme/example/v2")).unwrap();
        fs::write(root.join("acme/README.md"), "# Acme\n").unwrap();
        fs::write(root.join("acme/example/README.md"), "# Example\n").unwrap();
        fs::write(root.join("acme/example/v1/README.md"), "# V1\n").unwrap();
        fs::write(root.join("acme/example/v1/MOVING-TO-V2.md"), "# Moving\n").unwrap();

        let opts = Options {
            proto_search_path: vec![root.to_path_buf()],
            ..Options::default()
        };

        let proto_files = vec![];
        let inputs = vec![
            "acme/example/v1/echo.proto".into(),
            "acme/example/v2/types.proto".into(),
        ];
        let docs = discover_companion_docs(&proto_files, &inputs, &opts).unwrap();
        let names: Vec<_> = docs.iter().map(|d| d.output_rel.as_str()).collect();
        assert!(names.contains(&"acme.README.md"));
        assert!(names.contains(&"acme.example.README.md"));
        assert!(names.contains(&"acme.example.v1.README.md"));
        assert!(names.contains(&"acme.example.v1.MOVING-TO-V2.md"));
    }

    #[test]
    fn partial_inputs_skip_other_branch() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("a/b/c/d/e/f/g/h/v1")).unwrap();
        fs::create_dir_all(root.join("a/b/c/d/e/f/g/h/v2")).unwrap();
        fs::write(root.join("a/b/NOTES.md"), "# Notes\n").unwrap();
        fs::write(root.join("a/b/c/d/e/f/g/h/v2/more-notes.md"), "# More\n").unwrap();

        let opts = Options {
            proto_search_path: vec![root.to_path_buf()],
            ..Options::default()
        };

        let inputs = vec!["a/b/c/d/e/f/g/h/v1/stuff.proto".into()];
        let docs = discover_companion_docs(&[], &inputs, &opts).unwrap();
        let names: Vec<_> = docs.iter().map(|d| d.output_rel.as_str()).collect();
        assert!(names.contains(&"a.b.NOTES.md"));
        assert!(!names.iter().any(|n| n.contains("more-notes")));
    }
}
