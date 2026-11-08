//! Cross-profile execution smoke for Nobody's deliberately narrow Rust runtime.
//! This is NOT proof of semantic equivalence for all supported syntax features.
use nobody::profiles::PROFILES;
use nobody::{evaluate_source, translate_source};

#[test]
fn same_basic_program_executes_in_all_40_syntax_profiles() {
    let ai = "let a = 4\nlet b = 5\nprint(a + b)\na = a * 2\nprint(a)";
    let expected = evaluate_source(ai, "ai").expect("AI fixture executes");
    assert_eq!(expected, "9\n8\n");
    assert_eq!(PROFILES.len(), 40, "Update the profile matrix on language expansion");
    for profile in PROFILES {
        let translated = translate_source(ai, "ai", profile.code)
            .unwrap_or_else(|error| panic!("translate {}: {error}", profile.code));
        let output = evaluate_source(&translated, profile.code)
            .unwrap_or_else(|error| panic!("execute {}: {error}", profile.code));
        assert_eq!(output, expected, "execution mismatch for profile {}", profile.code);
    }
}

#[test]
fn conditional_and_bounded_loop_match_across_profiles() {
    let ai = "let n = 0\nwhile n < 3 { n = n + 1 }\nif n == 3 { print(n) }";
    let expected = evaluate_source(ai, "ai").expect("AI loop fixture executes");
    assert_eq!(expected, "3\n");
    for profile in PROFILES {
        let translated = translate_source(ai, "ai", profile.code)
            .unwrap_or_else(|error| panic!("translate {}: {error}", profile.code));
        let output = evaluate_source(&translated, profile.code)
            .unwrap_or_else(|error| panic!("execute {}: {error}", profile.code));
        assert_eq!(output, expected, "control-flow mismatch for {}", profile.code);
    }
}
