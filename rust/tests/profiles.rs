use nobody::profiles::{semantic, word, CANONICAL, PROFILES};
use nobody::{migrate_legacy_ai, parse, parse_validated};

#[test]
fn every_language_has_complete_semantic_table() {
    assert_eq!(PROFILES.len(), 40);
    for p in PROFILES {
        for term in CANONICAL {
            let native = word(p.code, term).expect("every keyword is defined");
            assert_eq!(
                semantic(p.code, native),
                Some(term),
                "profile={} word={native}",
                p.code
            );
        }
    }
}
#[test]
fn every_profile_parses_native_declaration_and_print() {
    for p in PROFILES {
        let let_word = word(p.code, "let").unwrap();
        let integer = word(p.code, "int").unwrap();
        let print = word(p.code, "print").unwrap();
        let source = format!("{let_word} x: {integer} = 3\n{print}(x)");
        parse_validated(&source, p.code)
            .unwrap_or_else(|error| panic!("failed native profile {}: {}", p.code, error));
    }
}
#[test]
fn existing_native_control_flow_is_parsed() {
    for code in ["ar", "he", "ru", "ja", "ko", "zh", "ne", "ta", "th"] {
        let conditional = word(code, "if").unwrap();
        let truth = word(code, "true").unwrap();
        let printer = word(code, "print").unwrap();
        let source = format!("{conditional} {truth} {{ {printer}(1) }}");
        parse(&source, code)
            .unwrap_or_else(|error| panic!("failed condition profile {}: {}", code, error));
    }
}
#[test]
fn ai_profile_is_familiar_and_does_not_require_prefixes() {
    let s = "fn twice(x) { return x * 2 }\nlet n: int = 4\nif n > 2 { print(twice(n)) }";
    let tree = parse_validated(s, "ai").unwrap();
    assert_eq!(tree.statements.len(), 3);
}
#[test]
fn ai_legacy_can_be_migrated_without_touching_string_literals() {
    let old = "ai_let x: ai_int = 2 # ai_if\nai_if ai_true { ai_print(\"ai_print\", x) }";
    let translated = migrate_legacy_ai(old).unwrap();
    assert_eq!(
        translated,
        "let x: int = 2 # ai_if\nif true { print(\"ai_print\", x) }"
    );
    parse_validated(&translated, "ai").unwrap();
}
