//! Bounded range-based for-loop evaluator regression tests.
use nobody::evaluate_source;

#[test]
fn ai_and_slovak_for_range_match() {
    let ai = "for i in range(4) { print(i) }";
    let sk = "pre i v rozsah(4) { vypis(i) }";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "0\n1\n2\n3\n");
    assert_eq!(evaluate_source(sk, "sk").unwrap(), "0\n1\n2\n3\n");
}

#[test]
fn range_start_stop_and_step() {
    assert_eq!(
        evaluate_source("for i in range(2, 8, 2) { print(i) }", "ai").unwrap(),
        "2\n4\n6\n"
    );
    assert_eq!(
        evaluate_source("for i in range(5, -1, -2) { print(i) }", "ai").unwrap(),
        "5\n3\n1\n"
    );
    assert_eq!(
        evaluate_source("for i in range(3, 0) { print(i) }", "ai").unwrap(),
        ""
    );
}

#[test]
fn loop_variable_and_local_bindings_do_not_escape() {
    assert_eq!(
        evaluate_source("let i = 99\nfor i in range(2) { print(i) }\nprint(i)", "ai").unwrap(),
        "0\n1\n99\n"
    );
    assert_eq!(
        evaluate_source("for i in range(1) { let local = i }\nprint(local)", "ai")
            .unwrap_err()
            .code,
        4001
    );
    assert_eq!(
        evaluate_source("for i in range(1) { print(i) }\nprint(i)", "ai")
            .unwrap_err()
            .code,
        4001
    );
}

#[test]
fn break_continue_and_nested_while() {
    let source = "for i in range(6) { if i == 2 { continue }\nif i == 4 { break }\nprint(i) }";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "0\n1\n3\n");
    assert_eq!(
        evaluate_source("for i in range(2) { while true { print(i)\nbreak } }", "ai").unwrap(),
        "0\n1\n"
    );
}

#[test]
fn iterable_parameters_evaluate_once() {
    assert_eq!(
        evaluate_source(
            "let limit = 3\nfor i in range(limit) { limit = limit + 1\nprint(i) }\nprint(limit)",
            "ai"
        )
        .unwrap(),
        "0\n1\n2\n6\n"
    );
}

#[test]
fn bad_range_parameters_are_explicit() {
    for source in [
        "for i in range() { print(i) }",
        "for i in range(1, 2, 3, 4) { print(i) }",
        "for i in range(\"2\") { print(i) }",
        "for i in 3 { print(i) }",
    ] {
        assert_eq!(
            evaluate_source(source, "ai").unwrap_err().code,
            4004,
            "{source}"
        );
    }
    assert_eq!(
        evaluate_source("for i in range(0, 4, 0) { print(i) }", "ai")
            .unwrap_err()
            .code,
        4006
    );
}

#[test]
fn huge_range_is_stopped_by_step_budget() {
    assert_eq!(
        evaluate_source("for i in range(1000000000) { }", "ai")
            .unwrap_err()
            .code,
        4005
    );
}

#[test]
fn empty_ranges_do_not_evaluate_body() {
    assert_eq!(
        evaluate_source("for i in range(0) { print(missing) }\nprint(7)", "ai").unwrap(),
        "7\n"
    );
}

#[test]
fn boundary_values_do_not_overflow_iteration() {
    assert_eq!(
        evaluate_source(
            "for i in range(170141183460469231731687303715884105726, 170141183460469231731687303715884105727) { print(i) }",
            "ai"
        )
        .unwrap(),
        "170141183460469231731687303715884105726\n"
    );
}
