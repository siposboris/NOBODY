//! Conservative source emitter for the currently supported Nobody syntax AST.
//! It never executes imported AST JSON. The final source is parsed back and
//! structurally compared before being returned; unsupported shapes fail closed.
//! Source comments, original whitespace and position metadata are not retained.
use crate::ast_json_lower::lower_ast_json_v2;
use crate::profiles;
use crate::syntax::{Expr, Program, Statement};
use crate::{parse_validated, validate_program, Diagnostic};

const MAX_OUTPUT_BYTES: usize = 1_048_576;
const MAX_DEPTH: usize = 64;

fn invalid(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 5001,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use a supported Nobody AST v2 tree whose syntax can be faithfully re-emitted in the selected profile.",
    }
}

fn keyword(profile: &str, semantic: &str) -> Result<&'static str, Diagnostic> {
    profiles::word(profile, semantic).ok_or_else(|| {
        invalid(format!(
            "No localized keyword for {semantic:?} in profile {profile:?}"
        ))
    })
}

fn type_word(profile: &str, ty: &str) -> Result<&'static str, Diagnostic> {
    if !matches!(ty, "int" | "float" | "str" | "bool") {
        return Err(invalid(format!("Unsupported primitive type {ty:?}")));
    }
    keyword(profile, ty)
}

fn expression(expr: &Expr, profile: &str, depth: usize) -> Result<String, Diagnostic> {
    if depth > MAX_DEPTH {
        return Err(invalid("Expression nesting exceeds emitter limit"));
    }
    match expr {
        Expr::Number(n) | Expr::String(n) | Expr::Name(n) => Ok(n.clone()),
        Expr::Boolean(value) => Ok(keyword(profile, if *value { "true" } else { "false" })?.into()),
        Expr::Null => Ok(keyword(profile, "none")?.into()),
        Expr::Call { callee, args } => {
            let callee = expression(callee, profile, depth + 1)?;
            let arguments = args
                .iter()
                .map(|arg| expression(arg, profile, depth + 1))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("{callee}({arguments})"))
        }
        Expr::Unary { op, value } => {
            let inner = expression(value, profile, depth + 1)?;
            if op == "not" {
                Ok(format!("({} {inner})", keyword(profile, "not")?))
            } else if op == "+" || op == "-" {
                Ok(format!("({op}{inner})"))
            } else {
                Err(invalid(format!("Unsupported unary operator {op:?}")))
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let a = expression(lhs, profile, depth + 1)?;
            let b = expression(rhs, profile, depth + 1)?;
            let operator = if matches!(op.as_str(), "and" | "or") {
                keyword(profile, op)?
            } else {
                op.as_str()
            };
            // Parenthesizing every binary node preserves precedence and
            // associativity, including nested exponentiation and comparisons.
            Ok(format!("({a} {operator} {b})"))
        }
    }
}

fn block(body: &[Statement], profile: &str, indent: usize) -> Result<String, Diagnostic> {
    let mut result = String::from("{\n");
    for statement in body {
        result.push_str(&emit_statement(statement, profile, indent + 1)?);
        if result.len() > MAX_OUTPUT_BYTES {
            return Err(invalid("Emitted Nobody source exceeds 1 MiB"));
        }
    }
    result.push_str(&"  ".repeat(indent));
    result.push('}');
    Ok(result)
}

fn emit_statement(
    stmt: &Statement,
    profile: &str,
    indent: usize,
) -> Result<String, Diagnostic> {
    if indent > MAX_DEPTH {
        return Err(invalid("Statement nesting exceeds emitter limit"));
    }
    let content = match stmt {
        Statement::Let {
            name,
            type_name,
            expr,
        } => {
            let ty = type_name
                .as_ref()
                .map(|ty| type_word(profile, ty).map(|spelling| format!(": {spelling}")))
                .transpose()?
                .unwrap_or_default();
            format!(
                "{} {name}{ty} = {};",
                keyword(profile, "let")?,
                expression(expr, profile, 0)?
            )
        }
        Statement::Assign { name, expr } => {
            format!("{name} = {};", expression(expr, profile, 0)?)
        }
        Statement::If {
            branches,
            otherwise,
        } => {
            let mut branch_iter = branches.iter();
            let Some((first_condition, first_body)) = branch_iter.next() else {
                return Err(invalid("If node has no branch"));
            };
            let mut output = format!(
                "{} {} {}",
                keyword(profile, "if")?,
                expression(first_condition, profile, 0)?,
                block(first_body, profile, indent)?
            );
            for (condition, body) in branch_iter {
                output.push_str(&format!(
                    " {} {} {}",
                    keyword(profile, "elif")?,
                    expression(condition, profile, 0)?,
                    block(body, profile, indent)?
                ));
            }
            if let Some(otherwise) = otherwise {
                output.push_str(&format!(
                    " {} {}",
                    keyword(profile, "else")?,
                    block(otherwise, profile, indent)?
                ));
            }
            output
        }
        Statement::While { condition, body } => format!(
            "{} {} {}",
            keyword(profile, "while")?,
            expression(condition, profile, 0)?,
            block(body, profile, indent)?
        ),
        Statement::For {
            variable,
            iterable,
            body,
        } => format!(
            "{} {variable} {} {} {}",
            keyword(profile, "for")?,
            keyword(profile, "in")?,
            expression(iterable, profile, 0)?,
            block(body, profile, indent)?
        ),
        Statement::Function {
            name,
            parameters,
            parameter_types,
            return_type,
            body,
        } => {
            if parameters.len() != parameter_types.len() {
                return Err(invalid("Function signature parameter/type length mismatch"));
            }
            let args = parameters
                .iter()
                .zip(parameter_types.iter())
                .map(|(name, ty)| {
                    let annotation = ty
                        .as_deref()
                        .map(|ty| type_word(profile, ty).map(|word| format!(": {word}")))
                        .transpose()?
                        .unwrap_or_default();
                    Ok(format!("{name}{annotation}"))
                })
                .collect::<Result<Vec<_>, Diagnostic>>()?
                .join(", ");
            let result_type = return_type
                .as_deref()
                .map(|ty| type_word(profile, ty).map(|word| format!(": {word}")))
                .transpose()?
                .unwrap_or_default();
            format!(
                "{} {name}({args}){result_type} {}",
                keyword(profile, "def")?,
                block(body, profile, indent)?
            )
        }
        Statement::Return(value) => {
            format!("{} {};", keyword(profile, "return")?, expression(value, profile, 0)?)
        }
        Statement::Break => format!("{};", keyword(profile, "break")?),
        Statement::Continue => format!("{};", keyword(profile, "continue")?),
        Statement::Expression(value) => format!("{};", expression(value, profile, 0)?),
    };
    Ok(format!("{}{}\n", "  ".repeat(indent), content))
}

/// Emit source in the selected language from the ordinary Rust syntax tree.
/// Safety gate: Rust re-parses and validates emitted text, and the resulting
/// unspanned AST must exactly match the input AST before returning any source.
pub fn emit_program(program: &Program, profile: &str) -> Result<String, Diagnostic> {
    let language = crate::syntax::Language::from_code(profile)?;
    if language != program.language {
        return Err(invalid("AST language does not match requested output profile"));
    }
    validate_program(program)?;
    let mut source = String::new();
    for statement in &program.statements {
        source.push_str(&emit_statement(statement, profile, 0)?);
        if source.len() > MAX_OUTPUT_BYTES {
            return Err(invalid("Emitted Nobody source exceeds 1 MiB"));
        }
    }
    let reparsed = parse_validated(&source, profile)?;
    if reparsed != *program {
        return Err(invalid(
            "Source emitter refused a non-equivalent syntax AST round-trip",
        ));
    }
    Ok(source)
}

/// Parse nobody.ast.v2, lower it to Rust syntax and re-emit Nobody source.
/// This does NOT run or compile the imported JSON.
pub fn emit_ast_json_v2(json: &str, profile: &str) -> Result<String, Diagnostic> {
    let program = lower_ast_json_v2(json, profile)?;
    emit_program(&program, profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_ast_json_v2;

    fn assert_roundtrip(profile: &str, source: &str) {
        let json = export_ast_json_v2(source, profile).unwrap();
        let written = emit_ast_json_v2(&json, profile).unwrap();
        assert_eq!(
            parse_validated(source, profile).unwrap(),
            parse_validated(&written, profile).unwrap(),
            "{profile}: {written}"
        );
        assert_eq!(written, emit_ast_json_v2(&json, profile).unwrap());
    }

    #[test]
    fn typed_functions_and_branches_roundtrip() {
        for (profile, source) in [
            ("ai", "fn choose(flag: bool, a: int, b: int): int { if flag { return a } else { return b } } print(choose(true, 3, 8))"),
            ("sk", "funkcia choose(flag: logicke, a: cele, b: cele): cele { ak flag { vrat a } inak { vrat b } } vypis(choose(pravda, 3, 8))"),
            ("en", "function choose(flag: bool, a: int, b: int): int { if flag { return a } else { return b } } print(choose(true, 3, 8))"),
        ] {
            assert_roundtrip(profile, source);
        }
    }

    #[test]
    fn nested_control_flow_boolean_and_operator_precedence() {
        assert_roundtrip("ai", "let count: int = 0\nfor n in range(3) { if n > 1 { count = count + n } else { continue } }\nwhile false { break }\nprint((2 + 3) * (4 ** 2))");
        assert_roundtrip("sk", "nech výsledok: cele = 1\nak pravda a_zaroven nepravda { vypis(2) } inak_ak nepravda { vypis(3) } inak { vypis(výsledok) }");
    }

    #[test]
    fn strings_are_preserved_as_raw_source_tokens() {
        assert_roundtrip("ai", r#"let text: str = "A \"quote\" and \n escape"; print(text)"#);
        assert_roundtrip("sk", "nech text = 'Život'\nvypis(text)");
    }

    #[test]
    fn all_profiles_keep_their_own_syntax_keywords() {
        for profile in crate::profiles::PROFILES {
            let source = format!(
                "{} test = 3",
                crate::profiles::word(profile.code, "let").unwrap()
            );
            assert_roundtrip(profile.code, &source);
        }
    }

    #[test]
    fn mismatched_profiles_are_refused() {
        let source = "let n = 3";
        let program = parse_validated(source, "ai").unwrap();
        assert_eq!(emit_program(&program, "sk").unwrap_err().code, 5001);
    }

    #[test]
    fn refuses_forged_source_tokens_in_json() {
        let source = "print(3)";
        let json = export_ast_json_v2(source, "ai").unwrap();
        let corrupt = json.replace(r#""value":"print""#, r#""value":"print(1); break""#);
        assert!(emit_ast_json_v2(&corrupt, "ai").is_err());
    }
}
