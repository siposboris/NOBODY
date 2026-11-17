//! Rust-native regression coverage for the legacy Python bidi safety tests.
//! This verifies source behavior without invoking the Python frontend.
use nobody::profiles::direction;
use nobody::{is_bidi_control, scan, translate_source, TokenKind};

#[test]
fn native_direction_metadata_matches_legacy_profiles() {
    for language in ["ar", "ur", "he", "fa"] {
        assert_eq!(direction(language), Some("rtl"), "{language}");
    }
    for language in ["sk", "en", "ja", "zh"] {
        assert_eq!(direction(language), Some("ltr"), "{language}");
    }
    assert_eq!(direction("unknown-rtl"), None);
}

#[test]
fn native_rtl_translation_preserves_logical_source_order() {
    let source = "nech pocet = 3\nak pocet > 1 {\n vypis(\"Ahoj (3) مرحبا\")\n}\n";
    for language in ["ar", "ur"] {
        let translated = translate_source(source, "sk", language).expect("translate to RTL");
        let restored = translate_source(&translated, language, "sk").expect("restore SK");
        assert_eq!(restored, source, "{language}");
        assert!(translated.contains("\"Ahoj (3) مرحبا\""));
    }
}

#[test]
fn native_arabic_tokens_retain_logical_order() {
    let tokens = scan("إذا صحيح { اطبع(\"مرحبا\") }").expect("Arabic tokens");
    let names: Vec<&str> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Name)
        .map(|t| t.value.as_str())
        .collect();
    assert_eq!(names, ["إذا", "صحيح", "اطبع"]);
}

#[test]
fn native_lexer_rejects_bidi_controls_in_strings_comments_and_code() {
    for prefix in ["vypis(\"", "# ", "nech cislo = "] {
        for control in ['\u{202e}', '\u{2066}', '\u{200f}', '\u{061c}'] {
            assert!(is_bidi_control(control));
            let source = format!("{prefix}{control}abc\")");
            let err = scan(&source).expect_err("bidi control must be rejected");
            assert_eq!(err.code, 2005, "prefix={prefix:?}, control={control:?}");
            assert!(err.detail.contains("U+"));
        }
    }
}

#[test]
fn native_lexer_accepts_arabic_diacritics() {
    scan("إذا صحيح { اطبع(\"السَّلَامُ\") }").expect("Arabic marks are not bidi controls");
}
