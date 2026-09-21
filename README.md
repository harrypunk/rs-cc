# rs-cc

A tiny, literal Simplified↔Traditional Chinese converter in Rust —
a learning project inspired by [OpenCC](https://github.com/ByVoid/OpenCC),
not a replacement for it.

Zero dependencies, pure library crate, one algorithm: **greedy longest
match** ("maximal munch") over plain-text dictionaries.

```rust
use rs_cc::{BuiltinConfig, Converter};

let c = Converter::new(BuiltinConfig::S2t);
assert_eq!(c.convert("我用鼠标写了一段关于头发的文字"), "我用鼠標寫了一段關於頭髮的文字");
```

## What it does / doesn't do vs OpenCC

| | rs-cc | OpenCC |
|---|---|---|
| Character conversion (s2t, t2s) | yes | yes |
| Phrase dictionaries | yes, keys ≤ 5 chars | yes, any length |
| Matching | greedy longest match | mmseg segmentation + match policies |
| One-to-many ambiguities | first candidate wins | first candidate wins (standard configs) |
| Regional vocabularies (TW/HK/JP) | no | yes |
| Dictionary format | OpenCC text format (`key⇥value`) | same text format, compiled to `.ocd2` |
| Dependencies | none | marisa-trie, rapidjson, … |

## How it works

- `src/dict.rs` — parses OpenCC text dictionaries (`key<TAB>value1 value2 …`)
  into a `HashMap`. First candidate wins; first occurrence of a key wins,
  so load order is priority order. Keys longer than
  `Dict::MAX_KEY_LEN` (5 chars) are skipped.
- `src/convert.rs` — greedy longest match: at each position grow the key
  char by char (≤ `max_key_len`) and keep the longest hit; no hit means
  the char passes through unchanged. UTF-8 safe (works on char
  boundaries), O(n · MAX_KEY_LEN). `convert` and `explain` share this
  one code path, so they can never disagree.
- `src/lib.rs` — `Converter::new(BuiltinConfig::S2t | T2s)` merges
  embedded dictionaries via `include_str!`, in priority order:
  `custom.txt` → phrases → characters.

## Dictionaries

`data/` contains OpenCC's own dictionaries (Apache-2.0, © OpenCC
contributors), copied from the OpenCC repo: `STCharacters.txt`,
`TSCharacters.txt` verbatim; `STPhrases.len5.txt` / `TSPhrases.len5.txt`
filtered to keys ≤ 5 chars (keeps ~97.6% of phrase entries).

To customize:

- Edit `data/custom.txt` (embedded, highest priority) and rebuild, or
- Load any OpenCC-format file at runtime:
  `Converter::with_extra_dict(BuiltinConfig::S2t, path)`.

Both literal characters and `\u{...}` escapes are accepted in any
dictionary file, so custom entries can be written without an IME or
CJK font: `\u{6C49}\u{5B57}⇥\u{6F22}\u{5B57}` is `汉字⇥漢字`.

## Debugging conversions

`Converter::explain` shows which entries fired, segment by segment;
`code_points` renders any text as Unicode values — handy when two
glyphs look identical but aren't (e.g. `不` U+F967 vs `不` U+4E0D):

```rust
use rs_cc::{BuiltinConfig, Converter, code_points};

let c = Converter::new(BuiltinConfig::S2t);
for step in c.explain("a头发") {
    println!("{} ({}) -> {} ({}) matched={}",
        step.source, code_points(&step.source),
        step.output, code_points(&step.output), step.matched);
}
// a (U+0061) -> a (U+0061) matched=false
// 头发 (U+5934 U+53D1) -> 頭髮 (U+982D U+9AEE) matched=true
```

## Accuracy vs OpenCC

Against OpenCC's golden test cases (`test/testcases/testcases.json`):
s2t matches 106/108, t2s matches 52/56. Divergences are exactly the
cases this project deliberately skips: segmentation-sensitive phrases
(e.g. OpenCC keeps `札记` unchanged), variant choices (`鍾` vs `锺`),
and keys longer than 5 chars.

## Deliberate limits (first edition)

- No CLI, no config files, no binary dictionary format.
- No semantic disambiguation beyond phrase longest-match.
- Entries with keys > 5 chars are silently dropped (counted in
  `ParseStats::skipped_too_long`).

Possible next steps: configurable `MAX_KEY_LEN`, a trie instead of
`HashMap`, a criterion benchmark, Taiwan/HK variant dictionaries.

## License

Code: MIT OR Apache-2.0. Dictionaries under `data/`: Apache-2.0,
© OpenCC contributors (see file headers).
