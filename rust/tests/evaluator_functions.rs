//! Regression tests for lexical function execution and closures in Rust.
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
    let ai =
        "fn first() { let n=0 while n<5 { n=n+1 if n==3 { return n } } return 0 }\nprint(first())";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "3\n");
}

#[test]
fn function_parameters_and_global_reads_do_not_capture_caller_locals() {
    let ai = "let base=4\nfn plus(n) { return n+base }\nif true { let base=100 print(plus(3)) }";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "7\n");
}

#[test]
fn function_calls_mutate_lexical_global_binding() {
    let ai = "let base=5\nfn f() { base=9 return base }\nprint(f())\nprint(base)";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "9\n9\n");
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
fn nested_functions_can_be_called() {
    let ai = "fn f() { fn g() { return 2 } return g() }\nprint(f())";
    assert_eq!(evaluate_source(ai, "ai").unwrap(), "2\n");
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

#[test]
fn escaping_closure_keeps_mutable_capture_and_instances_are_independent() {
    let source = "fn maker(n: int) { fn inc() { n = n + 1 return n } return inc }\nlet a = maker(1)\nlet b = maker(10)\nprint(a(), a(), b(), a())";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "2 3 11 4\n");
    let sk = translate_source(source, "ai", "sk").unwrap();
    assert_eq!(evaluate_source(&sk, "sk").unwrap(), "2 3 11 4\n");
}

#[test]
fn captured_typed_binding_rejects_invalid_assignment() {
    let source =
        "fn maker() { let n: int = 1 fn bad() { n = \"wrong\" } return bad }\nlet f = maker()\nf()";
    assert_eq!(evaluate_source(source, "ai").unwrap_err().code, 4004);
}

#[test]
fn first_class_function_can_be_passed_as_argument() {
    let source =
        "fn twice(n) { return n * 2 }\nfn apply(f, x) { return f(x) }\nprint(apply(twice, 6))";
    assert_eq!(evaluate_source(source, "ai").unwrap(), "12\n");
}
