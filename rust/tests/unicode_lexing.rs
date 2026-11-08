//! Unicode/RTL lexer acceptance fixtures: logical order, UTF-8 byte spans,
//! scalar-value columns and explicit rejection of invisible bidi controls.
//! This does not claim Unicode normalization or confusable detection.
use nobody::{parse_validated, scan, translate_source, TokenKind};
use nobody::profiles::word;

#[test]
fn rtl_profiles_keep_logical_token_order_and_original_spelling() {
    for code in ["ar", "he", "fa", "ur"] {
        let declaration = word(code, "let").unwrap();
        let printer = word(code, "print").unwrap();
        let source = format!("{declaration} قيمة = 3\n{printer}(قيمة)");
        let tokens = scan(&source).unwrap();
        assert_eq!(tokens[0].value, declaration, "{code}");
        assert_eq!(tokens[1].value, "قيمة", "{code}");
        assert_eq!(tokens[1].kind, TokenKind::Name, "{code}");
        assert_eq!(tokens[1].start, declaration.len() + 1, "{code}");
        assert_eq!(&source[tokens[1].start..tokens[1].end], "قيمة", "{code}");
        assert_eq!(tokens[1].column, declaration.chars().count() + 2, "{code}");
        assert_eq!(
            tokens.iter().find(|t| t.line == 2).unwrap().value,
            printer,
            "{code}"
        );
        parse_validated(&source, code).unwrap();
    }
}

#[test]
fn combining_marks_after_letter_are_part_of_one_identifier() {
    let source = "let cafe\u{301} = 1";
    let tokens = scan(source).unwrap();
    let name = &tokens[1];
    assert_eq!(name.value, "cafe\u{301}");
    assert_eq!(name.column, 5);
    assert_eq!(&source[name.start..name.end], name.value);
    assert_eq!(name.end - name.start, name.value.len());
    parse_validated(source, "ai").unwrap();
}

#[test]
fn orphan_combining_mark_is_rejected_with_precise_location() {
    let diagnostic = scan("let \u{301}x = 1").unwrap_err();
    assert_eq!(diagnostic.code, 2003);
    assert_eq!((diagnostic.line, diagnostic.column), (1, 5));
}

#[test]
fn bidi_override_is_rejected_even_in_comments_and_strings() {
    for source in ["let x = 1 # \u{202e} hidden", "print(\"\u{2066}\")"] {
        let diagnostic = scan(source).unwrap_err();
        assert_eq!(diagnostic.code, 2005);
        assert!(diagnostic.detail.contains("U+"), "{diagnostic}");
    }
}

#[test]
fn rtl_translation_preserves_string_and_comment_bytes() {
    let source = "let x = \"עברית العربية\" # שלום مرحبا\nprint(x)";
    let ar = translate_source(source, "ai", "ar").unwrap();
    assert!(ar.contains("\"עברית العربية\" # שלום مرحبا"));
    let roundtrip = translate_source(&ar, "ar", "ai").unwrap();
    assert_eq!(roundtrip, source);
}

#[test]
fn unicode_token_spans_are_valid_utf8_slices() {
    let source = "let café = \"שלום\"\nprint(café)";
    let tokens = scan(source).unwrap();
    for t in tokens {
        assert!(source.is_char_boundary(t.start));
        assert!(source.is_char_boundary(t.end));
        assert_eq!(&source[t.start..t.end], t.value);
    }
}
