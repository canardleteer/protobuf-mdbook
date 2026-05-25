//! Split Protovalidate message-level CEL from protobuf fence bodies.
//!
//! Extraction uses brace-depth scanning, not a full protobuf/CEL lexer. If an
//! `expression:` string literal contains `};` sequences, the split may truncate or
//! mis-bound the block.

const MESSAGE_CEL_PREFIX: &str = "option (buf.validate.message).cel";
const MESSAGE_CEL_EXPR_PREFIX: &str = "option (buf.validate.message).cel_expression";

/// Extract message-level Protovalidate CEL from a protobuf message body.
///
/// Returns the body with CEL options removed and zero or more CEL block strings
/// suitable for ` ```cel ` fences (inner rule content, trimmed).
pub fn split_message_cel_blocks(body: &str) -> (String, Vec<String>) {
    let mut cel_blocks = Vec::new();
    let mut out = String::new();
    let mut rest = body;
    while !rest.is_empty() {
        let Some((idx, kind)) = next_message_cel_start(rest) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..idx]);
        let tail = &rest[idx..];
        match kind {
            CelMatchKind::Block => {
                if let Some((block, consumed)) = extract_message_cel_block(tail) {
                    cel_blocks.push(block);
                    rest = &tail[consumed..];
                } else {
                    out.push_str(tail);
                    break;
                }
            }
            CelMatchKind::Expression => {
                if let Some((block, consumed)) = extract_message_cel_expression_line(tail) {
                    cel_blocks.push(block);
                    rest = &tail[consumed..];
                } else {
                    out.push_str(tail);
                    break;
                }
            }
        }
    }
    (out, cel_blocks)
}

enum CelMatchKind {
    Block,
    Expression,
}

fn next_message_cel_start(s: &str) -> Option<(usize, CelMatchKind)> {
    let mut candidates = Vec::new();
    if let Some(i) = s.find(MESSAGE_CEL_EXPR_PREFIX) {
        candidates.push((i, CelMatchKind::Expression));
    }
    let mut search_from = 0;
    while let Some(rel) = s[search_from..].find(MESSAGE_CEL_PREFIX) {
        let i = search_from + rel;
        let after = &s[i + MESSAGE_CEL_PREFIX.len()..];
        if after.starts_with(|c: char| c.is_whitespace() || c == '=') {
            candidates.push((i, CelMatchKind::Block));
            break;
        }
        search_from = i + MESSAGE_CEL_PREFIX.len();
    }
    candidates.into_iter().min_by_key(|(i, _)| *i)
}

fn extract_message_cel_block(s: &str) -> Option<(String, usize)> {
    let rest = s.trim_start();
    if !rest.starts_with(MESSAGE_CEL_PREFIX) {
        return None;
    }
    let after = rest[MESSAGE_CEL_PREFIX.len()..].trim_start();
    if !after.starts_with('=') {
        return None;
    }
    let after_eq = after[1..].trim_start();
    let brace_start = after_eq.find('{')?;
    let (block, rel_end) = extract_brace_block(&after_eq[brace_start..])?;
    let brace_in_rest = rest.find('{')?;
    let mut end_in_rest = brace_in_rest + rel_end;
    let tail = rest[end_in_rest..].trim_start();
    if tail.starts_with(';') {
        end_in_rest += 1;
    }
    let consumed = s.len() - rest.len() + end_in_rest;
    let inner = block
        .strip_prefix('{')
        .and_then(|b| b.strip_suffix('}'))
        .unwrap_or(block)
        .trim()
        .to_string();
    (!inner.is_empty()).then_some((inner, consumed))
}

fn extract_message_cel_expression_line(s: &str) -> Option<(String, usize)> {
    let line_end = s.find('\n').map(|i| i + 1).unwrap_or(s.len());
    let line = &s[..line_end];
    let block = try_parse_cel_expression_line(line)?;
    Some((block, line_end))
}

fn try_parse_cel_expression_line(line: &str) -> Option<String> {
    let rest = line.trim_start();
    if !rest.starts_with(MESSAGE_CEL_EXPR_PREFIX) {
        return None;
    }
    let after = rest[MESSAGE_CEL_EXPR_PREFIX.len()..].trim_start();
    if !after.starts_with('=') {
        return None;
    }
    let value = after[1..].trim_start();
    let expr = parse_quoted_or_bare(value)?;
    Some(format!("expression: {expr}"))
}

fn parse_quoted_or_bare(s: &str) -> Option<String> {
    let s = s.trim().trim_end_matches(';').trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return Some(s.to_string());
    }
    Some(format!("\"{s}\""))
}

/// From the first `{`, return the substring through its matching `}` and bytes consumed.
fn extract_brace_block(s: &str) -> Option<(&str, usize)> {
    if !s.starts_with('{') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut quote = '"';
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == quote {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                quote = ch;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[..=i], i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUMERIC_RANGE: &str = r#"message NumericRange {
  int64 min_inclusive = 1;
  int64 max_inclusive = 2;

  option (buf.validate.message).cel = {
    id: "numeric_range.min_lte_max"
    message: "min_inclusive must not exceed max_inclusive"
    expression: "this.min_inclusive <= this.max_inclusive"
  };
}
"#;

    #[test]
    fn splits_message_cel_from_body() {
        let (body, blocks) = split_message_cel_blocks(NUMERIC_RANGE);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("numeric_range.min_lte_max"));
        assert!(blocks[0].contains("expression:"));
        assert!(!body.contains("buf.validate.message).cel"));
        assert!(body.contains("min_inclusive = 1"));
    }

    #[test]
    fn leaves_field_level_cel_in_body() {
        let input = r#"message M {
  string x = 1 [(buf.validate.field).cel = {
    id: "x"
    expression: "this.x != ''"
  }];
}
"#;
        let (body, blocks) = split_message_cel_blocks(input);
        assert!(blocks.is_empty());
        assert!(body.contains("buf.validate.field).cel"));
    }

    #[test]
    fn splits_cel_expression_shorthand() {
        let input = r#"message M {
  option (buf.validate.message).cel_expression = "this.ok";
}
"#;
        let (body, blocks) = split_message_cel_blocks(input);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("expression:"));
        assert!(!body.contains("cel_expression"));
    }
}
