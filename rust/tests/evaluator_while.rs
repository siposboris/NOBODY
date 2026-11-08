//! Explicit while/control-flow evaluator regression coverage.
use nobody::evaluate_source;

#[test]
fn ai_and_slovak_while_programs_agree() {
    let ai = "let i = 0\nwhile i < 3 { print(i)\ni = i + 1 }";
    let sk = "nech i = 0\npokial i < 3 { vypis(i)\ni = i + 1 }";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "0\n1\n2\n");
    assert_eq!(evaluate_source(sk, "sk").unwrap(), "0\n1\n2\n");
}

#[test]
fn break_and_continue_propagate_out_of_nested_if() {
    let source = "let i = 0\nwhile i < 7 { i = i + 1\nif i == 2 { continue }\nif i == 5 { break }\nprint(i) }\nprint(i)";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "1\n3\n4\n5\n");
}

#[test]
fn inner_loop_break_does_not_break_outer_loop() {
    let source =
        "let outer = 0\nwhile outer < 2 { outer = outer + 1\nwhile true { print(outer)\nbreak } }";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "1\n2\n");
}

#[test]
fn each_iteration_gets_fresh_lexical_scope() {
    let source = "let i = 0\nwhile i < 3 { let local = i\ni = i + 1\nprint(local) }";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "0\n1\n2\n");
    assert_eq!(
        evaluate_source("while false { let local = 1 }\nprint(local)", "ai")
            .unwrap_err()
            .code,
        4001
    );
}

#[test]
fn dead_loop_does_not_evaluate_body() {
    assert_eq!(
        evaluate_source("while false { print(missing) }\nprint(7)", "ai").unwrap(),
        "7\n"
    );
}

#[test]
fn loops_need_boolean_conditions() {
    assert_eq!(
        evaluate_source("while 1 { break }", "ai").unwrap_err().code,
        4004
    );
}

#[test]
fn forever_loop_ends_in_budget_error() {
    assert_eq!(
        evaluate_source("while true { }", "ai").unwrap_err().code,
        4005
    );
}

#[test]
fn loop_control_outside_loop_still_invalid_at_parse_time() {
    assert_eq!(evaluate_source("break", "ai").unwrap_err().code, 3009);
    assert_eq!(evaluate_source("continue", "ai").unwrap_err().code, 3009);
}

#[test]
fn range_for_loops_are_supported_but_other_iterables_are_deferred() {
    assert_eq!(
        evaluate_source("for x in range(2) { print(x) }", "ai").unwrap(),
        "0\n1\n"
    );
    assert_eq!(
        evaluate_source("for x in unknown { print(x) }", "ai")
            .unwrap_err()
            .code,
        4004
    );
}
