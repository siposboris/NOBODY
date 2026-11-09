//! Conservative Rust type checking for canonical Nobody IR.
//! Unknown expressions are deferred; this is NOT a sound whole-program type system.
use crate::ir::{IrExpr, IrStatement, Module, Name};
use crate::{lower_source, Diagnostic};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
    Null,
    Unknown,
}
impl Type {
    fn from_annotation(name: &str) -> Self {
        match name {
            "int" => Self::Int,
            "float" => Self::Float,
            "str" => Self::Str,
            "bool" => Self::Bool,
            _ => Self::Unknown,
        }
    }
}
fn diagnostic(code: u16, detail: String, repair: &'static str) -> Diagnostic {
    // IR v1 does not carry statement spans; do not fabricate line numbers.
    Diagnostic {
        code,
        line: 0,
        column: 0,
        detail,
        repair,
    }
}
fn compatible(expected: Type, actual: Type) -> bool {
    expected == Type::Unknown
        || actual == Type::Unknown
        || expected == actual
        || (expected == Type::Float && actual == Type::Int)
}
fn infer(expr: &IrExpr, env: &BTreeMap<String, Type>) -> Type {
    match expr {
        IrExpr::Number(n) => {
            if n.contains('.') || n.contains('e') || n.contains('E') {
                Type::Float
            } else {
                Type::Int
            }
        }
        IrExpr::String(_) => Type::Str,
        IrExpr::Boolean(_) => Type::Bool,
        IrExpr::Null => Type::Null,
        IrExpr::Name(Name::User(name)) => *env.get(name).unwrap_or(&Type::Unknown),
        IrExpr::Name(Name::Builtin(_)) | IrExpr::Call { .. } => Type::Unknown,
        IrExpr::Unary { op, value } => {
            let t = infer(value, env);
            if op == "not" {
                Type::Bool
            } else if (op == "-" || op == "+") && matches!(t, Type::Int | Type::Float) {
                t
            } else {
                Type::Unknown
            }
        }
        IrExpr::Binary { op, lhs, rhs } => {
            let a = infer(lhs, env);
            let b = infer(rhs, env);
            if matches!(
                op.as_str(),
                "==" | "!=" | "<" | ">" | "<=" | ">=" | "and" | "or"
            ) {
                return Type::Bool;
            }
            if op == "+" && a == Type::Str && b == Type::Str {
                return Type::Str;
            }
            if matches!(op.as_str(), "+" | "-" | "*" | "/" | "//" | "%" | "**")
                && matches!(a, Type::Int | Type::Float)
                && matches!(b, Type::Int | Type::Float)
            {
                if op == "/" || a == Type::Float || b == Type::Float {
                    Type::Float
                } else {
                    Type::Int
                }
            } else {
                Type::Unknown
            }
        }
    }
}
/// Check only syntactically certain zero numeric literals, including unary
/// signs. Do not use floating-point parsing: nonzero values such as 1e-9999
/// could underflow to zero in f64 without being a literal zero.
fn definitely_zero_literal(expr: &IrExpr) -> bool {
    match expr {
        IrExpr::Number(value) => {
            let mantissa = value.split(['e', 'E']).next().unwrap_or(value);
            let mut has_digit = false;
            for ch in mantissa.chars() {
                match ch {
                    '0' => has_digit = true,
                    '.' | '_' => {}
                    '1'..='9' => return false,
                    _ => return false,
                }
            }
            has_digit
        }
        IrExpr::Unary { op, value } if op == "+" || op == "-" => definitely_zero_literal(value),
        _ => false,
    }
}

/// Evaluate a deliberately narrow subset of exact integer constants.
/// Overflow, names, calls, decimal fractions, exponents and division remain
/// unknown rather than risking false positive diagnostics.
fn exact_integer_constant(expr: &IrExpr) -> Option<i128> {
    match expr {
        IrExpr::Number(value) => {
            if !value.bytes().all(|ch| ch.is_ascii_digit() || ch == b'_') {
                return None;
            }
            value.replace('_', "").parse::<i128>().ok()
        }
        IrExpr::Unary { op, value } => {
            let n = exact_integer_constant(value)?;
            match op.as_str() {
                "+" => Some(n),
                "-" => n.checked_neg(),
                _ => None,
            }
        }
        IrExpr::Binary { op, lhs, rhs } => {
            let a = exact_integer_constant(lhs)?;
            let b = exact_integer_constant(rhs)?;
            match op.as_str() {
                "+" => a.checked_add(b),
                "-" => a.checked_sub(b),
                "*" => a.checked_mul(b),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Reject only definitely incompatible primitive arithmetic.
/// Dynamic values are intentionally deferred.
fn check_binary_types(expr: &IrExpr, env: &BTreeMap<String, Type>) -> Result<(), Diagnostic> {
    match expr {
        IrExpr::Binary { op, lhs, rhs } => {
            check_binary_types(lhs, env)?;
            check_binary_types(rhs, env)?;
            let a = infer(lhs, env);
            let b = infer(rhs, env);
            let known = a != Type::Unknown && b != Type::Unknown;
            let numeric = |t: Type| matches!(t, Type::Int | Type::Float);
            let allowed = if op == "+" {
                (numeric(a) && numeric(b)) || (a == Type::Str && b == Type::Str)
            } else if matches!(op.as_str(), "-" | "*" | "/" | "//" | "%" | "**") {
                numeric(a) && numeric(b)
            } else {
                true
            };
            if known && !allowed {
                return Err(diagnostic(1002,
                    format!("Operator {op:?} cannot combine known types {a:?} and {b:?}"),
                    "Use compatible numeric operands for arithmetic, or two strings for text concatenation with '+'."));
            }
            if matches!(op.as_str(), "/" | "//" | "%")
                && (definitely_zero_literal(rhs) || exact_integer_constant(rhs) == Some(0))
            {
                return Err(diagnostic(
                    1008,
                    format!("Operator {op:?} uses a provably zero constant divisor"),
                    "Replace the zero divisor with a nonzero value, or validate dynamic divisors before division.",
                ));
            }
        }
        IrExpr::Call { callee, args } => {
            check_binary_types(callee, env)?;
            for arg in args {
                check_binary_types(arg, env)?;
            }
        }
        IrExpr::Unary { op, value } => {
            check_binary_types(value, env)?;
            let operand = infer(value, env);
            if matches!(op.as_str(), "+" | "-")
                && operand != Type::Unknown
                && !matches!(operand, Type::Int | Type::Float)
            {
                return Err(diagnostic(
                    1006,
                    format!(
                        "Unary operator {op:?} requires a numeric operand, but found {operand:?}"
                    ),
                    "Use a number with unary '+' or '-', or convert the operand to a numeric type.",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}
/// Verify calls to statically known user-defined functions only.
/// Unknown and dynamic callees remain deferred.
fn check_calls(expr: &IrExpr, functions: &BTreeMap<String, usize>) -> Result<(), Diagnostic> {
    match expr {
        IrExpr::Call { callee, args } => {
            if let IrExpr::Name(Name::User(name)) = callee.as_ref() {
                if let Some(expected) = functions.get(name) {
                    if *expected != args.len() {
                        return Err(diagnostic(1005,
                            format!("Function {name:?} expects {expected} argument(s), but call supplies {}",args.len()),
                            "Supply exactly the number of arguments in the function declaration, or correct that declaration."));
                    }
                }
            }
            check_calls(callee, functions)?;
            for arg in args {
                check_calls(arg, functions)?;
            }
        }
        IrExpr::Unary { value, .. } => check_calls(value, functions)?,
        IrExpr::Binary { lhs, rhs, .. } => {
            check_calls(lhs, functions)?;
            check_calls(rhs, functions)?;
        }
        _ => {}
    }
    Ok(())
}
/// Invalidate only writes to enclosing bindings that may execute in a
/// branch or loop. Local declarations, loop variables and function bodies
/// must never be mistaken for assignments to an outer function.
fn invalidate_possible_outer_writes(
    statements: &[IrStatement],
    functions: &mut BTreeMap<String, usize>,
    inherited_shadows: &BTreeSet<String>,
) {
    let mut shadows = inherited_shadows.clone();
    for stmt in statements {
        match stmt {
            IrStatement::Let { name, .. } | IrStatement::Function { name, .. } => {
                shadows.insert(name.clone());
            }
            IrStatement::Assign { name, .. } => {
                if !shadows.contains(name) {
                    functions.remove(name);
                }
            }
            IrStatement::If {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    invalidate_possible_outer_writes(body, functions, &shadows);
                }
                if let Some(body) = otherwise {
                    invalidate_possible_outer_writes(body, functions, &shadows);
                }
            }
            IrStatement::While { body, .. } => {
                invalidate_possible_outer_writes(body, functions, &shadows);
            }
            IrStatement::For { variable, body, .. } => {
                let mut loop_shadows = shadows.clone();
                loop_shadows.insert(variable.clone());
                invalidate_possible_outer_writes(body, functions, &loop_shadows);
            }
            // A function declaration does not immediately execute its body.
            _ => {}
        }
    }
}
fn block(
    statements: &[IrStatement],
    env: &mut BTreeMap<String, Type>,
    functions: &mut BTreeMap<String, usize>,
) -> Result<(), Diagnostic> {
    let mut declared_here = BTreeSet::new();
    for stmt in statements {
        match stmt {
            IrStatement::Let {
                name,
                type_name,
                expr,
            } => {
                if !declared_here.insert(name.clone()) {
                    return Err(diagnostic(1004,
                        format!("Duplicate declaration of {name:?} in the same lexical block"),
                        "Rename the second declaration, or use assignment without 'let' to update the existing variable."));
                }
                check_calls(expr, functions)?;
                check_binary_types(expr, env)?;
                let found = infer(expr, env);
                // An explicitly written but unknown primitive annotation must
                // not silently become dynamic typing. Profiles are normalized
                // to the canonical primitive names during IR lowering.
                let declared = match type_name.as_deref() {
                    Some(annotation) => {
                        let candidate = Type::from_annotation(annotation);
                        if candidate == Type::Unknown {
                            return Err(diagnostic(
                                1007,
                                format!("Variable {name:?} has unknown type annotation {annotation:?}"),
                                "Use a supported primitive annotation: int, float, str or bool (or its keyword in the selected syntax profile).",
                            ));
                        }
                        candidate
                    }
                    None => found,
                };
                if !compatible(declared, found) {
                    return Err(diagnostic(1001,
                        format!("Variable {name:?} declares {declared:?}, but initializer has known type {found:?}"),
                        "Change the initializer to the declared type or correct the type annotation after ':'."));
                }
                env.insert(name.clone(), declared);
                functions.remove(name);
            }
            IrStatement::Assign { name, expr } => {
                check_calls(expr, functions)?;
                check_binary_types(expr, env)?;
                if let Some(expected) = env.get(name).copied() {
                    let actual = infer(expr, env);
                    if !compatible(expected, actual) {
                        return Err(diagnostic(1001,
                            format!("Assignment to {name:?} expects {expected:?}, but expression has known type {actual:?}"),
                            "Assign a value compatible with the original declaration, or change that declaration."));
                    }
                    // A reassignment may replace a function value with an
                    // arbitrary value; its original static arity is no longer
                    // reliable. Do not report false N1005 errors afterwards.
                    functions.remove(name);
                } else {
                    return Err(diagnostic(1003,
                        format!("Assignment targets undeclared variable {name:?} in the visible lexical scope"),
                        "Declare the variable with 'let' before assigning to it, or correct the misspelled name."));
                }
            }
            IrStatement::If {
                branches,
                otherwise,
            } => {
                for (condition, body) in branches {
                    check_calls(condition, functions)?;
                    check_binary_types(condition, env)?;
                    let mut local = env.clone();
                    let mut local_functions = functions.clone();
                    block(body, &mut local, &mut local_functions)?;
                    invalidate_possible_outer_writes(body, functions, &BTreeSet::new());
                }
                if let Some(body) = otherwise {
                    let mut local = env.clone();
                    let mut local_functions = functions.clone();
                    block(body, &mut local, &mut local_functions)?;
                    invalidate_possible_outer_writes(body, functions, &BTreeSet::new());
                }
            }
            IrStatement::While { condition, body } => {
                check_calls(condition, functions)?;
                check_binary_types(condition, env)?;
                let mut local = env.clone();
                let mut local_functions = functions.clone();
                block(body, &mut local, &mut local_functions)?;
                invalidate_possible_outer_writes(body, functions, &BTreeSet::new());
            }
            IrStatement::For {
                variable,
                iterable,
                body,
            } => {
                check_calls(iterable, functions)?;
                check_binary_types(iterable, env)?;
                let mut local = env.clone();
                local.insert(variable.clone(), Type::Unknown);
                let mut local_functions = functions.clone();
                local_functions.remove(variable);
                block(body, &mut local, &mut local_functions)?;
                // The loop iterator shadows any outer signature only inside
                // the body; it must not invalidate that outer binding itself.
                if let Some(arity) = functions.get(variable).copied() {
                    local_functions.insert(variable.clone(), arity);
                }
                invalidate_possible_outer_writes(body, functions, &BTreeSet::new());
            }
            IrStatement::Function {
                name,
                parameters,
                body,
                ..
            } => {
                if !declared_here.insert(name.clone()) {
                    return Err(diagnostic(1004,
                        format!("Duplicate function/variable declaration {name:?} in the same lexical block"),
                        "Rename the function or remove the earlier declaration in this block."));
                }
                env.insert(name.clone(), Type::Unknown);
                functions.insert(name.clone(), parameters.len());
                let mut local = env.clone();
                let mut local_functions = functions.clone();
                let mut unique_parameters = BTreeSet::new();
                for param in parameters {
                    if !unique_parameters.insert(param.clone()) {
                        return Err(diagnostic(
                            1004,
                            format!("Function {name:?} repeats parameter {param:?}"),
                            "Give every parameter in the function signature a unique name.",
                        ));
                    }
                    local.insert(param.clone(), Type::Unknown);
                    local_functions.remove(param);
                }
                block(body, &mut local, &mut local_functions)?;
            }
            IrStatement::Return(expr) | IrStatement::Expression(expr) => {
                check_calls(expr, functions)?;
                check_binary_types(expr, env)?;
            }
            IrStatement::Break | IrStatement::Continue => {}
        }
    }
    Ok(())
}
pub fn check_ir(module: &Module) -> Result<(), Diagnostic> {
    block(
        &module.statements,
        &mut BTreeMap::new(),
        &mut BTreeMap::new(),
    )
}
pub fn check_source(source: &str, language: &str) -> Result<(), Diagnostic> {
    check_ir(&lower_source(source, language)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_known_wrong_initializer() {
        let e = check_source("let n: int = \"oops\"", "ai").unwrap_err();
        assert_eq!(e.code, 1001);
        assert!(e.detail.contains("n"));
    }
    #[test]
    fn rejects_known_wrong_assignment() {
        let e = check_source("let n: int = 1\nn = false", "ai").unwrap_err();
        assert_eq!(e.code, 1001);
        assert!(e.detail.contains("Assignment"));
    }
    #[test]
    fn accepts_compatible_numbers() {
        check_source("let x: float = 2\nx = 3.5", "ai").unwrap();
    }
    #[test]
    fn rejects_boolean_as_integer() {
        assert_eq!(
            check_source("let x: int = true", "ai").unwrap_err().code,
            1001
        );
    }
    #[test]
    fn accepts_unknown_function_results_conservatively() {
        check_source("let x: int = mystery(1)", "ai").unwrap();
    }
    #[test]
    fn detects_wrong_type_in_nested_branch() {
        let e = check_source("let x: int = 1\nif true { x = \"bad\" }", "ai").unwrap_err();
        assert_eq!(e.code, 1001);
    }
    #[test]
    fn typed_shadowing_in_inner_block_does_not_leak() {
        check_source(
            "let x: int = 1\nif true { let x: str = \"ok\" }\nx = 2",
            "ai",
        )
        .unwrap();
    }
    #[test]
    fn rejects_undeclared_assignment() {
        let err = check_source("missing = 3", "ai").unwrap_err();
        assert_eq!(err.code, 1003);
        assert!(err.detail.contains("missing"));
    }
    #[test]
    fn rejects_duplicate_local_declaration() {
        let err = check_source("let x = 1\nlet x = 2", "ai").unwrap_err();
        assert_eq!(err.code, 1004);
    }
    #[test]
    fn allows_shadowing_in_child_block() {
        check_source("let x: int = 1\nif true { let x: str = \"ok\" }", "ai").unwrap();
    }
    #[test]
    fn rejects_duplicate_function_parameter() {
        // Duplicate parameter names are rejected by the parser as N3003
        // before the IR type-checking pass can run.
        let err = check_source("fn f(x,x) { return x }", "ai").unwrap_err();
        assert_eq!(err.code, 3003);
    }
    #[test]
    fn rejects_function_and_variable_same_block() {
        let err = check_source("let f = 1\nfn f() { return 2 }", "ai").unwrap_err();
        assert_eq!(err.code, 1004);
    }
    #[test]
    fn accepts_assignment_to_visible_outer_variable() {
        check_source("let x: int = 1\nif true { x = 2 }", "ai").unwrap();
    }
    #[test]
    fn known_function_arity_mismatch_is_n1005() {
        let err = check_source("fn add(a,b) { return a+b }\nadd(1)", "ai").unwrap_err();
        assert_eq!(err.code, 1005);
        assert!(err.detail.contains("add"));
        assert!(err.detail.contains("2 argument"));
    }
    #[test]
    fn correct_function_arity_passes() {
        check_source("fn add(a,b) { return a+b }\nadd(1,2)", "ai").unwrap();
    }
    #[test]
    fn recursive_wrong_arity_is_detected() {
        assert_eq!(
            check_source("fn f(x) { return f(x,x) }", "ai")
                .unwrap_err()
                .code,
            1005
        );
    }
    #[test]
    fn unknown_dynamic_calls_are_deferred() {
        check_source("mystery(1,2,3)", "ai").unwrap();
    }
    #[test]
    fn parameter_shadowing_disables_outer_arity_check() {
        check_source("fn f(x) { return x }\nfn g(f) { return f(1,2) }", "ai").unwrap();
    }
    #[test]
    fn local_function_arity_does_not_leak_outside_block() {
        check_source("if true { fn local(x) { return x } }\nlocal(1,2)", "ai").unwrap();
    }
    #[test]
    fn reassigned_function_does_not_keep_stale_arity() {
        check_source("fn f(x) { return x }\nf = mystery\nf(1,2)", "ai").unwrap();
    }
    #[test]
    fn known_function_arity_checked_before_reassignment() {
        let err = check_source("fn f(x) { return x }\nf(1,2)\nf = mystery", "ai").unwrap_err();
        assert_eq!(err.code, 1005);
    }
    #[test]
    fn rejects_string_minus_integer() {
        assert_eq!(
            check_source("let x = \"hello\" - 1", "ai")
                .unwrap_err()
                .code,
            1002
        );
    }
    #[test]
    fn rejects_mixed_string_number_addition() {
        assert_eq!(
            check_source("let x = \"hello\" + 1", "ai")
                .unwrap_err()
                .code,
            1002
        );
    }
    #[test]
    fn rejects_incompatible_nested_expression() {
        assert_eq!(
            check_source("print(1 + (\"x\" * 2))", "ai")
                .unwrap_err()
                .code,
            1002
        );
    }
    #[test]
    fn accepts_numeric_and_text_addition() {
        check_source("let x = 2 + 3\nlet y = \"a\" + \"b\"", "ai").unwrap();
    }
    #[test]
    fn defers_unknown_arithmetic() {
        check_source("let x = unknown + 2", "ai").unwrap();
    }
    #[test]
    fn rejects_bad_binary_in_return() {
        assert_eq!(
            check_source("fn f() { return true * 2 }", "ai")
                .unwrap_err()
                .code,
            1002
        );
    }
    #[test]
    fn slovak_valid_primitive_annotations_pass() {
        check_source("nech vek: cele = 3", "sk").unwrap();
    }
    #[test]
    fn localized_float_accepts_integer_initializer() {
        check_source("nech pomer: desatinne = 2", "sk").unwrap();
    }
    #[test]
    fn rejects_negative_string() {
        assert_eq!(
            check_source("let x = -\"oops\"", "ai").unwrap_err().code,
            1006
        );
    }
    #[test]
    fn rejects_positive_boolean() {
        assert_eq!(check_source("let x = +true", "ai").unwrap_err().code, 1006);
    }
    #[test]
    fn accepts_numeric_unary_operators() {
        check_source("let x = -2\nlet y = +3.5", "ai").unwrap();
    }
    #[test]
    fn defers_unknown_unary_operand() {
        check_source("let x = -mystery", "ai").unwrap();
    }
    #[test]
    fn checks_nested_unary_in_call() {
        assert_eq!(
            check_source("print(-\"bad\")", "ai").unwrap_err().code,
            1006
        );
    }
    #[test]
    fn branch_reassignment_invalidates_stale_function_arity() {
        check_source(
            "fn f(x) { return x }\nif true { f = mystery }\nf(1,2)",
            "ai",
        )
        .unwrap();
    }
    #[test]
    fn loop_reassignment_invalidates_stale_function_arity() {
        check_source(
            "fn f(x) { return x }\nwhile false { f = mystery }\nf(1,2)",
            "ai",
        )
        .unwrap();
    }
    #[test]
    fn local_shadowing_does_not_invalidate_outer_arity() {
        assert_eq!(
            check_source("fn f(x) { return x }\nif true { let f = 1 }\nf(1,2)", "ai")
                .unwrap_err()
                .code,
            1005
        );
    }
    #[test]
    fn loop_iterator_shadowing_does_not_invalidate_outer_arity() {
        assert_eq!(
            check_source(
                "fn f(x) { return x }\nfor f in range(2) { print(f) }\nf(1,2)",
                "ai"
            )
            .unwrap_err()
            .code,
            1005
        );
    }
    #[test]
    fn nested_local_function_shadow_keeps_outer_arity() {
        assert_eq!(
            check_source(
                "fn f(x) { return x }\nif true { fn f(x,y) { return x } }\nf(1,2)",
                "ai"
            )
            .unwrap_err()
            .code,
            1005
        );
    }
    #[test]
    fn nested_reassignment_invalidates_outer_arity() {
        check_source(
            "fn f(x) { return x }\nif true { while false { f = mystery } }\nf(1,2)",
            "ai",
        )
        .unwrap();
    }
    #[test]
    fn reassignment_before_shadow_invalidates_outer_arity() {
        check_source(
            "fn f(x) { return x }\nif true { f = mystery; let f = 3 }\nf(1,2)",
            "ai",
        )
        .unwrap();
    }
    #[test]
    fn nested_function_body_does_not_immediately_mutate_outer_arity() {
        assert_eq!(
            check_source(
                "fn f(x) { return x }\nif true { fn g() { f = mystery } }\nf(1,2)",
                "ai"
            )
            .unwrap_err()
            .code,
            1005
        );
    }
    #[test]
    fn checks_slovak_annotations() {
        assert_eq!(
            check_source("nech vek: cele = \"text\"", "sk")
                .unwrap_err()
                .code,
            1001
        );
    }
}
