//! Conditional execution, lazy booleans and lexical scope regression tests.
use nobody::evaluate_source;

#[test]
fn ai_and_slovak_if_else_match() {
    let ai = "let score = 4\nif score > 3 { print(\"pass\") } else { print(\"fail\") }";
    let sk = "nech score = 4\nak score > 3 { vypis(\"pass\") } inak { vypis(\"fail\") }";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "pass\n");
    assert_eq!(evaluate_source(sk, "sk").unwrap(), "pass\n");
}

#[test]
fn else_branch_and_else_if_chains() {
    assert_eq!(
        evaluate_source(
            "if false { print(1) } else if false { print(2) } else { print(3) }",
            "ai"
        )
        .unwrap(),
        "3\n"
    );
    assert_eq!(
        evaluate_source("if false { print(1) } else { print(2) }", "ai").unwrap(),
        "2\n"
    );
}

#[test]
fn comparisons_and_boolean_logic() {
    assert_eq!(
        evaluate_source("print(2 < 3, 2 <= 2, 4 > 3, 4 >= 4, 1 == 1, 1 != 2)", "ai").unwrap(),
        "true true true true true true\n"
    );
    assert_eq!(
        evaluate_source("if true and not false { print(\"yes\") }", "ai").unwrap(),
        "yes\n"
    );
}

#[test]
fn logical_short_circuit_skips_unsafe_side() {
    assert_eq!(
        evaluate_source("print(false and missing, true or missing)", "ai").unwrap(),
        "false true\n"
    );
}

#[test]
fn branch_bindings_shadow_without_leaking_and_outer_assignments_persist() {
    let source = "let value = 1\nif true { let value = 7\nprint(value) }\nprint(value)\nif true { value = 9 }\nprint(value)";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "7\n1\n9\n");
    assert_eq!(
        evaluate_source("if true { let local = 1 }\nprint(local)", "ai")
            .unwrap_err()
            .code,
        4001
    );
}

#[test]
fn dead_branches_are_not_evaluated() {
    assert_eq!(
        evaluate_source("if false { print(missing) } else { print(42) }", "ai").unwrap(),
        "42\n"
    );
}

#[test]
fn non_boolean_condition_is_explicitly_unsupported() {
    assert_eq!(
        evaluate_source("if 1 { print(1) }", "ai").unwrap_err().code,
        4004
    );
}

#[test]
fn while_and_basic_named_functions_are_executable() {
    assert_eq!(
        evaluate_source("while false { print(1) }", "ai").unwrap(),
        ""
    );
    assert_eq!(
        evaluate_source("fn f() { return 1 }\nprint(f())", "ai").unwrap(),
        "1\n"
    );
}
