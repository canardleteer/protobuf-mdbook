//! Leading-comment prose formatting for mdBook output.

use crate::options::EscapeTags;

/// Format proto leading-comment text for markdown prose (outside code fences).
pub fn format_markdown_doc(comment: &str, mode: EscapeTags) -> String {
    match mode {
        EscapeTags::Off => comment.to_string(),
        EscapeTags::Backticks | EscapeTags::Entities => escape_htmlish_tags(comment, mode),
    }
}

fn escape_htmlish_tags(input: &str, mode: EscapeTags) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_backticks = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            in_backticks = !in_backticks;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '<'
            && !in_backticks
            && !preceded_by_entity_lt(&chars, i)
            && let Some(tag_len) = htmlish_tag_len(&chars, i)
        {
            let tag: String = chars[i..i + tag_len].iter().collect();
            match mode {
                EscapeTags::Backticks => {
                    out.push('`');
                    out.push_str(&tag);
                    out.push('`');
                }
                EscapeTags::Entities => {
                    for ch in tag.chars() {
                        match ch {
                            '<' => out.push_str("&lt;"),
                            '>' => out.push_str("&gt;"),
                            other => out.push(other),
                        }
                    }
                }
                EscapeTags::Off => unreachable!(),
            }
            i += tag_len;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn preceded_by_entity_lt(chars: &[char], lt: usize) -> bool {
    lt >= 3 && chars[lt - 3] == '&' && chars[lt - 2] == 'l' && chars[lt - 1] == 't'
}

/// Length of an HTML-like tag starting at `<`, or `None` if not a tag.
fn htmlish_tag_len(chars: &[char], lt: usize) -> Option<usize> {
    if chars.get(lt).copied() != Some('<') {
        return None;
    }
    let rest: String = chars[lt + 1..].iter().collect();
    if rest.starts_with("http://") || rest.starts_with("https://") {
        return None;
    }

    let mut i = lt + 1;
    if chars.get(i).copied() == Some('/') {
        i += 1;
    }
    let name_start = i;
    if !is_tag_name_start(chars.get(i).copied()?) {
        return None;
    }
    i += 1;
    while i < chars.len() && is_tag_name_char(chars[i]) {
        i += 1;
    }
    if i == name_start + 1 {
        return None;
    }

    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    if chars.get(i).copied() == Some('/') {
        i += 1;
        if chars.get(i).copied() == Some('>') {
            return Some(i - lt + 1);
        }
        return None;
    }

    if chars.get(i).copied() != Some('>') {
        while i < chars.len() {
            let ch = chars[i];
            if ch == '>' {
                return Some(i - lt + 1);
            }
            if ch == '"' || ch == '\'' {
                let quote = ch;
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                if i >= chars.len() {
                    return None;
                }
            }
            i += 1;
        }
        return None;
    }

    Some(i - lt + 1)
}

fn is_tag_name_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn is_tag_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == ':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_is_identity() {
        let s = "from <environment> CoT";
        assert_eq!(format_markdown_doc(s, EscapeTags::Off), s);
    }

    #[test]
    fn backticks_wraps_meshtastic_like_tags() {
        let s = "from <environment> CoT";
        assert_eq!(
            format_markdown_doc(s, EscapeTags::Backticks),
            "from `<environment>` CoT"
        );
        let s = "Maps to ATAK's <zMist> inside <zMistsMap>.";
        assert_eq!(
            format_markdown_doc(s, EscapeTags::Backticks),
            "Maps to ATAK's `<zMist>` inside `<zMistsMap>`."
        );
    }

    #[test]
    fn entities_mode() {
        let s = "from <environment> CoT";
        assert_eq!(
            format_markdown_doc(s, EscapeTags::Entities),
            "from &lt;environment&gt; CoT"
        );
    }

    #[test]
    fn preserves_markdown_constructs() {
        let s = "**bold** and [link](url) with `<environment>` already coded";
        assert_eq!(format_markdown_doc(s, EscapeTags::Backticks), s);
    }

    #[test]
    fn preserves_autolink() {
        let s = "see <https://example.com> for details";
        assert_eq!(format_markdown_doc(s, EscapeTags::Backticks), s);
    }

    #[test]
    fn skips_already_entity_escaped() {
        let s = "from &lt;environment&gt; CoT";
        assert_eq!(format_markdown_doc(s, EscapeTags::Backticks), s);
    }

    #[test]
    fn backticks_inside_inline_code_untouched() {
        let s = "use `<environment>` in proto comments";
        assert_eq!(format_markdown_doc(s, EscapeTags::Backticks), s);
    }

    #[test]
    fn closing_tag() {
        let s = "end </sensor> here";
        assert_eq!(
            format_markdown_doc(s, EscapeTags::Backticks),
            "end `</sensor>` here"
        );
    }

    #[test]
    fn self_closing_tag() {
        let s = "node <br/> break";
        assert_eq!(
            format_markdown_doc(s, EscapeTags::Backticks),
            "node `<br/>` break"
        );
    }
}
