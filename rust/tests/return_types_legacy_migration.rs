//! Regression coverage migrated from tests/test_021_return_types.py.
//! These tests exercise the production Rust parser and return-type inference,
//! not a Python subprocess or a mocked AST.
use nobody::{check_source_with_return_types, ReturnType};

fn inferred(source: &str) -> ReturnType {
    *check_source_with_return_types(source, "sk")
        .expect("valid Nobody source")
        .get("volba")
        .expect("function is present")
}

#[test]
fn known_scalar_returns_and_local_expressions() {
    assert_eq!(inferred("funkcia volba() { vrat 7 }"), ReturnType::Int);
    assert_eq!(
        inferred("funkcia volba() { vrat \"ahoj\" }"),
        ReturnType::Str
    );
    assert_eq!(
        inferred("funkcia volba() { nech a = 2; vrat a * 3 }"),
        ReturnType::Int
    );
}

#[test]
fn uniform_and_divergent_branches() {
    assert_eq!(
        inferred("funkcia volba(x) { ak x > 0 { vrat 1 } inak { vrat 2 } }"),
        ReturnType::Int
    );
    assert_eq!(
        inferred("funkcia volba(x) { ak x > 0 { vrat 1 } inak { vrat \"nie\" } }"),
        ReturnType::Unknown
    );
    assert_eq!(
        inferred("funkcia volba(x) { ak x > 0 { vrat 1 } }"),
        ReturnType::Unknown
    );
}

#[test]
fn empty_dynamic_and_loop_functions() {
    assert_eq!(inferred("funkcia volba() { }"), ReturnType::None);
    assert_eq!(
        inferred("funkcia volba() { vrat neznama() }"),
        ReturnType::Unknown
    );
    assert_eq!(
        inferred("funkcia volba() { pokial pravda { vrat 4 } }"),
        ReturnType::Unknown
    );
}

#[test]
fn language_profile_is_explicit_and_lowercase() {
    let source = "fn volba() { return 30 }";
    let result = check_source_with_return_types(source, "ai").unwrap();
    assert_eq!(result.get("volba"), Some(&ReturnType::Int));
}

#[test]
fn legacy_typechecker_preserves_known_arity_and_local_type_errors() {
    use nobody::check_source;
    let arity = "funkcia vek() { vrat 30 } nech n = vek(1)";
    assert_eq!(check_source(arity, "sk").unwrap_err().code, 1005);

    let local_mismatch = "funkcia vek() { nech n: cele = \"tridsat\"; vrat n }";
    assert_eq!(check_source(local_mismatch, "sk").unwrap_err().code, 1001);

    let correct = "funkcia vek() { nech n: cele = 30; vrat n }";
    assert!(check_source(correct, "sk").is_ok());
}

#[test]
fn legacy_return_inference_does_not_guess_unknown_call_types() {
    assert_eq!(
        inferred("funkcia volba() { vrat neznama() }"),
        ReturnType::Unknown
    );
}
