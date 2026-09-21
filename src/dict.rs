//! Dictionary loading: OpenCC text format (`key<TAB>value1 value2 ...`).
//!
//! First candidate value wins, and the first occurrence of a key wins —
//! so load order is priority order.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// What happened while merging one dictionary source.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    /// Entries newly inserted.
    pub loaded: usize,
    /// Entries skipped because the key is longer than [`Dict::MAX_KEY_LEN`] chars.
    pub skipped_too_long: usize,
    /// Entries skipped because the key was already present.
    pub skipped_duplicate: usize,
}

/// Flat key → value mapping built from OpenCC text dictionaries.
#[derive(Debug, Default)]
pub struct Dict {
    map: HashMap<Box<str>, Box<str>>,
    max_key_len: usize,
}

impl Dict {
    /// Keys longer than this many chars are skipped at load time.
    pub const MAX_KEY_LEN: usize = 5;

    pub fn new() -> Self {
        Self::default()
    }

    /// Number of loaded entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Longest loaded key, in chars. Matching never probes beyond this.
    pub fn max_key_len(&self) -> usize {
        self.max_key_len
    }

    /// Exact lookup of a single key.
    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|v| &**v)
    }

    /// Merge OpenCC-format text: lines of `key<TAB>value1 value2 ...`,
    /// with `#` comments and blank lines ignored.
    pub fn merge_opencc_str(&mut self, text: &str) -> ParseStats {
        let mut stats = ParseStats::default();
        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, values)) = line.split_once('\t') else {
                continue;
            };
            let Some(value) = values.split_whitespace().next() else {
                continue;
            };
            let key_len = key.chars().count();
            if key_len > Self::MAX_KEY_LEN {
                stats.skipped_too_long += 1;
                continue;
            }
            if self.map.contains_key(key) {
                stats.skipped_duplicate += 1;
                continue;
            }
            self.max_key_len = self.max_key_len.max(key_len);
            self.map.insert(key.into(), value.into());
            stats.loaded += 1;
        }
        stats
    }

    /// Merge an OpenCC-format dictionary file. See [`Dict::merge_opencc_str`].
    pub fn merge_opencc_file(&mut self, path: &Path) -> io::Result<ParseStats> {
        let text = fs::read_to_string(path)?;
        Ok(self.merge_opencc_str(&text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_opencc_format() {
        let mut d = Dict::new();
        let stats = d.merge_opencc_str("# comment\n\n汉\t漢\n头发\t頭髮 頭发\n");
        assert_eq!(stats.loaded, 2);
        assert_eq!(d.lookup("汉"), Some("漢"));
        // First candidate value wins.
        assert_eq!(d.lookup("头发"), Some("頭髮"));
        assert_eq!(d.max_key_len(), 2);
    }

    #[test]
    fn first_occurrence_wins() {
        let mut d = Dict::new();
        d.merge_opencc_str("干\t乾\n");
        let stats = d.merge_opencc_str("干\t幹\n");
        assert_eq!(stats.skipped_duplicate, 1);
        assert_eq!(d.lookup("干"), Some("乾"));
    }

    #[test]
    fn overlong_keys_are_skipped() {
        let mut d = Dict::new();
        let stats = d.merge_opencc_str("abcde\tX\nabcdef\tY\n"); // 5 and 6 chars
        assert_eq!(stats.loaded, 1);
        assert_eq!(stats.skipped_too_long, 1);
        assert_eq!(d.lookup("abcde"), Some("X"));
        assert_eq!(d.lookup("abcdef"), None);
        assert_eq!(d.max_key_len(), 5);
    }
}
