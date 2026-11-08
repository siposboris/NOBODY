//! Context validation for the Nobody Rust AST subset.
//! This pass does not execute code, infer types or claim runtime safety.
use crate::syntax::{Program, Statement};
use crate::Diagnostic;

fn diagnostic(code: u16, detail: &str, repair: &'static str) -> Diagnostic {
    // The AST has no per-node spans yet. Until spans are added, position 1:1
    // is deliberately documented as unavailable rather than fabricated.
    Diagnostic {
        code,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair,
    }
}

/// Validate whether return/break/continue occur in a legal lexical context.
/// A nested function resets the loop context even when defined inside a loop.
pub fn validate_program(program: &Program) -> Result<(), Diagnostic> {
    validate_statements(&program.statements, false, false)
}

fn validate_statements(
    statements: &[Statement],
    in_function: bool,
    in_loop: bool,
) -> Result<(), Diagnostic> {
    for stmt in statements {
        match stmt {
            Statement::Let { .. } | Statement::Assign { .. } | Statement::Expression(_) => {}
            Statement::Return(_) if !in_function => {
                return Err(diagnostic(
                    3008,
                    "Return outside a function body",
                    "Move the return inside a function declaration or remove it.",
                ));
            }
            Statement::Break | Statement::Continue if !in_loop => {
                return Err(diagnostic(
                    3009,
                    "Break or continue outside a loop in the current function",
                    "Move the instruction inside a while/for loop in this function.",
                ));
            }
            Statement::If {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    validate_statements(body, in_function, in_loop)?;
                }
                if let Some(body) = otherwise {
                    validate_statements(body, in_function, in_loop)?;
                }
            }
            Statement::While { body, .. } | Statement::For { body, .. } => {
                validate_statements(body, in_function, true)?;
            }
            Statement::Function { body, .. } => {
                // Functions may escape and run after the surrounding loop exits.
                validate_statements(body, true, false)?;
            }
            Statement::Return(_) | Statement::Break | Statement::Continue => {}
        }
    }
    Ok(())
}

/// Parse and check a supported subset of Nobody.
pub fn parse_validated(source: &str, language: &str) -> Result<Program, Diagnostic> {
    let program = crate::syntax::parse(source, language)?;
    validate_program(&program)?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_top_level_return_sk() {
        let error = parse_validated("vrat 10", "sk").unwrap_err();
        assert_eq!(error.code, 3008);
        assert!(error.repair.contains("function"));
    }
    #[test]
    fn reject_top_level_break_ai() {
        let error = parse_validated("break", "ai").unwrap_err();
        assert_eq!(error.code, 3009);
    }
    #[test]
    fn reject_top_level_continue_en() {
        let error = parse_validated("continue", "en").unwrap_err();
        assert_eq!(error.code, 3009);
    }
    #[test]
    fn allow_return_in_nested_function() {
        parse_validated(
            "funkcia outer() { funkcia inner() { vrat 2 } vrat 1 }",
            "sk",
        )
        .unwrap();
    }
    #[test]
    fn allow_break_continue_in_nested_if_with_loop() {
        parse_validated(
            "pokial pravda { ak pravda { pokracuj } inak { prerus } }",
            "sk",
        )
        .unwrap();
    }
    #[test]
    fn reject_return_inside_top_level_if() {
        let error = parse_validated("ak pravda { vrat 3 }", "sk").unwrap_err();
        assert_eq!(error.code, 3008);
    }
    #[test]
    fn reject_break_inside_nested_function_declared_in_loop() {
        let error = parse_validated("pokial pravda { funkcia f() { prerus } }", "sk").unwrap_err();
        assert_eq!(error.code, 3009);
    }
    #[test]
    fn reject_continue_inside_nested_function_declared_in_loop() {
        let error =
            parse_validated("pre n v rozsah(2) { funkcia f() { pokracuj } }", "sk").unwrap_err();
        assert_eq!(error.code, 3009);
    }
    #[test]
    fn allow_function_loop_control_when_function_has_own_loop() {
        parse_validated(
            "funkcia f() { pre n v rozsah(2) { ak pravda { prerus } } vrat 0 }",
            "sk",
        )
        .unwrap();
    }
    #[test]
    fn allow_break_in_ai_loop() {
        parse_validated("while true { break }", "ai").unwrap();
    }
}
