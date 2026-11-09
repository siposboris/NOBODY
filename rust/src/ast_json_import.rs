//! Strict, bounded importer for the experimental nobody.ast.v2 exchange tree.
//! This validates structure, not identifiers in a particular language profile,
//! runtime semantics, or source-code equivalence. Imported nodes are never run.
use crate::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};

const MAX_DOCUMENT_BYTES: usize = 1_048_576;
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 10_000;

fn invalid(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 5001,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Provide a valid nobody.ast.v2 document with known node kinds, canonical types and matching child shapes.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Json {
    Null,
    Text(String),
    Integer(usize),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

struct Reader<'a> {
    source: &'a str,
    at: usize,
}

impl Reader<'_> {
    fn byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.at).copied()
    }

    fn whitespace(&mut self) {
        while matches!(self.byte(), Some(b' ' | b'\n' | b'\t' | b'\r')) {
            self.at += 1;
        }
    }

    fn required(&mut self, ch: u8) -> Result<(), Diagnostic> {
        if self.byte() == Some(ch) {
            self.at += 1;
            Ok(())
        } else {
            Err(invalid(format!(
                "Expected JSON byte {ch:?} at offset {}",
                self.at
            )))
        }
    }

    fn hex4(&mut self) -> Result<u32, Diagnostic> {
        let mut code = 0u32;
        for _ in 0..4 {
            let digit = self.byte().and_then(|b| (b as char).to_digit(16));
            let digit = digit.ok_or_else(|| invalid("Invalid JSON unicode escape"))?;
            code = code * 16 + digit;
            self.at += 1;
        }
        Ok(code)
    }

    fn string(&mut self) -> Result<String, Diagnostic> {
        self.required(b'"')?;
        let mut result = String::new();
        loop {
            match self.byte() {
                None => return Err(invalid("Unterminated JSON string")),
                Some(b'"') => {
                    self.at += 1;
                    return Ok(result);
                }
                Some(b'\\') => {
                    self.at += 1;
                    let escaped = self
                        .byte()
                        .ok_or_else(|| invalid("Truncated JSON escape"))?;
                    self.at += 1;
                    let decoded = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let first = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&first) {
                                self.required(b'\\')?;
                                self.required(b'u')?;
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err(invalid("Invalid JSON surrogate pair"));
                                }
                                0x10000 + ((first - 0xd800) << 10) + low - 0xdc00
                            } else {
                                first
                            };
                            char::from_u32(scalar)
                                .ok_or_else(|| invalid("Invalid JSON Unicode scalar"))?
                        }
                        _ => return Err(invalid("Unknown JSON string escape")),
                    };
                    result.push(decoded);
                }
                Some(b) if b < 0x20 => return Err(invalid("Unescaped JSON control character")),
                Some(b) if b.is_ascii() => {
                    self.at += 1;
                    result.push(char::from(b));
                }
                Some(_) => {
                    let ch = self.source[self.at..]
                        .chars()
                        .next()
                        .ok_or_else(|| invalid("Invalid UTF-8 boundary"))?;
                    self.at += ch.len_utf8();
                    result.push(ch);
                }
            }
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(invalid("JSON nesting exceeds 64 levels"));
        }
        self.whitespace();
        match self.byte() {
            Some(b'"') => Ok(Json::Text(self.string()?)),
            Some(b'n') => {
                if self.source[self.at..].starts_with("null") {
                    self.at += 4;
                    Ok(Json::Null)
                } else {
                    Err(invalid("Invalid JSON null value"))
                }
            }
            Some(b'0'..=b'9') => {
                let start = self.at;
                if self.byte() == Some(b'0') {
                    self.at += 1;
                    if matches!(self.byte(), Some(b'0'..=b'9')) {
                        return Err(invalid("Leading zero in JSON integer"));
                    }
                } else {
                    while matches!(self.byte(), Some(b'0'..=b'9')) {
                        self.at += 1;
                    }
                }
                let number = self.source[start..self.at]
                    .parse::<usize>()
                    .map_err(|_| invalid("JSON integer exceeds platform limit"))?;
                Ok(Json::Integer(number))
            }
            Some(b'[') => {
                self.at += 1;
                let mut values = Vec::new();
                self.whitespace();
                if self.byte() == Some(b']') {
                    self.at += 1;
                    return Ok(Json::Array(values));
                }
                loop {
                    values.push(self.value(depth + 1)?);
                    self.whitespace();
                    match self.byte() {
                        Some(b',') => self.at += 1,
                        Some(b']') => {
                            self.at += 1;
                            return Ok(Json::Array(values));
                        }
                        _ => return Err(invalid("Expected comma or closing JSON array bracket")),
                    }
                }
            }
            Some(b'{') => {
                self.at += 1;
                let mut values = BTreeMap::new();
                self.whitespace();
                if self.byte() == Some(b'}') {
                    self.at += 1;
                    return Ok(Json::Object(values));
                }
                loop {
                    self.whitespace();
                    let key = self.string()?;
                    self.whitespace();
                    self.required(b':')?;
                    let value = self.value(depth + 1)?;
                    if values.insert(key, value).is_some() {
                        return Err(invalid("Duplicate JSON object key"));
                    }
                    self.whitespace();
                    match self.byte() {
                        Some(b',') => self.at += 1,
                        Some(b'}') => {
                            self.at += 1;
                            return Ok(Json::Object(values));
                        }
                        _ => return Err(invalid("Expected comma or closing JSON object brace")),
                    }
                }
            }
            _ => Err(invalid("Unsupported or malformed JSON value")),
        }
    }
}

fn object(value: Json) -> Result<BTreeMap<String, Json>, Diagnostic> {
    match value {
        Json::Object(map) => Ok(map),
        _ => Err(invalid("Expected JSON object")),
    }
}
fn array(value: Json) -> Result<Vec<Json>, Diagnostic> {
    match value {
        Json::Array(items) => Ok(items),
        _ => Err(invalid("Expected JSON array")),
    }
}
fn text(value: Json) -> Result<String, Diagnostic> {
    match value {
        Json::Text(s) => Ok(s),
        _ => Err(invalid("Expected JSON string")),
    }
}
fn field(map: &mut BTreeMap<String, Json>, key: &str) -> Result<Json, Diagnostic> {
    map.remove(key)
        .ok_or_else(|| invalid(format!("Missing required JSON field {key}")))
}
fn no_other_fields(map: &BTreeMap<String, Json>) -> Result<(), Diagnostic> {
    if map.is_empty() {
        Ok(())
    } else {
        Err(invalid("Unexpected JSON object field"))
    }
}
fn identifier(s: &str) -> Result<(), Diagnostic> {
    if s.is_empty()
        || s.len() > 256
        || s.chars().any(char::is_whitespace)
        || s.chars().any(char::is_control)
    {
        Err(invalid(
            "Invalid empty, overlong or control-containing AST identifier",
        ))
    } else {
        Ok(())
    }
}
fn primitive(value: Json) -> Result<Option<String>, Diagnostic> {
    match value {
        Json::Null => Ok(None),
        Json::Text(s) if matches!(s.as_str(), "int" | "float" | "str" | "bool") => Ok(Some(s)),
        _ => Err(invalid("Unsupported function primitive type")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstValue {
    Null,
    Text(String),
    TypedLet {
        name: String,
        type_name: String,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        parameter_types: Vec<Option<String>>,
        return_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    pub kind: String,
    pub line: usize,
    pub value: AstValue,
    pub children: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocument {
    pub root: AstNode,
    pub node_count: usize,
}

fn expression(node: &AstNode) -> bool {
    matches!(
        node.kind.as_str(),
        "number" | "string" | "literal" | "name" | "call" | "unary" | "binary"
    )
}
fn statement(node: &AstNode) -> bool {
    matches!(
        node.kind.as_str(),
        "let"
            | "typed_let"
            | "assign"
            | "if"
            | "while"
            | "for"
            | "function"
            | "return"
            | "break"
            | "continue"
            | "expression_statement"
    )
}
fn all_statements(children: &[AstNode]) -> bool {
    children.iter().all(statement)
}
fn expr_at(children: &[AstNode], at: usize) -> bool {
    children.get(at).is_some_and(expression)
}
fn block_at(children: &[AstNode], at: usize) -> bool {
    children.get(at).is_some_and(|x| x.kind == "block")
}

fn node(value: Json, depth: usize, count: &mut usize) -> Result<AstNode, Diagnostic> {
    if depth > MAX_DEPTH || *count >= MAX_NODES {
        return Err(invalid("AST depth or node budget exceeded"));
    }
    *count += 1;
    let mut map = object(value)?;
    let kind = text(field(&mut map, "kind")?)?;
    let line = match field(&mut map, "line")? {
        Json::Integer(n) if n > 0 => n,
        _ => return Err(invalid("AST line must be a positive integer")),
    };
    let raw_value = field(&mut map, "value")?;
    let raw_children = array(field(&mut map, "children")?)?;
    no_other_fields(&map)?;
    let mut children = Vec::with_capacity(raw_children.len());
    for child in raw_children {
        children.push(node(child, depth + 1, count)?);
    }

    let data = match kind.as_str() {
        "typed_let" => {
            let parts = array(raw_value)?;
            if parts.len() != 2 {
                return Err(invalid("typed_let value requires [name, type]"));
            }
            let mut parts = parts.into_iter();
            let name = text(parts.next().ok_or_else(|| invalid("Missing name"))?)?;
            identifier(&name)?;
            let ty = text(parts.next().ok_or_else(|| invalid("Missing type"))?)?;
            if !matches!(ty.as_str(), "int" | "float" | "str" | "bool") {
                return Err(invalid("Unknown typed_let primitive annotation"));
            }
            AstValue::TypedLet {
                name,
                type_name: ty,
            }
        }
        "function" => {
            let parts = array(raw_value)?;
            if parts.len() != 4 {
                return Err(invalid("V2 function value requires four fields"));
            }
            let mut parts = parts.into_iter();
            let name = text(
                parts
                    .next()
                    .ok_or_else(|| invalid("Missing function name"))?,
            )?;
            identifier(&name)?;
            let parameters = array(parts.next().ok_or_else(|| invalid("Missing parameters"))?)?
                .into_iter()
                .map(text)
                .collect::<Result<Vec<_>, _>>()?;
            let types = array(
                parts
                    .next()
                    .ok_or_else(|| invalid("Missing parameter types"))?,
            )?
            .into_iter()
            .map(primitive)
            .collect::<Result<Vec<_>, _>>()?;
            let return_type =
                primitive(parts.next().ok_or_else(|| invalid("Missing return type"))?)?;
            if parameters.len() != types.len() || parameters.len() > 256 {
                return Err(invalid("Function parameter names and type counts differ"));
            }
            let mut unique = BTreeSet::new();
            for param in &parameters {
                identifier(param)?;
                if !unique.insert(param) {
                    return Err(invalid("Duplicate function parameter name"));
                }
            }
            AstValue::Function {
                name,
                parameters,
                parameter_types: types,
                return_type,
            }
        }
        "let" | "assign" | "for" | "number" | "string" | "literal" | "name" | "unary"
        | "binary" => {
            let value = text(raw_value)?;
            if matches!(kind.as_str(), "let" | "assign" | "for" | "name") {
                identifier(&value)?;
            }
            AstValue::Text(value)
        }
        "module"
        | "block"
        | "branch"
        | "else"
        | "if"
        | "while"
        | "return"
        | "break"
        | "continue"
        | "expression_statement"
        | "call" => {
            if raw_value != Json::Null {
                return Err(invalid("AST node requires null value"));
            }
            AstValue::Null
        }
        _ => return Err(invalid(format!("Unknown AST node kind {kind:?}"))),
    };

    let good = match kind.as_str() {
        "module" | "block" | "else" | "function" => all_statements(&children),
        "branch" | "while" => {
            children.len() == 2 && expr_at(&children, 0) && block_at(&children, 1)
        }
        "for" => children.len() == 2 && expr_at(&children, 0) && block_at(&children, 1),
        "if" => {
            !children.is_empty()
                && children.first().is_some_and(|x| x.kind == "branch")
                && children.iter().enumerate().all(|(i, child)| {
                    child.kind == "branch" || (i == children.len() - 1 && child.kind == "else")
                })
        }
        "let" | "typed_let" | "assign" | "return" | "expression_statement" => {
            children.len() == 1 && expr_at(&children, 0)
        }
        "break" | "continue" | "number" | "string" | "literal" | "name" => children.is_empty(),
        "unary" => children.len() == 1 && expr_at(&children, 0),
        "binary" => children.len() == 2 && expr_at(&children, 0) && expr_at(&children, 1),
        "call" => !children.is_empty() && children.iter().all(expression),
        _ => false,
    };
    if !good {
        return Err(invalid(format!(
            "Invalid child structure for AST node {kind:?}"
        )));
    }
    if kind == "literal"
        && !matches!(&data, AstValue::Text(x) if matches!(x.as_str(), "True" | "False" | "None"))
    {
        return Err(invalid("Unsupported literal value"));
    }
    if kind == "unary"
        && !matches!(&data, AstValue::Text(x) if matches!(x.as_str(), "+" | "-" | "not"))
    {
        return Err(invalid("Unknown unary operator"));
    }
    if kind == "binary"
        && !matches!(&data, AstValue::Text(x) if matches!(x.as_str(),
            "+" | "-" | "*" | "/" | "//" | "%" | "**" | "==" | "!=" |
            "<" | ">" | "<=" | ">=" | "and" | "or"))
    {
        return Err(invalid("Unknown binary operator"));
    }
    if kind == "number"
        && !matches!(&data, AstValue::Text(x) if !x.is_empty() &&
            x.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.')))
    {
        return Err(invalid("Malformed number token in AST"));
    }
    Ok(AstNode {
        kind,
        line,
        value: data,
        children,
    })
}

/// Import and structurally validate experimental typed AST JSON v2.
/// No source is emitted and no interpreter is invoked.
pub fn import_ast_json_v2(source: &str) -> Result<AstDocument, Diagnostic> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(invalid("AST JSON input exceeds 1 MiB"));
    }
    let mut reader = Reader { source, at: 0 };
    let parsed = reader.value(0)?;
    reader.whitespace();
    if reader.at != source.len() {
        return Err(invalid("Trailing bytes after JSON document"));
    }
    let mut outer = object(parsed)?;
    if text(field(&mut outer, "schema")?)? != "nobody.ast.v2" {
        return Err(invalid("AST schema must be nobody.ast.v2"));
    }
    let ast = field(&mut outer, "ast")?;
    no_other_fields(&outer)?;
    let mut node_count = 0;
    let root = node(ast, 0, &mut node_count)?;
    if root.kind != "module" {
        return Err(invalid("AST root must be a module node"));
    }
    Ok(AstDocument { root, node_count })
}

fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000c}' => result.push_str("\\f"),
            c if c < '\u{20}' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn serialize(node: &AstNode) -> String {
    let children = node
        .children
        .iter()
        .map(serialize)
        .collect::<Vec<_>>()
        .join(",");
    let value = match &node.value {
        AstValue::Null => String::from("null"),
        AstValue::Text(s) => quote(s),
        AstValue::TypedLet { name, type_name } => format!("[{},{}]", quote(name), quote(type_name)),
        AstValue::Function {
            name,
            parameters,
            parameter_types,
            return_type,
        } => {
            let names = parameters
                .iter()
                .map(|x| quote(x))
                .collect::<Vec<_>>()
                .join(",");
            let types = parameter_types
                .iter()
                .map(|ty| {
                    ty.as_ref()
                        .map_or_else(|| String::from("null"), |x| quote(x))
                })
                .collect::<Vec<_>>()
                .join(",");
            let ret = return_type
                .as_ref()
                .map_or_else(|| String::from("null"), |x| quote(x));
            format!("[{},[{}],[{}],{}]", quote(name), names, types, ret)
        }
    };
    format!(
        r#"{{"children":[{}],"kind":{},"line":{},"value":{}}}"#,
        children,
        quote(&node.kind),
        node.line,
        value
    )
}

/// Structural JSON round-trip, not a source/semantic round-trip.
pub fn normalize_ast_json_v2(source: &str) -> Result<String, Diagnostic> {
    let parsed = import_ast_json_v2(source)?;
    Ok(format!(
        r#"{{"ast":{},"schema":"nobody.ast.v2"}}"#,
        serialize(&parsed.root)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_ast_json_v2;

    #[test]
    fn preserves_typed_signatures_from_exporter() {
        for (profile, source) in [
            ("ai", "fn same(flag: bool, n: int): bool { return flag }"),
            (
                "sk",
                "funkcia same(flag: logicke, n: cele): logicke { vrat flag }",
            ),
            (
                "en",
                "function same(flag: bool, n: int): bool { return flag }",
            ),
        ] {
            let exported = export_ast_json_v2(source, profile).unwrap();
            let imported = import_ast_json_v2(&exported).unwrap();
            assert_eq!(imported.root.children.len(), 1);
            assert!(imported.node_count > 3);
            assert_eq!(normalize_ast_json_v2(&exported).unwrap(), exported);
            assert!(
                matches!(&imported.root.children[0].value, AstValue::Function {
                parameter_types, return_type, ..
            } if parameter_types == &vec![Some("bool".into()), Some("int".into())]
                && return_type.as_deref() == Some("bool"))
            );
        }
    }

    #[test]
    fn refuses_malformed_or_untrusted_documents() {
        let good = export_ast_json_v2("fn f(x: int): int { return x }", "ai").unwrap();
        for bad in [
            good.replace("nobody.ast.v2", "nobody.ast.v1"),
            good.replace(r#""line":1"#, r#""line":0"#),
            good.replace(r#"["int"],"int""#, r#"[],"int""#),
            good.replace(r#""kind":"function""#, r#""kind":"unknown""#),
            good.replace(r#""kind":"return""#, r#""kind":"break""#),
            format!("{good} extra"),
            String::from("{\"schema\":\"nobody.ast.v2\",\"schema\":\"nobody.ast.v2\"}"),
            String::from("{\"schema\":\"nobody.ast.v2\",\"ast\":{\"kind\":\"module\"}}"),
            String::from("{\"schema\":\"nobody.ast.v2\",\"ast\":null}"),
        ] {
            assert_eq!(import_ast_json_v2(&bad).unwrap_err().code, 5001);
        }
    }

    #[test]
    fn parses_unicode_escapes_and_rejects_invalid_surrogates() {
        let src = export_ast_json_v2("print(\"Život\")", "ai").unwrap();
        let escaped = src.replace("Život", r#"\u017divot"#);
        assert_eq!(normalize_ast_json_v2(&escaped).unwrap(), src);
        assert!(import_ast_json_v2(&src.replace("Život", r#"\uD800"#)).is_err());
    }
}
