//! Regression tests for N1007: unknown explicit primitive type annotations.
use nobody::check_source;

#[test]
fn unknown_ai_type_annotation_reports_n1007() {
    let error = check_source("let age: intr = 3", "ai").unwrap_err();
    assert_eq!(error.code, 1007);
    assert!(error.detail.contains("age"));
    assert!(error.detail.contains("intr"));
    assert!(error.repair.contains("int"));
}

#[test]
fn unknown_slovak_type_annotation_reports_n1007() {
    let error = check_source("nech vek: neexistujuci = 3", "sk").unwrap_err();
    assert_eq!(error.code, 1007);
    assert!(error.detail.contains("neexistujuci"));
}

#[test]
fn unknown_annotations_fail_even_if_initializer_is_dynamic() {
    let error = check_source("let item: intr = mystery(1)", "ai").unwrap_err();
    assert_eq!(error.code, 1007);
}

#[test]
fn all_supported_primitive_annotations_still_work() {
    check_source(
        "let a: int = 1\nlet b: float = 1\nlet c: str = \"hello\"\nlet d: bool = true",
        "ai",
    )
    .unwrap();
    check_source("nech vek: cele = 3\nnech pomer: desatinne = 2", "sk").unwrap();
}

#[test]
fn an_unannotated_unknown_expression_remains_deferred() {
    check_source("let item = mystery(1)", "ai").unwrap();
}

#[test]
fn nested_unknown_type_annotation_is_rejected() {
    let error = check_source("fn f() { let item: intr = 4 }", "ai").unwrap_err();
    assert_eq!(error.code, 1007);
}
