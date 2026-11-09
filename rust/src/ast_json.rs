//! Python-compatible nobody.ast.v1 JSON export for a verified source subset.
//! Rust parser nodes do not yet carry source spans. For multiline input a
//! token-aligned visitor recovers real line numbers or rejects unsupported
//! constructs, rather than fabricating them. No code is executed; JSON
//! imports are NOT implemented. Version 2 preserves typed function signatures.
use crate::syntax::{Expr, Statement};
use crate::{parse_validated, Diagnostic};

fn unsupported(detail: &str) -> Diagnostic {
    Diagnostic {
        code: 4004,
        line: 0,
        column: 0,
        detail: detail.to_owned(),
        repair: "Use a single-line source for Rust ast-json; multiline source spans and AST import are not yet implemented.",
    }
}

fn quoted(text: &str) -> String {
    let mut output = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            c if c < '\u{20}' => output.push_str(&format!("\\u{:04x}", c as u32)),
            c => output.push(c),
        }
    }
    output.push('"');
    output
}

fn node(kind: &str, value_json: String, children: Vec<String>) -> String {
    format!(
        r#"{{"children":[{}],"kind":{},"line":1,"value":{}}}"#,
        children.join(","),
        quoted(kind),
        value_json
    )
}

fn bare(kind: &str, children: Vec<String>) -> String {
    node(kind, "null".into(), children)
}

fn expr(input: &Expr) -> String {
    match input {
        Expr::Number(value) => node("number", quoted(value), vec![]),
        Expr::String(value) => node("string", quoted(value), vec![]),
        Expr::Boolean(value) => node(
            "literal",
            quoted(if *value { "True" } else { "False" }),
            vec![],
        ),
        Expr::Null => node("literal", quoted("None"), vec![]),
        Expr::Name(name) => node("name", quoted(name), vec![]),
        Expr::Call { callee, args } => {
            let mut children = vec![expr(callee)];
            children.extend(args.iter().map(expr));
            bare("call", children)
        }
        Expr::Unary { op, value } => node("unary", quoted(op), vec![expr(value)]),
        Expr::Binary { op, lhs, rhs } => node("binary", quoted(op), vec![expr(lhs), expr(rhs)]),
    }
}

fn block(body: &[Statement], version: u8) -> String {
    bare(
        "block",
        body.iter()
            .map(|statement| stmt(statement, version))
            .collect(),
    )
}

fn stmt(statement: &Statement, version: u8) -> String {
    match statement {
        Statement::Let {
            name,
            type_name,
            expr: value,
        } => match type_name {
            Some(ty) => node(
                "typed_let",
                format!("[{},{}]", quoted(name), quoted(ty)),
                vec![expr(value)],
            ),
            None => node("let", quoted(name), vec![expr(value)]),
        },
        Statement::Assign { name, expr: value } => node("assign", quoted(name), vec![expr(value)]),
        Statement::If {
            branches,
            otherwise,
        } => {
            let mut children: Vec<_> = branches
                .iter()
                .map(|(condition, body)| {
                    bare("branch", vec![expr(condition), block(body, version)])
                })
                .collect();
            if let Some(body) = otherwise {
                children.push(bare(
                    "else",
                    body.iter()
                        .map(|statement| stmt(statement, version))
                        .collect(),
                ));
            }
            bare("if", children)
        }
        Statement::While { condition, body } => {
            bare("while", vec![expr(condition), block(body, version)])
        }
        Statement::For {
            variable,
            iterable,
            body,
        } => node(
            "for",
            quoted(variable),
            vec![expr(iterable), block(body, version)],
        ),
        Statement::Function {
            name,
            parameters,
            parameter_types,
            return_type,
            body,
        } => {
            let params = parameters
                .iter()
                .map(|name| quoted(name))
                .collect::<Vec<_>>();
            // V2 adds positional types and the return type. Null means
            // unspecified; the old V1 layout remains byte-for-byte stable.
            let value = if version == 2 {
                let types = parameter_types
                    .iter()
                    .map(|ty| {
                        ty.as_ref()
                            .map_or_else(|| "null".into(), |name| quoted(name))
                    })
                    .collect::<Vec<_>>();
                let result = return_type
                    .as_ref()
                    .map_or_else(|| "null".into(), |name| quoted(name));
                format!(
                    "[{},[{}],[{}],{}]",
                    quoted(name),
                    params.join(","),
                    types.join(","),
                    result
                )
            } else {
                format!("[{},[{}]]", quoted(name), params.join(","))
            };
            node(
                "function",
                value,
                body.iter()
                    .map(|statement| stmt(statement, version))
                    .collect(),
            )
        }
        Statement::Return(value) => bare("return", vec![expr(value)]),
        Statement::Break => bare("break", vec![]),
        Statement::Continue => bare("continue", vec![]),
        Statement::Expression(value) => bare("expression_statement", vec![expr(value)]),
    }
}

// AST JSON v1 cannot carry typed function signatures. Do not erase them.
fn has_typed_function(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::Function {
            parameter_types,
            return_type,
            body,
            ..
        } => {
            parameter_types.iter().any(Option::is_some)
                || return_type.is_some()
                || has_typed_function(body)
        }
        Statement::If {
            branches,
            otherwise,
        } => {
            branches.iter().any(|(_, body)| has_typed_function(body))
                || otherwise
                    .as_ref()
                    .is_some_and(|body| has_typed_function(body))
        }
        Statement::While { body, .. } | Statement::For { body, .. } => has_typed_function(body),
        _ => false,
    })
}

/// Export the supported subset to deterministic nobody.ast.v1 JSON.
/// Typed function signatures deliberately remain unsupported in V1.
pub fn export_ast_json(source: &str, profile: &str) -> Result<String, Diagnostic> {
    export_ast_json_versioned(source, profile, 1)
}

/// Export nobody.ast.v2: V1 nodes plus lossless primitive function signatures.
/// This is export-only, not a validated JSON importer or a stable AST ABI.
pub fn export_ast_json_v2(source: &str, profile: &str) -> Result<String, Diagnostic> {
    export_ast_json_versioned(source, profile, 2)
}

/// A token-aligned visitor recovers multiline source positions when possible.
/// Reject unmappable source shapes rather than inventing location data.
fn export_ast_json_versioned(
    source: &str,
    profile: &str,
    version: u8,
) -> Result<String, Diagnostic> {
    let program = parse_validated(source, profile)?;
    if version == 1 && has_typed_function(&program.statements) {
        return Err(unsupported(
            "AST JSON v1 has no typed function signature fields; refusing to discard annotations",
        ));
    }
    let tree = bare(
        "module",
        program
            .statements
            .iter()
            .map(|statement| stmt(statement, version))
            .collect(),
    );
    let json = format!(r#"{{"ast":{tree},"schema":"nobody.ast.v{version}"}}"#);
    if !source.contains('\n') && !source.contains('\r') {
        return Ok(json);
    }

    // JSON v1 uses stable sorted keys, so child nodes occur before their
    // parent's "line" field. The location visitor yields that same postorder.
    // Replacing only the serializer-owned exact field marker cannot affect
    // escaped string literal contents, e.g. \"line\":1 in a quoted value.
    let markers = crate::ast_json_locations::source_lines(source, profile, &program.statements)?;
    let parts: Vec<_> = json.split(r#""line":1"#).collect();
    if parts.len() != markers.len() + 1 {
        return Err(unsupported("AST JSON line/node count mismatch"));
    }
    let mut result = String::with_capacity(json.len() + markers.len() * 2);
    for (index, part) in parts.iter().enumerate() {
        result.push_str(part);
        if let Some(line) = markers.get(index) {
            result.push_str(r#""line":"#);
            result.push_str(&line.to_string());
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_preserves_function_types_and_multiline_lines() {
        let source =
            "fn choose(flag: bool, count: int): bool {\n return flag\n}\nprint(choose(true, 2))";
        let output = export_ast_json_v2(source, "ai").unwrap();
        assert!(output.contains(r#""schema":"nobody.ast.v2""#));
        assert!(output.contains(r#""value":["choose",["flag","count"],["bool","int"],"bool"]"#));
        assert!(output.contains(r#""kind":"return","line":2"#));
        assert!(output.contains(r#""kind":"expression_statement","line":4"#));
        let untyped = export_ast_json_v2("fn identity(value) { return value }", "ai").unwrap();
        assert!(untyped.contains(r#""value":["identity",["value"],[null],null]"#));
    }

    #[test]
    fn v2_canonical_type_names_match_across_syntax_profiles() {
        let ai = export_ast_json_v2("fn identity(flag: bool): bool { return flag }", "ai").unwrap();
        let sk = export_ast_json_v2(
            "funkcia identity(flag: logicke): logicke { vrat flag }",
            "sk",
        )
        .unwrap();
        let en = export_ast_json_v2("function identity(flag: bool): bool { return flag }", "en")
            .unwrap();
        assert_eq!(ai, sk);
        assert_eq!(ai, en);
        assert!(export_ast_json("fn identity(flag: bool): bool { return flag }", "ai").is_err());
    }

    #[test]
    fn typed_function_signatures_never_silently_disappear_from_ast_v1() {
        let source = "fn choose(flag: bool): bool { return flag }";
        assert_eq!(export_ast_json(source, "ai").unwrap_err().code, 4004);
        let nested = "if true { fn choose(flag: bool): bool { return flag } }";
        assert_eq!(export_ast_json(nested, "ai").unwrap_err().code, 4004);
        let old = export_ast_json("fn choose(flag) { return flag }", "ai").unwrap();
        assert!(old.contains("\"function\""));
    }

    #[test]
    fn emits_exact_sorted_python_reference_shape() {
        let output = export_ast_json("nech n = 3", "sk").unwrap();
        assert_eq!(
            output,
            r#"{"ast":{"children":[{"children":[{"children":[],"kind":"number","line":1,"value":"3"}],"kind":"let","line":1,"value":"n"}],"kind":"module","line":1,"value":null},"schema":"nobody.ast.v1"}"#
        );
    }

    #[test]
    fn exports_correct_multiline_source_positions() {
        let result = export_ast_json("nech n = 3\nvypis(n)", "sk").unwrap();
        assert!(result.contains(r#""kind":"let","line":1"#));
        assert!(result.contains(r#""kind":"expression_statement","line":2"#));
        assert!(result.contains(r#""kind":"call","line":2"#));
    }

    #[test]
    fn unicode_and_quotes_are_preserved_in_json() {
        let output = export_ast_json(r#"vypis("Život \"a\"")"#, "sk").unwrap();
        assert!(output.contains(r#""kind":"string""#));
        assert!(output.contains("Život"));
    }

    #[test]
    fn nested_constructs_export_without_executing() {
        let output =
            export_ast_json("function double(x) { return x * 2 } print(double(3))", "en").unwrap();
        assert!(output.contains(r#""kind":"function""#));
        assert!(output.contains(r#""kind":"return""#));
        assert!(output.contains(r#""kind":"call""#));
    }
}
