//! Focused tests for the initial Rust named-function execution subset.
//! Functions are currently non-recursive and nested definitions are unsupported.
use nobody::{evaluate_source, translate_source};

#[test]
fn ai_and_slovak_named_functions_match() {
    let ai = "fn add(a,b) { return a+b }\nprint(add(2,3))";
    let sk = translate_source(ai, "ai", "sk").expect("translate function");
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "5\n");
    assert_eq!(evaluate_source(&sk, "sk").unwrap(), "5\n");
}

#[test]
fn returns_from_nested_control_flow() {
    let ai = "fn first() { let n=0 while n<5 { n=n+1 if n==3 { return n } } return 0 }\nprint(first())";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "3\n");
}

#[test]
fn function_parameters_and_global_reads_do_not_capture_caller_locals() {
    let ai = "let base=4\nfn plus(n) { return n+base }\nif true { let base=100 print(plus(3)) }";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "7\n");
}

#[test]
fn function_calls_do_not_mutate_global_snapshot() {
    let ai = "let base=5\nfn f() { base=9 return base }\nprint(f())\nprint(base)";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "9\n5\n");
}

#[test]
fn no_implicit_caller_scope_access() {
    let ai = "fn f() { return hidden }\nif true { let hidden=5 print(f()) }";
    assert_eq!(evaluate_source(ai, "ai").unwrap_err().code, 4001);
}

#[test]
fn wrong_arity_has_numbered_diagnostic() {
    let ai = "fn add(a,b) { return a+b }\nprint(add(2))";
    assert_eq!(evaluate_source(ai, "ai").unwrap_err().code, 4007);
}

#[test]
fn recursion_and_mutual_recursion_are_bounded() {
    let direct = "fn f() { return f() }\nprint(f())";
    let indirect = "fn a() { return b() }\nfn b() { return a() }\nprint(a())";
    assert_eq!(evaluate_source(direct, "ai").unwrap_err().code, 4005);
    assert_eq!(evaluate_source(indirect, "ai").unwrap_err().code, 4005);
}

#[test]
fn nested_functions_are_explicitly_unsupported() {
    let ai = "fn f() { fn g() { return 2 } return g() }\nprint(f())";
    assert_eq!(evaluate_source(ai, "ai").unwrap_err().code, 4004);
}

#[test]
fn functions_without_return_produce_null() {
    let ai = "fn f(x) { print(x) }\nprint(f(3))";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "3\nnull\n");
}

#[test]
fn calling_variable_value_is_rejected() {
    let ai = "fn f() { return 1 }\nif true { let f=5 print(f()) }";
    assert_eq!(evaluate_source(ai, "ai").unwrap_err().code, 4004);
}
