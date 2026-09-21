//! Greedy longest-match conversion ("maximal munch").

use crate::dict::Dict;

/// Convert `input` against `dict`, scanning left to right and always
/// taking the longest key that matches at the current position.
/// Characters with no match pass through unchanged.
pub fn convert_with(dict: &Dict, input: &str) -> String {
    // Byte offset of every char boundary, plus end of string — lets us
    // slice by char count without ever splitting a UTF-8 codepoint.
    let bounds: Vec<usize> = input
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(input.len()))
        .collect();

    let mut out = String::with_capacity(input.len());
    let mut pos = 0;
    while pos + 1 < bounds.len() {
        let remaining = bounds.len() - 1 - pos; // chars left
        let max = dict.max_key_len().min(remaining);
        let mut matched_len = None;
        for len in (1..=max).rev() {
            let key = &input[bounds[pos]..bounds[pos + len]];
            if let Some(value) = dict.lookup(key) {
                out.push_str(value);
                matched_len = Some(len);
                break;
            }
        }
        match matched_len {
            Some(len) => pos += len,
            None => {
                out.push_str(&input[bounds[pos]..bounds[pos + 1]]);
                pos += 1;
            }
        }
    }
    out
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
}
