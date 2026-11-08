//! Cross-profile lexer security regression fixtures.
//! These verify source preservation and diagnostic locations, not full UTS #39.
use nobody::{scan, TokenKind};

#[test]
fn rtl_identifiers_keep_logical_order_and_byte_spans() {
    for name in ["שלום", "مرحبا", "سلام", "اردو"] {
        let source = format!("{name} = 1\n");
        let tokens = scan(&source).expect("native-script identifier");
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].value, name);
        assert_eq!((tokens[0].line, tokens[0].column), (1, 1));
        assert_eq!((tokens[0].start, tokens[0].end), (0, name.len()));
        assert_eq!(tokens[1].value, "=");
        assert_eq!(tokens[1].start, name.len() + 1);
    }
}

#[test]
fn unicode_columns_count_scalars_while_spans_count_utf8_bytes() {
    let source = "שלום x = 1\nαβ y = 2";
    let tokens = scan(source).unwrap();
    let x = tokens.iter().find(|t| t.value == "x").unwrap();
    let y = tokens.iter().find(|t| t.value == "y").unwrap();
    assert_eq!((x.line, x.column, x.start), (1, 6, "שלום ".len()));
    assert_eq!((y.line, y.column, y.start), (2, 4, "שלום x = 1\nαβ ".len()));
}

#[test]
fn nfc_rejects_decomposed_identifiers_but_not_quoted_text() {
    let e = scan("let cafe\u{301} = 1").unwrap_err();
    assert_eq!((e.code, e.line, e.column), (2008, 1, 5));
    let tokens = scan("let café = \"cafe\u{301}\" # cafe\u{301}").unwrap();
    assert!(tokens.iter().any(|t| t.value == "café"));
    assert!(tokens.iter().any(|t| t.value == "\"cafe\u{301}\""));
    assert!(tokens.iter().any(|t| t.value == "# cafe\u{301}"));
}

#[test]
fn mixed_latin_cyrillic_and_greek_report_actual_character_column() {
    for name in ["pаypal", "pαypal"] {
        let e = scan(&format!("let {name} = 2")).unwrap_err();
        assert_eq!((e.code, e.line, e.column), (2007, 1, 6));
    }
    let e = scan("let x = 1\nlet pаypal = 2").unwrap_err();
    assert_eq!((e.code, e.line, e.column), (2007, 2, 6));
}

#[test]
fn mixed_scripts_allowed_in_literals_and_comments() {
    let tokens = scan("print(\"pаypal α\") # pаypal α").unwrap();
    assert_eq!(
        tokens
            .iter()
            .filter(|t| t.kind == TokenKind::String)
            .count(),
        1
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Comment)
            .count(),
        1
    );
}

#[test]
fn japanese_mixed_han_and_hiragana_remains_legal() {
    let tokens = scan("名前かな = 1").unwrap();
    assert_eq!(tokens[0].value, "名前かな");
}

#[test]
fn bidi_controls_rejected_even_inside_quotes_and_comments() {
    for source in [
        "print(\"safe\u005cu{202E}\")",
        "# hidden\u005cu{2066}",
        "let x = 1\u005cu{200F}",
    ] {
        let e = scan(source).unwrap_err();
        assert_eq!(e.code, 2005, "{source:?}");
        assert_eq!(e.line, 1);
        assert!(e.column > 1);
    }
}

#[test]
fn visual_override_reports_second_line_scalar_column() {
    let e = scan("שלום\nאב\u{202E}").unwrap_err();
    assert_eq!((e.code, e.line, e.column), (2005, 2, 3));
}
