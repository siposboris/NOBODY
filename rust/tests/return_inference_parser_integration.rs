//! End-to-end Rust parser -> return inference regressions, no Python interpreter.
use nobody::{infer_function_return, parse, ReturnType, Statement};

fn inferred(source: &str, profile: &str) -> ReturnType {
    let program = parse(source, profile).expect("source must parse in its own profile");
    let function = program.statements.first().expect("function statement");
    assert!(matches!(function, Statement::Function { .. }));
    infer_function_return(function)
}

#[test]
fn inference_uses_real_ai_parser() {
    assert_eq!(
        inferred("fn result() { let x = 2 x = x + 3 return x }", "ai"),
        ReturnType::Int
    );
    assert_eq!(
        inferred("fn result() { if true { return 1 } else { return 2 } }", "ai"),
        ReturnType::Int
    );
}

#[test]
fn inference_preserves_slovak_profile_semantics() {
    assert_eq!(
        inferred("funkcia result() { nech x = 2 vrat x }", "sk"),
        ReturnType::Int
    );
}

#[test]
fn inference_never_claims_complete_return_when_branch_falls_through() {
    assert_eq!(
        inferred("fn result() { if true { return 1 } }", "ai"),
        ReturnType::Unknown
    );
}

#[test]
fn inference_rejects_unsupported_loop_flow_conservatively() {
    assert_eq!(
        inferred("fn result() { while false { return 1 } return 2 }", "ai"),
        ReturnType::Unknown
    );
}

#[test]
fn inference_does_not_run_or_depend_on_python() {
    let program = parse("fn result(x) { return x }", "ai").unwrap();
    assert_eq!(infer_function_return(&program.statements[0]), ReturnType::Unknown);
}
