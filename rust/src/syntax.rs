//! Minimal Rust AST parser for the Nobody migration.
//! Supported: declarations, annotated declarations, assignments, calls,
//! number/string/bool/name expressions and basic binary operators.
//! Unsupported constructs are rejected with precise numbered errors.
//! This is not a compiler or executable interpreter.
use crate::{scan, Diagnostic, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    code: &'static str,
}
impl Language {
    pub fn from_code(s: &str) -> Result<Self, Diagnostic> {
        if let Some(p) = crate::profiles::profile(s) {
            Ok(Self { code: p.code })
        } else {
            Err(error(3007,1,1,format!("Unknown Nobody syntax profile {s:?}. Use a listed language code; `ai` is the canonical AI syntax")))
        }
    }
    fn let_word(self) -> &'static str {
        crate::profiles::word(self.code, "let").expect("complete profile table")
    }
    fn boolean(self, s: &str) -> Option<bool> {
        match crate::profiles::semantic(self.code, s) {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        }
    }
    fn ty(self, s: &str) -> Option<&'static str> {
        match crate::profiles::semantic(self.code, s) {
            Some("int") => Some("int"),
            Some("float") => Some("float"),
            Some("str") => Some("str"),
            Some("bool") => Some("bool"),
            _ => None,
        }
    }
    fn kw(self, semantic: &str, spelling: &str) -> bool {
        crate::profiles::semantic(self.code, spelling) == Some(semantic)
            || (self.code == "sk" && semantic == "while" && spelling == "kym")
            || (self.code == "en" && semantic == "def" && spelling == "def")
    }
    fn is_unsupported_keyword(self, spelling: &str) -> bool {
        matches!(
            crate::profiles::semantic(self.code, spelling),
            Some(
                "if" | "else"
                    | "elif"
                    | "while"
                    | "for"
                    | "in"
                    | "def"
                    | "return"
                    | "break"
                    | "continue"
                    | "let"
                    | "true"
                    | "false"
                    | "none"
                    | "and"
                    | "or"
                    | "not"
            )
        )
    }
    fn legacy_ai_keyword(self, spelling: &str) -> bool {
        self.code == "ai"
            && spelling
                .strip_prefix("ai_")
                .is_some_and(|suffix| crate::profiles::CANONICAL.contains(&suffix))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Number(String),
    String(String),
    Boolean(bool),
    Null,
    Name(String),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Binary {
        op: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let {
        name: String,
        type_name: Option<String>,
        expr: Expr,
    },
    Assign {
        name: String,
        expr: Expr,
    },
    If {
        branches: Vec<(Expr, Vec<Statement>)>,
        otherwise: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Statement>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    Return(Expr),
    Break,
    Continue,
    Expression(Expr),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub language: Language,
    pub statements: Vec<Statement>,
}

fn error(code: u16, line: usize, column: usize, detail: impl Into<String>) -> Diagnostic {
    let repair = match code {
        3001 => "Insert the required punctuation or value at the reported location.",
        3002 => "Close the current block with a matching '}' brace.",
        3003 => "Use a valid lowercase variable name after the declaration keyword.",
        3005 => "Complete the expression with a valid literal, identifier or parenthesized expression.",
        3006 => "Use only declarations, assignments and expressions in the current Rust parser subset.",
        3007 => "Choose a supported code from the 39 human profiles or ai.",
        3010 => "Use `nobody-rs migrate-ai legacy.nbd > updated.nbd`; new ai syntax uses let/fn/if/return/true.",
        _ => "Correct the reported Nobody syntax.",
    };
    Diagnostic {
        code,
        line,
        column,
        detail: detail.into(),
        repair,
    }
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
    language: Language,
}
impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }
    fn word(&self) -> Option<&str> {
        self.peek().map(|t| t.value.as_str())
    }
    fn take(&mut self) -> Option<Token> {
        let item = self.tokens.get(self.at)?.clone();
        self.at += 1;
        Some(item)
    }
    fn at_symbol(&self, s: &str) -> bool {
        self.word() == Some(s)
    }
    fn take_symbol(&mut self, s: &str) -> bool {
        if self.at_symbol(s) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn location(&self) -> (usize, usize) {
        self.peek()
            .map(|t| (t.line, t.column))
            .or_else(|| {
                self.tokens
                    .last()
                    .map(|t| (t.line, t.column + t.value.chars().count()))
            })
            .unwrap_or((1, 1))
    }
    fn expected(&self, what: &str) -> Diagnostic {
        let (line, col) = self.location();
        error(
            3001,
            line,
            col,
            format!(
                "Expected {what}, found {:?}",
                self.word().unwrap_or("end of file")
            ),
        )
    }
    fn need(&mut self, s: &str) -> Result<(), Diagnostic> {
        if self.take_symbol(s) {
            Ok(())
        } else {
            Err(self.expected(s))
        }
    }
    fn identifier(&mut self) -> Result<String, Diagnostic> {
        let (line, col) = self.location();
        match self.take() {
            Some(t)
                if t.kind == TokenKind::Name
                    && self.language.boolean(&t.value).is_none()
                    && !self.language.is_unsupported_keyword(&t.value)
                    && t.value != self.language.let_word() =>
            {
                Ok(t.value)
            }
            Some(t) => Err(error(
                3003,
                line,
                col,
                format!("Expected variable name; found {:?}", t.value),
            )),
            None => Err(error(3003, line, col, "Missing variable name")),
        }
    }
    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let (line, col) = self.location();
        let Some(tok) = self.take() else {
            return Err(error(3005, line, col, "Missing expression operand"));
        };
        let mut expr = match tok.kind {
            TokenKind::Number => Expr::Number(tok.value),
            TokenKind::String => Expr::String(tok.value),
            TokenKind::Name => {
                if self.language.legacy_ai_keyword(&tok.value) {
                    return Err(error(
                        3010,
                        line,
                        col,
                        format!(
                            "Legacy AI keyword {:?}; new syntax removes ai_ prefixes",
                            tok.value
                        ),
                    ));
                }
                if let Some(b) = self.language.boolean(&tok.value) {
                    Expr::Boolean(b)
                } else if self.language.kw("none", &tok.value) {
                    Expr::Null
                } else if self.language.kw("not", &tok.value) {
                    Expr::Unary {
                        op: "not".into(),
                        value: Box::new(self.expression(3)?),
                    }
                } else if self.language.is_unsupported_keyword(&tok.value) {
                    return Err(error(
                        3006,
                        line,
                        col,
                        format!(
                            "Reserved grammar keyword {:?} cannot start an expression here",
                            tok.value
                        ),
                    ));
                } else {
                    Expr::Name(tok.value)
                }
            }
            TokenKind::Symbol if tok.value == "(" => {
                let inner = self.expression(0)?;
                self.need(")")?;
                inner
            }
            TokenKind::Symbol if tok.value == "-" || tok.value == "+" => {
                let inner = self.expression(9)?;
                Expr::Unary {
                    op: tok.value,
                    value: Box::new(inner),
                }
            }
            _ => {
                return Err(error(
                    3005,
                    line,
                    col,
                    format!("Invalid expression start {:?}", tok.value),
                ))
            }
        };
        while self.take_symbol("(") {
            let mut args = Vec::new();
            if !self.at_symbol(")") {
                loop {
                    args.push(self.expression(0)?);
                    if !self.take_symbol(",") {
                        break;
                    }
                }
            }
            self.need(")")?;
            expr = Expr::Call {
                callee: Box::new(expr),
                args,
            };
        }
        Ok(expr)
    }
    fn operator(&self) -> Option<(&'static str, usize, u8)> {
        let first = self.word()?;
        let second = self.tokens.get(self.at + 1).map(|t| t.value.as_str());
        let pair = match (first, second) {
            ("=", Some("=")) => Some("=="),
            ("!", Some("=")) => Some("!="),
            ("<", Some("=")) => Some("<="),
            (">", Some("=")) => Some(">="),
            ("/", Some("/")) => Some("//"),
            ("*", Some("*")) => Some("**"),
            _ => None,
        };
        if let Some(op) = pair {
            let prev = self.tokens.get(self.at)?;
            let next = self.tokens.get(self.at + 1)?;
            if prev.end == next.start {
                return Some((
                    op,
                    2,
                    match op {
                        "==" | "!=" | "<=" | ">=" => 3,
                        "//" => 6,
                        "**" => 8,
                        _ => 3,
                    },
                ));
            }
        }
        if self.language.kw("or", first) {
            return Some(("or", 1, 1));
        }
        if self.language.kw("and", first) {
            return Some(("and", 1, 2));
        }
        match first {
            "<" | ">" => Some((if first == "<" { "<" } else { ">" }, 1, 3)),
            "+" => Some(("+", 1, 4)),
            "-" => Some(("-", 1, 4)),
            "*" => Some(("*", 1, 6)),
            "/" => Some(("/", 1, 6)),
            "%" => Some(("%", 1, 6)),
            _ => None,
        }
    }
    fn expression(&mut self, min_prec: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.primary()?;
        while let Some((op, n, prec)) = self.operator() {
            if prec < min_prec {
                break;
            }
            self.at += n;
            let right = self.expression(if op == "**" { prec } else { prec + 1 })?;
            left = Expr::Binary {
                op: op.to_owned(),
                lhs: Box::new(left),
                rhs: Box::new(right),
            };
        }
        Ok(left)
    }
    fn keyword(&self, semantic: &str) -> bool {
        self.word()
            .is_some_and(|word| self.language.kw(semantic, word))
    }
    fn block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        self.need("{")?;
        let mut body = Vec::new();
        loop {
            if self.take_symbol("}") {
                return Ok(body);
            }
            if self.peek().is_none() {
                let (line, col) = self.location();
                return Err(error(3002, line, col, "Unclosed block: missing '}'"));
            }
            body.push(self.statement()?);
        }
    }
    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        if self.keyword("if") {
            self.at += 1;
            let condition = self.expression(0)?;
            let body = self.block()?;
            let mut branches = vec![(condition, body)];
            while self.keyword("elif")
                || (self.keyword("else")
                    && self
                        .tokens
                        .get(self.at + 1)
                        .is_some_and(|next| self.language.kw("if", &next.value)))
            {
                if self.keyword("elif") {
                    self.at += 1;
                } else {
                    self.at += 2;
                }
                let condition = self.expression(0)?;
                branches.push((condition, self.block()?));
            }
            let otherwise = if self.keyword("else") {
                self.at += 1;
                Some(self.block()?)
            } else {
                None
            };
            return Ok(Statement::If {
                branches,
                otherwise,
            });
        }
        if self.keyword("while") {
            self.at += 1;
            let condition = self.expression(0)?;
            let body = self.block()?;
            return Ok(Statement::While { condition, body });
        }
        if self.keyword("for") {
            self.at += 1;
            let variable = self.identifier()?;
            if !self.keyword("in") {
                return Err(self.expected("language-specific 'in' keyword"));
            }
            self.at += 1;
            let iterable = self.expression(0)?;
            let body = self.block()?;
            return Ok(Statement::For {
                variable,
                iterable,
                body,
            });
        }
        if self.keyword("def") {
            self.at += 1;
            let name = self.identifier()?;
            self.need("(")?;
            let mut parameters = Vec::new();
            if !self.at_symbol(")") {
                loop {
                    let name = self.identifier()?;
                    if parameters.contains(&name) {
                        let (line, col) = self.location();
                        return Err(error(
                            3003,
                            line,
                            col,
                            format!("Duplicate function parameter {name:?}"),
                        ));
                    }
                    parameters.push(name);
                    if !self.take_symbol(",") {
                        break;
                    }
                }
            }
            self.need(")")?;
            let body = self.block()?;
            return Ok(Statement::Function {
                name,
                parameters,
                body,
            });
        }
        if self.keyword("return") {
            self.at += 1;
            let value = self.expression(0)?;
            self.take_symbol(";");
            return Ok(Statement::Return(value));
        }
        if self.keyword("break") {
            self.at += 1;
            self.take_symbol(";");
            return Ok(Statement::Break);
        }
        if self.keyword("continue") {
            self.at += 1;
            self.take_symbol(";");
            return Ok(Statement::Continue);
        }

        if self.word() == Some(self.language.let_word()) {
            self.at += 1;
            let name = self.identifier()?;
            let type_name = if self.take_symbol(":") {
                let (line, col) = self.location();
                let ty = self.take().ok_or_else(|| self.expected("type name"))?;
                Some(
                    self.language
                        .ty(&ty.value)
                        .ok_or_else(|| {
                            error(
                                3003,
                                line,
                                col,
                                format!("Unknown type {:?} in selected profile", ty.value),
                            )
                        })?
                        .into(),
                )
            } else {
                None
            };
            self.need("=")?;
            let expr = self.expression(0)?;
            self.take_symbol(";");
            return Ok(Statement::Let {
                name,
                type_name,
                expr,
            });
        }
        if let (Some(a), Some(b)) = (self.tokens.get(self.at), self.tokens.get(self.at + 1)) {
            if a.kind == TokenKind::Name && b.value == "=" {
                let name = self.identifier()?;
                self.need("=")?;
                let expr = self.expression(0)?;
                self.take_symbol(";");
                return Ok(Statement::Assign { name, expr });
            }
        }
        let expr = self.expression(0)?;
        self.take_symbol(";");
        Ok(Statement::Expression(expr))
    }
}
/// Parse a deliberately limited subset. It never executes code.
pub fn parse(source: &str, language_code: &str) -> Result<Program, Diagnostic> {
    let language = Language::from_code(language_code)?;
    let tokens = scan(source)?
        .into_iter()
        .filter(|t| t.kind != TokenKind::Comment)
        .collect();
    let mut parser = Parser {
        tokens,
        at: 0,
        language,
    };
    let mut statements = Vec::new();
    while parser.peek().is_some() {
        let start = parser.at;
        statements.push(parser.statement()?);
        if parser.at == start {
            let (line, col) = parser.location();
            return Err(error(3005, line, col, "Parser did not consume a token"));
        }
    }
    Ok(Program {
        language,
        statements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sk_declaration_assignment_and_call() {
        let program = parse("nech vek: cele = 30\nvek = vek + 1\nvypis(vek)", "sk").unwrap();
        assert_eq!(program.statements.len(), 3);
        assert!(matches!(&program.statements[0],Statement::Let {type_name:Some(s),..} if s=="int"));
        assert!(matches!(
            &program.statements[2],
            Statement::Expression(Expr::Call { .. })
        ));
    }
    #[test]
    fn english_declaration_and_precedence() {
        let program = parse("let n = 2 + 3 * 4", "en").unwrap();
        assert!(
            matches!(&program.statements[0],Statement::Let{expr:Expr::Binary{op,..},..} if op=="+")
        );
    }
    #[test]
    fn ai_profile_with_native_literal() {
        let program = parse("let age: int = 3\nprint(\"世界\", age)", "ai").unwrap();
        assert_eq!(program.statements.len(), 2);
    }
    #[test]
    fn exact_unknown_language_error() {
        let err = parse("nech vek = 1", "zz").unwrap_err();
        assert_eq!(err.code, 3007);
        assert!(err.to_string().contains("Unknown Nobody syntax profile"));
    }
    #[test]
    fn control_flow_is_parsed() {
        let prog = parse("ak pravda { vypis(1) } inak { vypis(2) }", "sk").unwrap();
        assert!(matches!(&prog.statements[0], Statement::If { .. }));
    }
    #[test]
    fn malformed_let_has_actionable_number() {
        let err = parse("nech vek 30", "sk").unwrap_err();
        assert_eq!(err.code, 3001);
        assert!(err.repair.contains("Insert"));
    }
    #[test]
    fn string_comment_preserved_as_token() {
        let program = parse("nech text = \"ak if ai_if\" # comment\nvypis(text)", "sk").unwrap();
        assert_eq!(program.statements.len(), 2);
    }
    #[test]
    fn sk_nested_function_and_return() {
        let program = parse(
            "funkcia dvojnasobok(x) { vrat x * 2 }\nvypis(dvojnasobok(3))",
            "sk",
        )
        .unwrap();
        assert!(
            matches!(&program.statements[0], Statement::Function{parameters,..} if parameters.len()==1)
        );
    }
    #[test]
    fn sk_if_elif_else() {
        let program = parse(
            "ak pravda { vypis(1) } inak_ak nepravda { vypis(2) } inak { vypis(3) }",
            "sk",
        )
        .unwrap();
        assert!(
            matches!(&program.statements[0],Statement::If{branches,otherwise} if branches.len()==2 && otherwise.is_some())
        );
    }
    #[test]
    fn ai_while_with_assignment() {
        let program = parse("let n = 0\nwhile n < 3 { n = n + 1 }", "ai").unwrap();
        assert!(matches!(&program.statements[1], Statement::While { .. }));
    }
    #[test]
    fn english_for_range() {
        let program = parse("for n in range(3) { print(n) }", "en").unwrap();
        assert!(matches!(&program.statements[0],Statement::For{variable,..} if variable=="n"));
    }
    #[test]
    fn ai_function_return() {
        let program = parse("fn make(x) { return x }", "ai").unwrap();
        assert!(matches!(&program.statements[0],Statement::Function{name,..} if name=="make"));
    }
    #[test]
    fn nested_blocks_with_loops() {
        let program = parse(
            "funkcia f(x) { pokial x > 0 { ak pravda { x = x - 1 } } vrat x }",
            "sk",
        )
        .unwrap();
        assert!(matches!(&program.statements[0],Statement::Function{body,..} if body.len()==2));
    }
    #[test]
    fn missing_close_brace_numbered() {
        let err = parse("ak pravda { vypis(1)", "sk").unwrap_err();
        assert_eq!(err.code, 3002);
        assert!(err.repair.contains("}"));
    }
    #[test]
    fn duplicate_function_parameter_numbered() {
        let err = parse("funkcia f(x,x) { vrat x }", "sk").unwrap_err();
        assert_eq!(err.code, 3003);
    }

    #[test]
    fn dangling_else_is_precise_n3006() {
        let err = parse("inak { vypis(1) }", "sk").unwrap_err();
        assert_eq!(err.code, 3006);
        assert!(err.detail.contains("inak"));
    }
    #[test]
    fn for_body_break_and_continue() {
        let program = parse("pre n v rozsah(3) { ak n > 1 { prerus } pokracuj }", "sk").unwrap();
        assert!(matches!(&program.statements[0],Statement::For{body,..} if body.len()==2));
    }
    #[test]
    fn function_arity_is_not_yet_typechecked() {
        let program = parse("funkcia f(x) { vrat x }\nvypis(f(1,2))", "sk").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn ai_familiar_else_if_not_and_or_null() {
        let program=parse("let active = not false and true or false\nif active { print(null) } else if false { print(2) } else { print(3) }","ai").unwrap();
        assert_eq!(program.statements.len(), 2);
        assert!(
            matches!(&program.statements[1],Statement::If{branches,otherwise} if branches.len()==2 && otherwise.is_some())
        );
    }
    #[test]
    fn ai_legacy_prefix_has_numbered_migration_error() {
        let err = parse("ai_let x = 1", "ai").unwrap_err();
        assert_eq!(err.code, 3010);
        assert!(err.repair.contains("migrate-ai"));
    }
}
