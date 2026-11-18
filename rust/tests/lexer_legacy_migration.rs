//! Legacy Python lexer regressions ported to the Rust scanner.
//! Keep Python tests until remaining consumers and CI are migrated.
use nobody::{scan, TokenKind};

#[test]
fn legacy_identifier_positions() {
    let tokens = scan("nech cislo = 42\nvypis(cislo)\n").unwrap();
    let names: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Name)
        .map(|t| (t.value.as_str(), t.line, t.column)).collect();
    assert_eq!(names, vec![
        ("nech", 1, 1), ("cislo", 1, 6),
        ("vypis", 2, 1), ("cislo", 2, 7),
    ]);
}

#[test]
fn legacy_unicode_string_and_comment() {
    let tokens = scan("vypis(\"Čau IF PRINT\") # Živá poznámka\n").unwrap();
    let kinds: Vec<_> = tokens.iter().filter(|t|
        matches!(t.kind, TokenKind::String | TokenKind::Comment))
        .map(|t| t.kind).collect();
    assert_eq!(kinds, vec![TokenKind::String, TokenKind::Comment]);
}

#[test]
fn legacy_multiline_triple_quoted_string() {
    let tokens = scan("vypis(\"\"\"if\nprint\n\"\"\")").unwrap();
    assert_eq!(tokens.iter().filter(|t| t.kind == TokenKind::String).count(), 1);
}

#[test]
fn legacy_invalid_identifier_and_symbol() {
    assert_eq!(scan("Vypis(1)").unwrap_err().code, 2002);
    assert_eq!(scan("vypis(1) × 2").unwrap_err().code, 2004);
}

#[test]
fn legacy_unterminated_string() {
    assert_eq!(scan("vypis(\"ahoj)").unwrap_err().code, 2001);
}

#[test]
fn legacy_comment_words_not_identifiers() {
    let tokens = scan("# if IF ČAU\nnech a = 1").unwrap();
    let names: Vec<_> = tokens.iter().filter(|t| t.kind == TokenKind::Name)
        .map(|t| t.value.as_str()).collect();
    assert_eq!(names, vec!["nech", "a"]);
}
