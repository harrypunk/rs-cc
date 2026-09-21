//! Greedy longest-match conversion ("maximal munch").

use crate::dict::Dict;

/// One segment of a conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainStep {
    /// Source text of this segment.
    pub source: Box<str>,
    /// What it became (equals `source` when `matched` is false).
    pub output: Box<str>,
    /// Whether a dictionary entry fired.
    pub matched: bool,
}

/// Convert `input` against `dict` by greedy longest match.
/// Characters with no match pass through unchanged.
pub fn convert_with(dict: &Dict, input: &str) -> String {
    explain_with(dict, input)
        .into_iter()
        .map(|step| step.output.into_string())
        .collect()
}

/// The segment-by-segment breakdown of a conversion, so you can see
/// exactly which dictionary entries fired.
pub fn explain_with(dict: &Dict, input: &str) -> Vec<ExplainStep> {
    let mut steps = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        let step = take_step(dict, rest);
        rest = &rest[step.source.len()..];
        steps.push(step);
    }
    steps
}

/// Read one segment from the start of `rest` (must be non-empty):
/// the longest dictionary hit, or one passthrough char.
fn take_step(dict: &Dict, rest: &str) -> ExplainStep {
    // Grow the key char by char and remember every hit: the last hit
    // is the longest one, which is what "maximal munch" wants.
    let mut longest_hit = None;
    let mut key = String::new();
    for ch in rest.chars().take(dict.max_key_len()) {
        key.push(ch);
        if let Some(value) = dict.lookup(&key) {
            longest_hit = Some((key.len(), value));
        }
    }
    match longest_hit {
        Some((byte_len, value)) => ExplainStep {
            source: rest[..byte_len].into(),
            output: value.into(),
            matched: true,
        },
        None => {
            let ch = rest.chars().next().expect("take_step needs non-empty input");
            ExplainStep {
                source: ch.to_string().into(),
                output: ch.to_string().into(),
                matched: false,
            }
        }
    }
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
