use rs_cc::{BuiltinConfig, Converter};

fn s2t() -> Converter {
    Converter::new(BuiltinConfig::S2t)
}

fn t2s() -> Converter {
    Converter::new(BuiltinConfig::T2s)
}

#[test]
fn single_characters() {
    assert_eq!(s2t().convert("汉"), "漢");
    assert_eq!(t2s().convert("髮"), "发");
}

#[test]
fn phrase_beats_character_fallback() {
    // 头发 must become 頭髮 (hair), not 頭發 (develop): the phrase
    // dictionary decides what character fallback alone gets wrong.
    assert_eq!(s2t().convert("头发"), "頭髮");
}

#[test]
fn char_level_fallback() {
    // 鼠标 is not a phrase entry; 标 converts per character.
    assert_eq!(s2t().convert("鼠标"), "鼠標");
}

#[test]
fn sentence_roundtrip() {
    let simp = "我用鼠标写了一段关于头发的文字";
    let trad = "我用鼠標寫了一段關於頭髮的文字";
    assert_eq!(s2t().convert(simp), trad);
    assert_eq!(t2s().convert(trad), simp);
}

#[test]
fn passthrough() {
    assert_eq!(s2t().convert(""), "");
    assert_eq!(s2t().convert("abc123 🙂"), "abc123 🙂");
}

#[test]
fn keys_longer_than_five_chars_are_ignored() {
    let c = Converter::with_extra_dict_str(BuiltinConfig::S2t, "abcde\tX\nabcdef\tY\n");
    assert_eq!(c.convert("abcde"), "X");
    // 6-char key skipped at load; the 5-char prefix still matches.
    assert_eq!(c.convert("abcdef"), "Xf");
}

#[test]
fn custom_dict_overrides_builtin() {
    let c = Converter::with_extra_dict_str(BuiltinConfig::S2t, "头发\t自定義\n");
    assert_eq!(c.convert("头发"), "自定義");
}

#[test]
fn unicode_escapes_in_custom_dict() {
    // \u{6C49}\u{5B57} = 汉字, \u{6F22}\u{5B57} = 漢字 — no IME needed.
    let c = Converter::with_extra_dict_str(
        BuiltinConfig::S2t,
        "\\u{6C49}\\u{5B57}\t\\u{6F22}\\u{5B57}\n",
    );
    assert_eq!(c.convert("汉字"), "漢字");
}

#[test]
fn explain_reports_matches() {
    let steps = s2t().explain("a头发");
    assert_eq!(steps.len(), 2);
    assert!(!steps[0].matched); // passthrough
    assert!(steps[1].matched); // phrase hit
    assert_eq!(rs_cc::code_points(&steps[1].output), "U+982D U+9AEE");
}
