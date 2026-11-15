//! Rust-native regression coverage for the legacy Python bidi safety contract.
//! This does not invoke Python or change source character order.
use nobody::profiles::{direction, PROFILES};
use nobody::{is_bidi_control, scan};

#[test]
fn all_profiles_have_direction_and_unknown_profiles_fail() {
    for profile in PROFILES {
        let expected = if matches!(profile.code, "ar" | "ur" | "he" | "fa") {
            "rtl"
        } else {
            "ltr"
        };
        assert_eq!(direction(profile.code), Some(expected), "{}", profile.code);
    }
    assert_eq!(direction("unknown-rtl"), None);
}

#[test]
fn every_forbidden_bidi_control_is_rejected_in_code_comments_and_strings() {
    let controls = [
        0x061c, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066, 0x2067, 0x2068,
        0x2069,
    ];
    for scalar in controls {
        let ch = char::from_u32(scalar).unwrap();
        assert!(is_bidi_control(ch));
        for (prefix, suffix) in [
            ("nech cislo = ", "1"),
            ("# comment ", ""),
            ("vypis(\"", "\")"),
        ] {
            let source = format!("{prefix}{ch}{suffix}");
            let error = scan(&source).expect_err("bidi control must fail even inside strings");
            assert_eq!(error.code, 2005, "U+{scalar:04X}: {source:?}");
            assert_eq!(error.line, 1);
            assert_eq!(error.column, prefix.chars().count() + 1);
        }
    }
}

#[test]
fn native_rtl_letters_and_diacritics_keep_logical_token_order() {
    let source = "إذا صحيح { اطبع(\"السَّلَامُ\") }";
    let names: Vec<_> = scan(source)
        .unwrap()
        .into_iter()
        .filter(|token| token.kind == nobody::TokenKind::Name)
        .map(|token| token.value)
        .collect();
    assert_eq!(names, ["إذا", "صحيح", "اطبع"]);
    assert!(scan("vypis(\"مرحبا\")").is_ok());
}

#[test]
fn rtl_translation_roundtrip_preserves_logical_source_and_string_bytes() {
    // Port of tests/test_bidi.py::test_no_reordering_round_trip.
    let source = "nech pocet = 3\nak pocet > 1 {\n vypis(\"Ahoj (3) مرحبا\")\n}\n";
    for language in ["ar", "ur"] {
        let translated =
            nobody::translate_source(source, "sk", language).expect("translation into RTL syntax");
        assert!(translated.contains("\"Ahoj (3) مرحبا\""));
        assert_eq!(
            nobody::translate_source(&translated, language, "sk")
                .expect("translation back into Slovak"),
            source,
            "RTL roundtrip changed source for {language}"
        );
        nobody::parse(&translated, language).expect("RTL source must parse");
    }
}
