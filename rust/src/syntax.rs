//! Minimal Rust AST parser for the Nobody migration.
//! Supported: declarations, annotated declarations, assignments, calls,
//! number/string/bool/name expressions and basic binary operators.
//! Unsupported constructs are rejected with precise numbered errors.
//! This is not a compiler or executable interpreter.
use crate::{scan, Diagnostic, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language { Sk, En, Ai }

impl Language {
    pub fn from_code(s: &str) -> Result<Self, Diagnostic> {
        match s {
            "sk" => Ok(Self::Sk),
            "en" => Ok(Self::En),
            "ai" => Ok(Self::Ai),
            _ => Err(error(3007, 1, 1,
                format!("Unsupported Rust syntax profile '{s}'; supported: sk, en, ai"))),
        }
    }
    fn let_word(self) -> &'static str {
        match self { Self::Sk => "nech", Self::En => "let", Self::Ai => "ai_let" }
    }
    fn boolean(self, s: &str) -> Option<bool> {
        match self {
            Self::Sk => match s { "pravda" => Some(true), "nepravda" => Some(false), _ => None },
            Self::En => match s { "true" => Some(true), "false" => Some(false), _ => None },
            Self::Ai => match s { "ai_true" => Some(true), "ai_false" => Some(false), _ => None },
        }
    }
    fn ty(self, s: &str) -> Option<&'static str> {
        match self {
            Self::Sk => match s {
                "cele" => Some("int"), "desatinne" => Some("float"),
                "text" => Some("str"), "logicke" => Some("bool"), _ => None
            },
            Self::En => match s {
                "int" => Some("int"), "float" => Some("float"),
                "str" => Some("str"), "bool" => Some("bool"), _ => None
            },
            Self::Ai => match s {
                "ai_int" => Some("int"), "ai_float" => Some("float"),
                "ai_str" => Some("str"), "ai_bool" => Some("bool"), _ => None
            },
        }
    }
    fn is_unsupported_keyword(self, s: &str) -> bool {
        let common = ["ak", "inak", "pokial", "pre", "funkcia", "vrat",
                      "if", "else", "elif", "while", "for", "function", "def", "return",
                      "ai_if", "ai_else", "ai_elif", "ai_while", "ai_for",
                      "ai_def", "ai_return", "ai_break", "ai_continue"];
        let _ = self;
        common.contains(&s)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Number(String),
    String(String),
    Boolean(bool),
    Name(String),
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Unary { op: String, value: Box<Expr> },
    Binary { op: String, lhs: Box<Expr>, rhs: Box<Expr> },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let { name: String, type_name: Option<String>, expr: Expr },
    Assign { name: String, expr: Expr },
    Expression(Expr),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program { pub language: Language, pub statements: Vec<Statement> }

fn error(code: u16, line: usize, column: usize, detail: impl Into<String>) -> Diagnostic {
    let repair = match code {
        3001 => "Insert the required punctuation or value at the reported location.",
        3003 => "Use a valid lowercase variable name after the declaration keyword.",
        3005 => "Complete the expression with a valid literal, identifier or parenthesized expression.",
        3006 => "Use only declarations, assignments and expressions in the current Rust parser subset.",
        3007 => "Choose --lang sk, en or ai; other profiles require the future full Rust language-table port.",
        _ => "Correct the reported Nobody syntax.",
    };
    Diagnostic { code, line, column, detail: detail.into(), repair }
}

struct Parser {
    tokens: Vec<Token>, at: usize, language: Language,
}
impl Parser {
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.at) }
    fn word(&self) -> Option<&str> { self.peek().map(|t| t.value.as_str()) }
    fn take(&mut self) -> Option<Token> {
        let item = self.tokens.get(self.at)?.clone();
        self.at += 1;
        Some(item)
    }
    fn at_symbol(&self, s: &str) -> bool { self.word() == Some(s) }
    fn take_symbol(&mut self, s: &str) -> bool {
        if self.at_symbol(s) { self.at += 1; true } else { false }
    }
    fn location(&self) -> (usize, usize) {
        self.peek().map(|t|(t.line,t.column)).or_else(||
            self.tokens.last().map(|t|(t.line,t.column+t.value.chars().count())))
            .unwrap_or((1,1))
    }
    fn expected(&self, what: &str) -> Diagnostic {
        let (line,col) = self.location();
        error(3001,line,col,format!("Expected {what}, found {:?}",self.word().unwrap_or("end of file")))
    }
    fn need(&mut self,s:&str)->Result<(),Diagnostic> {
        if self.take_symbol(s) { Ok(()) } else { Err(self.expected(s)) }
    }
    fn identifier(&mut self)->Result<String,Diagnostic> {
        let (line,col)=self.location();
        match self.take() {
            Some(t) if t.kind == TokenKind::Name &&
                self.language.boolean(&t.value).is_none() &&
                !self.language.is_unsupported_keyword(&t.value) &&
                t.value != self.language.let_word() => Ok(t.value),
            Some(t) => Err(error(3003,line,col,format!("Expected variable name; found {:?}",t.value))),
            None => Err(error(3003,line,col,"Missing variable name")),
        }
    }
    fn primary(&mut self)->Result<Expr,Diagnostic> {
        let (line,col)=self.location();
        let Some(tok)=self.take() else {
            return Err(error(3005,line,col,"Missing expression operand"));
        };
        let mut expr=match tok.kind {
            TokenKind::Number => Expr::Number(tok.value),
            TokenKind::String => Expr::String(tok.value),
            TokenKind::Name => {
                if self.language.is_unsupported_keyword(&tok.value) {
                    return Err(error(3006,line,col,
                        format!("Keyword {:?} is not implemented by the Rust subset parser",tok.value)));
                }
                if let Some(b)=self.language.boolean(&tok.value) {
                    Expr::Boolean(b)
                } else if tok.value == self.language.let_word() {
                    return Err(error(3005,line,col,"Declaration cannot appear in an expression"));
                } else {
                    Expr::Name(tok.value)
                }
            }
            TokenKind::Symbol if tok.value=="(" => {
                let inner=self.expression(0)?;
                self.need(")")?;
                inner
            }
            TokenKind::Symbol if tok.value=="-" || tok.value=="+" => {
                let inner=self.expression(9)?;
                Expr::Unary {op:tok.value, value:Box::new(inner)}
            }
            _ => return Err(error(3005,line,col,
                format!("Invalid expression start {:?}",tok.value))),
        };
        while self.take_symbol("(") {
            let mut args=Vec::new();
            if !self.at_symbol(")") {
                loop {
                    args.push(self.expression(0)?);
                    if !self.take_symbol(",") { break; }
                }
            }
            self.need(")")?;
            expr=Expr::Call {callee:Box::new(expr),args};
        }
        Ok(expr)
    }
    fn operator(&self)->Option<(&'static str,usize,u8)> {
        let first=self.word()?;
        let second=self.tokens.get(self.at+1).map(|t|t.value.as_str());
        let pair = match (first,second) {
            ("=",Some("=")) => Some("=="),
            ("!",Some("=")) => Some("!="),
            ("<",Some("=")) => Some("<="),
            (">",Some("=")) => Some(">="),
            ("/",Some("/")) => Some("//"),
            ("*",Some("*")) => Some("**"),
            _ => None
        };
        if let Some(op)=pair {
            let prev=self.tokens.get(self.at)?;
            let next=self.tokens.get(self.at+1)?;
            if prev.end==next.start {
                return Some((op,2,match op {
                    "=="|"!="|"<="|">="=>3,"//"=>6,"**"=>8,_=>3
                }));
            }
        }
        match first {
            "<"|">" => Some((if first=="<" {"<"} else {">"},1,3)),
            "+" => Some(("+",1,4)), "-" => Some(("-",1,4)),
            "*" => Some(("*",1,6)), "/" => Some(("/",1,6)),
            "%" => Some(("%",1,6)),
            _ => None
        }
    }
    fn expression(&mut self,min_prec:u8)->Result<Expr,Diagnostic> {
        let mut left=self.primary()?;
        while let Some((op,n,prec))=self.operator() {
            if prec < min_prec {break;}
            self.at+=n;
            let right=self.expression(if op=="**" {prec} else {prec+1})?;
            left=Expr::Binary {op:op.to_owned(),lhs:Box::new(left),rhs:Box::new(right)};
        }
        Ok(left)
    }
    fn statement(&mut self)->Result<Statement,Diagnostic> {
        if self.word()==Some(self.language.let_word()) {
            self.at+=1;
            let name=self.identifier()?;
            let type_name=if self.take_symbol(":") {
                let (line,col)=self.location();
                let ty=self.take().ok_or_else(||self.expected("type name"))?;
                Some(self.language.ty(&ty.value).ok_or_else(||
                    error(3003,line,col,format!("Unknown type {:?} in selected profile",ty.value)))?.into())
            }else{None};
            self.need("=")?;
            let expr=self.expression(0)?;
            self.take_symbol(";");
            return Ok(Statement::Let {name,type_name,expr});
        }
        if let (Some(a),Some(b))=(self.tokens.get(self.at),self.tokens.get(self.at+1)) {
            if a.kind==TokenKind::Name && b.value=="=" {
                let name=self.identifier()?;
                self.need("=")?;
                let expr=self.expression(0)?;
                self.take_symbol(";");
                return Ok(Statement::Assign{name,expr});
            }
        }
        let expr=self.expression(0)?;
        self.take_symbol(";");
        Ok(Statement::Expression(expr))
    }
}
/// Parse a deliberately limited subset. It never executes code.
pub fn parse(source:&str,language_code:&str)->Result<Program,Diagnostic>{
    let language=Language::from_code(language_code)?;
    let tokens=scan(source)?.into_iter().filter(|t|t.kind!=TokenKind::Comment).collect();
    let mut parser=Parser {tokens,at:0,language};
    let mut statements=Vec::new();
    while parser.peek().is_some() {
        let start=parser.at;
        statements.push(parser.statement()?);
        if parser.at==start {
            let (line,col)=parser.location();
            return Err(error(3005,line,col,"Parser did not consume a token"));
        }
    }
    Ok(Program { language, statements })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sk_declaration_assignment_and_call() {
        let program=parse("nech vek: cele = 30\nvek = vek + 1\nvypis(vek)","sk").unwrap();
        assert_eq!(program.statements.len(),3);
        assert!(matches!(&program.statements[0],Statement::Let {type_name:Some(s),..} if s=="int"));
        assert!(matches!(&program.statements[2],Statement::Expression(Expr::Call{..})));
    }
    #[test]
    fn english_declaration_and_precedence() {
        let program=parse("let n = 2 + 3 * 4","en").unwrap();
        assert!(matches!(&program.statements[0],Statement::Let{expr:Expr::Binary{op,..},..} if op=="+"));
    }
    #[test]
    fn ai_profile_with_native_literal() {
        let program=parse("ai_let age: ai_int = 3\nai_print(\"世界\", age)","ai").unwrap();
        assert_eq!(program.statements.len(),2);
    }
    #[test]
    fn exact_unknown_language_error() {
        let err=parse("nech vek = 1","ja").unwrap_err();
        assert_eq!(err.code,3007);
        assert!(err.to_string().contains("sk, en, ai"));
    }
    #[test]
    fn unsupported_control_flow_has_specific_number() {
        let err=parse("ak pravda { vypis(1) }","sk").unwrap_err();
        assert_eq!(err.code,3006);
    }
    #[test]
    fn malformed_let_has_actionable_number() {
        let err=parse("nech vek 30","sk").unwrap_err();
        assert_eq!(err.code,3001);
        assert!(err.repair.contains("Insert"));
    }
    #[test]
    fn string_comment_preserved_as_token() {
        let program=parse("nech text = \"ak if ai_if\" # comment\nvypis(text)","sk").unwrap();
        assert_eq!(program.statements.len(),2);
    }
}
