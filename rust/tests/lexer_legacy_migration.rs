//! Regression parity for the former Python standalone lexer suite.
//! Keep the Python test file until this replacement is green in CI.
use nobody::{scan, TokenKind};

#[test]
fn legacy_identifier_positions() {
    let tokens = scan("nech cislo = 42\nvypis(cislo)\n").unwrap();
    let names: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Name)
        .map(|t| (t.value.as_str(), t.line, t.column)).collect();
    assert_eq!(names, [("nech", 1, 1), ("cislo", 1, 6), ("vypis", 2, 1), ("cislo", 2, 7)]);
}

#[test]
fn legacy_unicode_string_and_comment() {
    let tokens = scan("vypis(\"Čau IF PRINT\") # Živá poznámka\n").unwrap();
    let kinds: Vec<_> = tokens.iter().filter(|t| matches!(t.kind, TokenKind::String | TokenKind::Comment))
        .map(|t| &t.kind).collect();
    assert_eq!(kinds, [&TokenKind::String, &TokenKind::Comment]);
}

#[test]
fn legacy_multiline_triple_string() {
    let tokens = scan("vypis(\"\"\"if\nprint\n\"\"\")").unwrap();
    assert_eq!(tokens.iter().filter(|t| t.kind == TokenKind::String).count(), 1);
}

#[test]
fn legacy_invalid_names_and_unterminated_strings() {
    for source in ["Vypis(1)", "vypis(1) × 2"] {
        assert!(scan(source).is_err(), "{source}");
    }
    assert_eq!(scan("vypis(\"ahoj)").unwrap_err().code, 2001);
}

#[test]
fn legacy_comments_do_not_emit_identifiers() {
    let tokens = scan("# if IF ČAU\nnech a = 1").unwrap();
    let names: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Name)
        .map(|t| t.value.as_str()).collect();
    assert_eq!(names, ["nech", "a"]);
}
