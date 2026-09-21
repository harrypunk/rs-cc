//! Greedy longest-match conversion ("maximal munch").

use crate::dict::Dict;

/// Scan `input` left to right, returning `(source_slice, matched_value)`
/// segments. `matched_value` is `None` for chars that passed through
/// unchanged. Always takes the longest key matching at the position.
fn scan<'a, 'd>(dict: &'d Dict, input: &'a str) -> Vec<(&'a str, Option<&'d str>)> {
    // Byte offset of every char boundary, plus end of string — lets us
    // slice by char count without ever splitting a UTF-8 codepoint.
    let bounds: Vec<usize> = input
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(input.len()))
        .collect();

    let mut segments = Vec::new();
    let mut pos = 0;
    while pos + 1 < bounds.len() {
        let remaining = bounds.len() - 1 - pos; // chars left
        let max = dict.max_key_len().min(remaining);
        let mut matched = None;
        for len in (1..=max).rev() {
            let key = &input[bounds[pos]..bounds[pos + len]];
            if let Some(value) = dict.lookup(key) {
                matched = Some((len, value));
                break;
            }
        }
        match matched {
            Some((len, value)) => {
                segments.push((&input[bounds[pos]..bounds[pos + len]], Some(value)));
                pos += len;
            }
            None => {
                segments.push((&input[bounds[pos]..bounds[pos + 1]], None));
                pos += 1;
            }
        }
    }
    segments
}

/// Convert `input` against `dict` by greedy longest match.
/// Characters with no match pass through unchanged.
pub fn convert_with(dict: &Dict, input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for (source, matched) in scan(dict, input) {
        out.push_str(matched.unwrap_or(source));
    }
    out
}

/// One segment of a conversion, for debugging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainStep {
    /// Source text of this segment.
    pub source: Box<str>,
    /// What it became (equals `source` when `matched` is false).
    pub output: Box<str>,
    /// Whether a dictionary entry fired.
    pub matched: bool,
}

/// Like [`convert_with`], but returns the segment-by-segment breakdown
/// so you can see exactly which dictionary entries fired.
pub fn explain_with(dict: &Dict, input: &str) -> Vec<ExplainStep> {
    scan(dict, input)
        .into_iter()
        .map(|(source, matched)| ExplainStep {
            source: source.into(),
            output: matched.unwrap_or(source).into(),
            matched: matched.is_some(),
        })
        .collect()
}

/// Format text as Unicode code points: `"头发"` → `"U+5934 U+53D1"`.
/// Handy when two glyphs look identical but aren't (`不` U+F967 vs `不` U+4E0D).
pub fn code_points(s: &str) -> String {
    s.chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_dict() -> Dict {
        let mut d = Dict::new();
        d.merge_opencc_str("a\tA\nab\tAB\n汉\t漢\n头发\t頭髮\n");
        d
    }

    #[test]
    fn longest_match_wins() {
        assert_eq!(convert_with(&demo_dict(), "ab"), "AB");
        assert_eq!(convert_with(&demo_dict(), "abc"), "ABc");
    }

    #[test]
    fn phrase_beats_chars() {
        assert_eq!(convert_with(&demo_dict(), "头发"), "頭髮");
    }

    #[test]
    fn passthrough_and_empty() {
        assert_eq!(convert_with(&demo_dict(), ""), "");
        assert_eq!(convert_with(&demo_dict(), "xyz123"), "xyz123");
        // Multi-byte non-CJK text must survive untouched.
        assert_eq!(convert_with(&demo_dict(), "héllo→🙂"), "héllo→🙂");
    }

    #[test]
    fn explain_marks_hits_and_passthrough() {
        let steps = explain_with(&demo_dict(), "x头发");
        assert_eq!(steps.len(), 2);
        assert!(!steps[0].matched);
        assert_eq!(steps[0].source.as_ref(), "x");
        assert_eq!(steps[0].output.as_ref(), "x");
        assert!(steps[1].matched);
        assert_eq!(steps[1].source.as_ref(), "头发");
        assert_eq!(steps[1].output.as_ref(), "頭髮");
    }

    #[test]
    fn code_points_formatting() {
        assert_eq!(code_points("头发"), "U+5934 U+53D1");
        assert_eq!(code_points("𣨼"), "U+23A3C"); // non-BMP
        assert_eq!(code_points(""), "");
    }
}
