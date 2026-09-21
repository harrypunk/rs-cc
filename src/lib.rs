//! rs-cc — a tiny, literal Simplified↔Traditional Chinese converter.
//!
//! Learning project inspired by [OpenCC](https://github.com/ByVoid/OpenCC):
//! greedy longest match over plain-text dictionaries, no segmentation,
//! no semantics. Keys longer than [`Dict::MAX_KEY_LEN`] chars are ignored.
//!
//! ```
//! use rs_cc::{BuiltinConfig, Converter};
//!
//! let c = Converter::new(BuiltinConfig::S2t);
//! assert_eq!(c.convert("头发"), "頭髮");
//! ```

mod convert;
pub mod dict;

use std::io;
use std::path::Path;

pub use convert::convert_with;
pub use dict::{Dict, ParseStats};

// Embedded OpenCC-format dictionaries (Apache-2.0, © OpenCC contributors).
// Loaded in priority order: earlier sources win on duplicate keys.
const CUSTOM: &str = include_str!("../data/custom.txt");
const ST_PHRASES: &str = include_str!("../data/STPhrases.len5.txt");
const ST_CHARS: &str = include_str!("../data/STCharacters.txt");
const TS_PHRASES: &str = include_str!("../data/TSPhrases.len5.txt");
const TS_CHARS: &str = include_str!("../data/TSCharacters.txt");

/// Built-in conversion directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinConfig {
    /// Simplified → Traditional (OpenCC standard, literal subset).
    S2t,
    /// Traditional → Simplified.
    T2s,
}

impl BuiltinConfig {
    /// Dictionary sources in load (= priority) order.
    fn sources(self) -> [&'static str; 3] {
        match self {
            BuiltinConfig::S2t => [CUSTOM, ST_PHRASES, ST_CHARS],
            BuiltinConfig::T2s => [CUSTOM, TS_PHRASES, TS_CHARS],
        }
    }
}

/// A ready-to-use converter holding one merged dictionary.
#[derive(Debug)]
pub struct Converter {
    dict: Dict,
}

impl Converter {
    /// Build from the embedded dictionaries — no file I/O.
    pub fn new(cfg: BuiltinConfig) -> Self {
        let mut dict = Dict::new();
        for src in cfg.sources() {
            dict.merge_opencc_str(src);
        }
        Self { dict }
    }

    /// Like [`Converter::new`], but an extra OpenCC-format dictionary file
    /// is loaded first, so its entries override the built-in ones.
    pub fn with_extra_dict(cfg: BuiltinConfig, path: &Path) -> io::Result<Self> {
        let mut dict = Dict::new();
        dict.merge_opencc_file(path)?;
        for src in cfg.sources() {
            dict.merge_opencc_str(src);
        }
        Ok(Self { dict })
    }

    /// String variant of [`Converter::with_extra_dict`], handy in tests.
    pub fn with_extra_dict_str(cfg: BuiltinConfig, text: &str) -> Self {
        let mut dict = Dict::new();
        dict.merge_opencc_str(text);
        for src in cfg.sources() {
            dict.merge_opencc_str(src);
        }
        Self { dict }
    }

    /// Convert `input` by greedy longest match.
    pub fn convert(&self, input: &str) -> String {
        convert_with(&self.dict, input)
    }

    /// The merged dictionary, exposed for inspection while learning.
    pub fn dict(&self) -> &Dict {
        &self.dict
    }
}
