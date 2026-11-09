//! Convert an imported nobody.ast.v2 JSON tree into the existing Rust syntax AST.
//! This is deliberately NON-EXECUTING: JSON never invokes the evaluator or VM.
//! The importer validates structure; this layer validates profile-specific names,
//! decimal numeric literals and return/loop context before creating a Program.
use crate::ast_json_import::{import_ast_json_v2, AstNode, AstValue};
use crate::syntax::{Expr, Language, Program, Statement};
use crate::{parse_validated, profiles, scan, validate_program, Diagnostic, TokenKind};

fn invalid(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 5001,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use a valid nobody.ast.v2 document exported from the chosen Nobody syntax profile, or correct the malformed node.",
    }
}

fn text(node: &AstNode) -> Result<&str, Diagnostic> {
    if let AstValue::Text(value) = &node.value {
        Ok(value)
    } else {
        Err(invalid(format!("Missing text for {} node", node.kind)))
    }
}

fn check_name(name: &str, profile: &str) -> Result<String, Diagnostic> {
    // The source lexer applies lowercase ASCII for keywords, Unicode NFC,
    // mixed-script restrictions and valid identifier token boundaries.
    let tokens = scan(name)?;
    if tokens.len() != 1 || tokens[0].kind != TokenKind::Name || tokens[0].value != name {
        return Err(invalid(format!("Invalid Nobody identifier {name:?}")));
    }
    let semantic = profiles::semantic(profile, name);
    if matches!(
        semantic,
        Some(
            "if" | "else"
                | "elif"
                | "while"
                | "for"
                | "in"
                | "def"
                | "return"
                | "true"
                | "false"
                | "none"
                | "and"
                | "or"
                | "not"
                | "break"
                | "continue"
                | "let"
        )
    ) {
        return Err(invalid(format!(
            "Identifier {name:?} is a reserved keyword in profile {profile:?}"
        )));
    }
    if profile == "ai" && name.starts_with("ai_") && profiles::CANONICAL.contains(&&name[3..]) {
        return Err(invalid("Legacy ai_ syntax is not accepted in imported AST"));
    }
    Ok(name.to_owned())
}

fn check_number(value: &str) -> Result<String, Diagnostic> {
    let tokens = scan(value)?;
    if tokens.len() != 1 || tokens[0].kind != TokenKind::Number || tokens[0].value != value {
        return Err(invalid(format!(
            "Invalid decimal numeric literal {value:?}"
        )));
    }
    Ok(value.to_owned())
}

fn child(node: &AstNode, index: usize) -> Result<&AstNode, Diagnostic> {
    node.children
        .get(index)
        .ok_or_else(|| invalid(format!("Missing child {index} of {}", node.kind)))
}

fn expression(node: &AstNode, profile: &str) -> Result<Expr, Diagnostic> {
    match node.kind.as_str() {
        "number" => Ok(Expr::Number(check_number(text(node)?)?)),
        "string" => Ok(Expr::String(text(node)?.to_owned())),
        "literal" => match text(node)? {
            "True" => Ok(Expr::Boolean(true)),
            "False" => Ok(Expr::Boolean(false)),
            "None" => Ok(Expr::Null),
            _ => Err(invalid("Invalid AST literal")),
        },
        "name" => Ok(Expr::Name(check_name(text(node)?, profile)?)),
        "unary" => Ok(Expr::Unary {
            op: text(node)?.to_owned(),
            value: Box::new(expression(child(node, 0)?, profile)?),
        }),
        "binary" => Ok(Expr::Binary {
            op: text(node)?.to_owned(),
            lhs: Box::new(expression(child(node, 0)?, profile)?),
            rhs: Box::new(expression(child(node, 1)?, profile)?),
        }),
        "call" => {
            let mut nodes = node.children.iter();
            let callee = nodes
                .next()
                .ok_or_else(|| invalid("Call node has no callee"))?;
            Ok(Expr::Call {
                callee: Box::new(expression(callee, profile)?),
                args: nodes
                    .map(|node| expression(node, profile))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        _ => Err(invalid(format!(
            "Unexpected expression node {:?}",
            node.kind
        ))),
    }
}

fn block(node: &AstNode, profile: &str) -> Result<Vec<Statement>, Diagnostic> {
    if node.kind != "block" {
        return Err(invalid("Expected AST block node"));
    }
    statements(&node.children, profile)
}

fn statements(nodes: &[AstNode], profile: &str) -> Result<Vec<Statement>, Diagnostic> {
    nodes
        .iter()
        .map(|node| statement(node, profile))
        .collect::<Result<Vec<_>, _>>()
}

fn statement(node: &AstNode, profile: &str) -> Result<Statement, Diagnostic> {
    match node.kind.as_str() {
        "let" | "typed_let" => {
            let (name, ty) = match &node.value {
                AstValue::Text(name) => (name.clone(), None),
                AstValue::TypedLet { name, type_name } => (name.clone(), Some(type_name.clone())),
                _ => return Err(invalid("Missing declaration name/type")),
            };
            Ok(Statement::Let {
                name: check_name(&name, profile)?,
                type_name: ty,
                expr: expression(child(node, 0)?, profile)?,
            })
        }
        "assign" => Ok(Statement::Assign {
            name: check_name(text(node)?, profile)?,
            expr: expression(child(node, 0)?, profile)?,
        }),
        "if" => {
            let mut branches = Vec::new();
            let mut otherwise = None;
            for entry in &node.children {
                match entry.kind.as_str() {
                    "branch" => branches.push((
                        expression(child(entry, 0)?, profile)?,
                        block(child(entry, 1)?, profile)?,
                    )),
                    "else" => otherwise = Some(statements(&entry.children, profile)?),
                    _ => return Err(invalid("Invalid branch in if node")),
                }
            }
            Ok(Statement::If {
                branches,
                otherwise,
            })
        }
        "while" => Ok(Statement::While {
            condition: expression(child(node, 0)?, profile)?,
            body: block(child(node, 1)?, profile)?,
        }),
        "for" => Ok(Statement::For {
            variable: check_name(text(node)?, profile)?,
            iterable: expression(child(node, 0)?, profile)?,
            body: block(child(node, 1)?, profile)?,
        }),
        "function" => {
            let AstValue::Function {
                name,
                parameters,
                parameter_types,
                return_type,
            } = &node.value
            else {
                return Err(invalid("Invalid function signature"));
            };
            Ok(Statement::Function {
                name: check_name(name, profile)?,
                parameters: parameters
                    .iter()
                    .map(|name| check_name(name, profile))
                    .collect::<Result<Vec<_>, _>>()?,
                parameter_types: parameter_types.clone(),
                return_type: return_type.clone(),
                body: statements(&node.children, profile)?,
            })
        }
        "return" => Ok(Statement::Return(expression(child(node, 0)?, profile)?)),
        "break" => Ok(Statement::Break),
        "continue" => Ok(Statement::Continue),
        "expression_statement" => Ok(Statement::Expression(expression(child(node, 0)?, profile)?)),
        _ => Err(invalid(format!(
            "Unexpected statement node {:?}",
            node.kind
        ))),
    }
}

/// Convert strictly validated AST JSON v2 into the ordinary parser AST.
/// This does not execute imported programs and is not a security sandbox.
pub fn lower_ast_json_v2(json: &str, profile: &str) -> Result<Program, Diagnostic> {
    let language = Language::from_code(profile)?;
    let tree = import_ast_json_v2(json)?;
    let program = Program {
        language,
        statements: statements(&tree.root.children, profile)?,
    };
    validate_program(&program)?;
    Ok(program)
}

/// Prove that the imported JSON tree yields the same *unspanned* Rust AST as
/// the source parser. Position differences are not compared here.
pub fn same_ast_json_v2(source: &str, profile: &str, json: &str) -> Result<bool, Diagnostic> {
    let expected = parse_validated(source, profile)?;
    let actual = lower_ast_json_v2(json, profile)?;
    Ok(expected == actual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_ast_json_v2;

    #[test]
    fn typed_source_roundtrips_to_exact_rust_syntax_ast() {
        for (profile, source) in [
            ("ai", "fn choose(flag: bool, a: int): int { if flag { return a } else { return 2 } } print(choose(true, 4))"),
            ("sk", "funkcia choose(flag: logicke, a: cele): cele { ak flag { vrat a } inak { vrat 2 } } vypis(choose(pravda, 4))"),
            ("en", "function choose(flag: bool, a: int): int { if flag { return a } else { return 2 } } print(choose(true, 4))"),
        ] {
            let json = export_ast_json_v2(source, profile).unwrap();
            let expected = parse_validated(source, profile).unwrap();
            let actual = lower_ast_json_v2(&json, profile).unwrap();
            assert_eq!(actual, expected, "{profile}");
            assert!(same_ast_json_v2(source, profile, &json).unwrap());
        }
    }

    #[test]
    fn nested_loops_and_unicode_names_roundtrip() {
        let src = "nech pocet = 0\npre číslo v rozsah(3) {\n ak číslo > 1 { pocet = pocet + číslo }\n}\nvypis(pocet)";
        let json = export_ast_json_v2(src, "sk").unwrap();
        assert!(same_ast_json_v2(src, "sk", &json).unwrap());
    }

    #[test]
    fn rejects_reserved_or_invalid_names_and_numbers() {
        let source = "let test = 3\nprint(test)";
        let json = export_ast_json_v2(source, "ai").unwrap();
        let reserved = json.replace(r#""value":"test""#, r#""value":"if""#);
        assert_eq!(lower_ast_json_v2(&reserved, "ai").unwrap_err().code, 5001);
        let malformed = json.replace(r#""value":"3""#, r#""value":"3bad""#);
        assert!(lower_ast_json_v2(&malformed, "ai").is_err());
    }

    #[test]
    fn prevents_escaping_function_and_loop_context() {
        let source = "print(3)";
        let json = export_ast_json_v2(source, "ai").unwrap();
        let forged = json.replace(r#""kind":"expression_statement""#, r#""kind":"return""#);
        assert_eq!(lower_ast_json_v2(&forged, "ai").unwrap_err().code, 3008);
    }

    #[test]
    fn refuses_semantically_different_import() {
        let src = "let test = 3";
        let json = export_ast_json_v2(src, "ai").unwrap();
        let changed = json.replace(r#""value":"3""#, r#""value":"4""#);
        assert!(!same_ast_json_v2(src, "ai", &changed).unwrap());
    }
}
