//! Conservative source-line mapping for the existing, validated Rust syntax AST.
//!
//! The syntax AST is presently unspanned. This token-aligned visitor derives
//! source lines without reinterpreting or executing the program. It rejects
//! any unsupported source shape rather than inventing source locations.
//! A dedicated spanned parser is the longer-term replacement for this bridge.
use crate::profiles;
use crate::syntax::{Expr, Statement};
use crate::{scan, Diagnostic, Token, TokenKind};

fn unsupported(detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 4004,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use the supported AST JSON subset; a fully spanned Rust syntax parser is still planned.",
    }
}

/// Node lines share the JSON serializer's structural child ordering.
/// The JSON v1 root is always line 1, as in the historical Python reference.
struct Lines {
    line: usize,
    children: Vec<Lines>,
}

impl Lines {
    fn new(line: usize, children: Vec<Self>) -> Self {
        Self { line, children }
    }

    fn postorder(&self, result: &mut Vec<usize>) {
        for child in &self.children {
            child.postorder(result);
        }
        result.push(self.line);
    }
}

struct Cursor<'a> {
    tokens: Vec<Token>,
    at: usize,
    language: &'a str,
    group_depth: usize,
}

impl Cursor<'_> {
    fn peek(&self) -> Result<&Token, Diagnostic> {
        self.tokens
            .get(self.at)
            .ok_or_else(|| unsupported("Source tokens ended before the validated AST"))
    }

    fn take(&mut self) -> Result<Token, Diagnostic> {
        let token = self.peek()?.clone();
        self.at += 1;
        Ok(token)
    }

    fn exact(&mut self, expected: &str) -> Result<Token, Diagnostic> {
        let token = self.take()?;
        if token.value != expected {
            return Err(unsupported(format!(
                "Cannot locate {expected:?}: found {:?} at {}:{}",
                token.value, token.line, token.column
            )));
        }
        Ok(token)
    }

    fn keyword(&mut self, semantic: &str) -> Result<Token, Diagnostic> {
        let token = self.take()?;
        if profiles::semantic(self.language, &token.value) == Some(semantic)
            || (self.language == "en" && semantic == "def" && token.value == "def")
            || (self.language == "sk" && semantic == "while" && token.value == "kym")
        {
            Ok(token)
        } else {
            Err(unsupported(format!(
                "Cannot locate keyword {semantic:?} at {}:{}",
                token.line, token.column
            )))
        }
    }

    fn operator(&mut self, op: &str) -> Result<Token, Diagnostic> {
        if matches!(op, "not" | "and" | "or") {
            return self.keyword(op);
        }
        let mut chars = op.chars();
        let first = chars
            .next()
            .ok_or_else(|| unsupported("Empty AST operator"))?;
        let token = self.exact(&first.to_string())?;
        for rest in chars {
            self.exact(&rest.to_string())?;
        }
        Ok(token)
    }

    fn separator(&mut self) {
        if self
            .tokens
            .get(self.at)
            .is_some_and(|token| token.value == ";")
        {
            self.at += 1;
        }
    }

    fn expression(&mut self, input: &Expr) -> Result<Lines, Diagnostic> {
        // The Rust parser discards grouping parentheses. Try treating a
        // leading '(' as a wrapper around this whole AST expression first.
        // If it actually wraps only the left subexpression, restore the
        // cursor and let the structural visitor handle that child instead.
        // Preserve the expression's inner source line, just as Python does.
        if self
            .tokens
            .get(self.at)
            .is_some_and(|token| token.value == "(")
        {
            if self.group_depth >= 64 {
                return Err(unsupported(
                    "Nested grouping exceeds AST source-line budget",
                ));
            }
            let saved = self.at;
            self.at += 1;
            self.group_depth += 1;
            let grouped = self.expression(input);
            self.group_depth -= 1;
            if let Ok(lines) = grouped {
                if self
                    .tokens
                    .get(self.at)
                    .is_some_and(|token| token.value == ")")
                {
                    self.at += 1;
                    return Ok(lines);
                }
            }
            self.at = saved;
        }
        match input {
            Expr::Number(value) => {
                let token = self.exact(value)?;
                if token.kind != TokenKind::Number {
                    return Err(unsupported("Number token kind mismatch"));
                }
                Ok(Lines::new(token.line, vec![]))
            }
            Expr::String(value) => {
                let token = self.exact(value)?;
                if token.kind != TokenKind::String {
                    return Err(unsupported("String token kind mismatch"));
                }
                Ok(Lines::new(token.line, vec![]))
            }
            Expr::Name(name) => {
                let token = self.exact(name)?;
                if token.kind != TokenKind::Name {
                    return Err(unsupported("Name token kind mismatch"));
                }
                Ok(Lines::new(token.line, vec![]))
            }
            Expr::Boolean(value) => {
                let token = self.keyword(if *value { "true" } else { "false" })?;
                Ok(Lines::new(token.line, vec![]))
            }
            Expr::Null => {
                let token = self.keyword("none")?;
                Ok(Lines::new(token.line, vec![]))
            }
            Expr::Unary { op, value } => {
                let token = self.operator(op)?;
                let child = self.expression(value)?;
                Ok(Lines::new(token.line, vec![child]))
            }
            Expr::Binary { op, lhs, rhs } => {
                let left = self.expression(lhs)?;
                let token = self.operator(op)?;
                let right = self.expression(rhs)?;
                // Python AST v1 attaches a binary node to its operator line.
                Ok(Lines::new(token.line, vec![left, right]))
            }
            Expr::Call { callee, args } => {
                let first = self.expression(callee)?;
                let line = first.line;
                self.exact("(")?;
                let mut children = vec![first];
                for (index, argument) in args.iter().enumerate() {
                    if index > 0 {
                        self.exact(",")?;
                    }
                    children.push(self.expression(argument)?);
                }
                self.exact(")")?;
                Ok(Lines::new(line, children))
            }
        }
    }

    fn block_statements(&mut self, body: &[Statement]) -> Result<Vec<Lines>, Diagnostic> {
        self.exact("{")?;
        let mut nodes = Vec::with_capacity(body.len());
        for statement in body {
            nodes.push(self.statement(statement)?);
        }
        self.exact("}")?;
        Ok(nodes)
    }

    fn block(&mut self, body: &[Statement], line: usize) -> Result<Lines, Diagnostic> {
        Ok(Lines::new(line, self.block_statements(body)?))
    }

    fn statement(&mut self, input: &Statement) -> Result<Lines, Diagnostic> {
        match input {
            Statement::Let {
                name,
                type_name,
                expr,
            } => {
                let start = self.keyword("let")?;
                self.exact(name)?;
                if let Some(ty) = type_name {
                    self.exact(":")?;
                    self.keyword(ty)?;
                }
                self.exact("=")?;
                let value = self.expression(expr)?;
                self.separator();
                Ok(Lines::new(start.line, vec![value]))
            }
            Statement::Assign { name, expr } => {
                let start = self.exact(name)?;
                self.exact("=")?;
                let value = self.expression(expr)?;
                self.separator();
                Ok(Lines::new(start.line, vec![value]))
            }
            Statement::Expression(expr) => {
                let value = self.expression(expr)?;
                self.separator();
                Ok(Lines::new(value.line, vec![value]))
            }
            Statement::Return(expr) => {
                let start = self.keyword("return")?;
                let value = self.expression(expr)?;
                self.separator();
                Ok(Lines::new(start.line, vec![value]))
            }
            Statement::Break => {
                let token = self.keyword("break")?;
                self.separator();
                Ok(Lines::new(token.line, vec![]))
            }
            Statement::Continue => {
                let token = self.keyword("continue")?;
                self.separator();
                Ok(Lines::new(token.line, vec![]))
            }
            Statement::While { condition, body } => {
                let token = self.keyword("while")?;
                let condition = self.expression(condition)?;
                let block = self.block(body, token.line)?;
                Ok(Lines::new(token.line, vec![condition, block]))
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let token = self.keyword("for")?;
                self.exact(variable)?;
                self.keyword("in")?;
                let iterable = self.expression(iterable)?;
                let block = self.block(body, token.line)?;
                Ok(Lines::new(token.line, vec![iterable, block]))
            }
            Statement::Function {
                name,
                parameters,
                body,
                ..
            } => {
                let start = self.keyword("def")?;
                self.exact(name)?;
                self.exact("(")?;
                for (index, name) in parameters.iter().enumerate() {
                    if index > 0 {
                        self.exact(",")?;
                    }
                    self.exact(name)?;
                }
                self.exact(")")?;
                Ok(Lines::new(start.line, self.block_statements(body)?))
            }
            Statement::If {
                branches,
                otherwise,
            } => {
                let first = self.keyword("if")?;
                let mut nodes =
                    Vec::with_capacity(branches.len() + usize::from(otherwise.is_some()));
                for (index, (condition, body)) in branches.iter().enumerate() {
                    let line = if index == 0 {
                        first.line
                    } else {
                        // Python reference supports the direct 'elif' keyword.
                        // Rust's additional 'else if' form has no Python AST v1
                        // equivalent and must not be silently treated as one.
                        self.keyword("elif")?.line
                    };
                    let condition = self.expression(condition)?;
                    let body = self.block(body, line)?;
                    nodes.push(Lines::new(line, vec![condition, body]));
                }
                if let Some(body) = otherwise {
                    let start = self.keyword("else")?;
                    nodes.push(Lines::new(start.line, self.block_statements(body)?));
                }
                Ok(Lines::new(first.line, nodes))
            }
        }
    }
}

/// Determine the exact source line of every exported AST node.
///
/// The locations are returned in JSON postorder (children before parent),
/// matching the existing stable sorted-key JSON v1 serializer. A mismatch is
/// rejected with N4004, so no fabricated location can enter the output.
pub fn source_lines(
    source: &str,
    language: &str,
    statements: &[Statement],
) -> Result<Vec<usize>, Diagnostic> {
    let mut cursor = Cursor {
        tokens: scan(source)?
            .into_iter()
            .filter(|token| token.kind != TokenKind::Comment)
            .collect(),
        at: 0,
        language,
        group_depth: 0,
    };
    let mut roots = Vec::with_capacity(statements.len());
    for statement in statements {
        roots.push(cursor.statement(statement)?);
    }
    if cursor.at != cursor.tokens.len() {
        return Err(unsupported(format!(
            "Unmapped source tokens remain after AST traversal: {}",
            cursor.tokens.len() - cursor.at
        )));
    }
    let root = Lines::new(1, roots);
    let mut result = Vec::new();
    root.postorder(&mut result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_validated;

    #[test]
    fn maps_operators_to_their_actual_source_lines() {
        let source = "nech x = 1\nvypis(\n x +\n 2\n)";
        let tree = parse_validated(source, "sk").unwrap();
        let lines = source_lines(source, "sk", &tree.statements).unwrap();
        assert_eq!(lines, vec![1, 1, 2, 3, 4, 3, 2, 2, 1]);
    }

    #[test]
    fn maps_grouped_expressions_without_inventing_lines() {
        let source = "nech x = (\n 1 + 2\n)";
        let tree = parse_validated(source, "sk").unwrap();
        let lines = source_lines(source, "sk", &tree.statements).unwrap();
        assert_eq!(lines, vec![2, 2, 2, 1, 1]);
    }

    #[test]
    fn maps_parenthesized_left_operand_and_nested_call() {
        let source = "let x = (3 + 4) * 2\nprint((x))";
        let tree = parse_validated(source, "en").unwrap();
        let lines = source_lines(source, "en", &tree.statements).unwrap();
        assert!(lines.contains(&2));
        assert_eq!(lines.last(), Some(&1));
    }
}
