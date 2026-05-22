//! Read `.proto` sources and slice declarations via `SourceCodeInfo` spans.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cache of loaded proto sources for span-based synthesis.
pub struct SourceCache {
    roots: Vec<PathBuf>,
    files: HashMap<String, String>,
}

impl SourceCache {
    pub fn new(roots: impl IntoIterator<Item = PathBuf>) -> Self {
        Self {
            roots: roots.into_iter().collect(),
            files: HashMap::new(),
        }
    }

    pub fn load(&mut self, file_name: &str) -> Option<&str> {
        if self.files.contains_key(file_name) {
            return self.files.get(file_name).map(|s| s.as_str());
        }
        let path = self.resolve_path(file_name)?;
        let text = std::fs::read_to_string(&path).ok()?;
        self.files.insert(file_name.to_string(), text);
        self.files.get(file_name).map(|s| s.as_str())
    }

    fn resolve_path(&self, file_name: &str) -> Option<PathBuf> {
        let direct = Path::new(file_name);
        if direct.is_file() {
            return Some(direct.to_path_buf());
        }
        for root in &self.roots {
            let candidate = root.join(file_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}

/// Extract source text for a `SourceCodeInfo.location.span` (0-based line/column).
pub fn extract_span_snippet(source: &str, span: &[i32]) -> Option<String> {
    if span.len() < 3 {
        return None;
    }
    let lines: Vec<&str> = source.split('\n').collect();
    let (start_line, start_col, end_line, end_col) = match span {
        [sl, sc, ec] => (*sl, *sc, *sl, *ec),
        [sl, sc, el, ec] => (*sl, *sc, *el, *ec),
        _ => return None,
    };
    let start_line = usize::try_from(start_line).ok()?;
    let end_line = usize::try_from(end_line).ok()?;
    let start_col = usize::try_from(start_col).ok()?;
    let end_col = usize::try_from(end_col).ok()?;
    if start_line >= lines.len() || end_line >= lines.len() {
        return None;
    }
    if start_line == end_line {
        let line = lines[start_line];
        let end = end_col.min(line.len());
        if start_col > end {
            return None;
        }
        return Some(line[start_col..end].to_string());
    }
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate().take(end_line + 1).skip(start_line) {
        if i == start_line {
            out.push_str(&line[start_col.min(line.len())..]);
        } else if i == end_line {
            out.push_str(&line[..end_col.min(line.len())]);
        } else {
            out.push_str(line);
        }
        if i != end_line {
            out.push('\n');
        }
    }
    Some(out)
}

pub fn push_indented_lines(out: &mut String, text: &str, indent: &str) {
    for line in text.lines() {
        out.push_str(indent);
        out.push_str(line);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_multiline_field() {
        let src = "message M {\n  string id = 1 [\n    (ext).foo = true\n  ];\n}\n";
        // lines 0-based: line 1 cols 2..end for field
        let got = extract_span_snippet(src, &[1, 2, 3, 4]).expect("snippet");
        assert!(got.contains("(ext).foo"));
    }
}
