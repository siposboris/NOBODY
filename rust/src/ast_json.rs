//! Python-compatible nobody.ast.v1 JSON export for a verified source subset.
//! Rust parser nodes do not yet carry source spans. For multiline input a
//! token-aligned visitor recovers real line numbers or rejects unsupported
//! constructs, rather than fabricating them. No code is executed; JSON
//! imports are NOT implemented.
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

fn block(body: &[Statement]) -> String {
    bare("block", body.iter().map(stmt).collect())
}

fn stmt(statement: &Statement) -> String {
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
                .map(|(condition, body)| bare("branch", vec![expr(condition), block(body)]))
                .collect();
            if let Some(body) = otherwise {
                children.push(bare("else", body.iter().map(stmt).collect()));
            }
            bare("if", children)
        }
        Statement::While { condition, body } => bare("while", vec![expr(condition), block(body)]),
        Statement::For {
            variable,
            iterable,
            body,
        } => node("for", quoted(variable), vec![expr(iterable), block(body)]),
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            let params = parameters
                .iter()
                .map(|name| quoted(name))
                .collect::<Vec<_>>();
            node(
                "function",
                format!("[{},[{}]]", quoted(name), params.join(",")),
                body.iter().map(stmt).collect(),
            )
        }
        Statement::Return(value) => bare("return", vec![expr(value)]),
        Statement::Break => bare("break", vec![]),
        Statement::Continue => bare("continue", vec![]),
        Statement::Expression(value) => bare("expression_statement", vec![expr(value)]),
    }
}

/// Export the supported subset to deterministic nobody.ast.v1 JSON.
/// A token-aligned position map recovers multiline node locations when safe;
/// otherwise return N4004 rather than reporting fabricated line information.
pub fn export_ast_json(source: &str, profile: &str) -> Result<String, Diagnostic> {
    let program = parse_validated(source, profile)?;
    let tree = bare("module", program.statements.iter().map(stmt).collect());
    let json = format!(r#"{{"ast":{tree},"schema":"nobody.ast.v1"}}"#);
    if !source.contains('\n') && !source.contains('\r') {
        return Ok(json);
    }

    // JSON v1 uses stable sorted keys, so child nodes occur before their
    // parent's "line" field. The location visitor yields that same postorder.
    // Replacing only the serializer-owned exact field marker cannot affect
    // escaped string literal contents, e.g. \"line\":1 in a quoted value.
    let markers = crate::ast_json_locations::source_lines(
        source,
        profile,
        &program.statements,
    )?;
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
