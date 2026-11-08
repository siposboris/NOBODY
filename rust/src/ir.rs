//! Canonical, language-neutral Nobody IR, version 1 (experimental).
//!
//! Lowering normalizes built-in names only when no lexical declaration shadows
//! them. No source code is executed and no semantic-equivalence proof is made.
//! The IR is an in-memory Rust representation, NOT the JSON AST v1 format.
use std::collections::BTreeSet;
use crate::{parse_validated, Diagnostic};
use crate::profiles;
use crate::syntax::{Expr, Statement};

pub const IR_VERSION: &str = "nobody.ir.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub version: &'static str,
    pub statements: Vec<IrStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    User(String),
    Builtin(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrExpr {
    Number(String),
    String(String),
    Boolean(bool),
    Null,
    Name(Name),
    Call { callee: Box<IrExpr>, args: Vec<IrExpr> },
    Unary { op: String, value: Box<IrExpr> },
    Binary { op: String, lhs: Box<IrExpr>, rhs: Box<IrExpr> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrStatement {
    Let { name: String, type_name: Option<String>, expr: IrExpr },
    Assign { name: String, expr: IrExpr },
    If { branches: Vec<(IrExpr, Vec<IrStatement>)>,
         otherwise: Option<Vec<IrStatement>> },
    While { condition: IrExpr, body: Vec<IrStatement> },
    For { variable: String, iterable: IrExpr, body: Vec<IrStatement> },
    Function { name: String, parameters: Vec<String>, body: Vec<IrStatement> },
    Return(IrExpr),
    Break,
    Continue,
    Expression(IrExpr),
}

fn builtin(code: &str, name: &str) -> Option<&'static str> {
    match profiles::semantic(code, name) {
        Some("print") => Some("print"),
        Some("range") => Some("range"),
        Some("len") => Some("len"),
        Some("sum") => Some("sum"),
        Some("min") => Some("min"),
        Some("max") => Some("max"),
        Some("abs") => Some("abs"),
        Some("round") => Some("round"),
        Some("int") => Some("int"),
        Some("float") => Some("float"),
        Some("str") => Some("str"),
        Some("bool") => Some("bool"),
        _ => None,
    }
}

fn expression(expr: &Expr, language: &str, visible: &BTreeSet<String>) -> IrExpr {
    match expr {
        Expr::Number(n) => IrExpr::Number(n.clone()),
        Expr::String(s) => IrExpr::String(s.clone()),
        Expr::Boolean(b) => IrExpr::Boolean(*b),
        Expr::Null => IrExpr::Null,
        Expr::Name(name) => {
            let normalized = if visible.contains(name) {
                Name::User(name.clone())
            } else if let Some(semantic) = builtin(language, name) {
                Name::Builtin(semantic)
            } else {
                Name::User(name.clone())
            };
            IrExpr::Name(normalized)
        }
        Expr::Call { callee, args } => IrExpr::Call {
            callee: Box::new(expression(callee, language, visible)),
            args: args.iter().map(|x| expression(x, language, visible)).collect(),
        },
        Expr::Unary { op, value } => IrExpr::Unary {
            op: op.clone(),
            value: Box::new(expression(value, language, visible)),
        },
        Expr::Binary { op, lhs, rhs } => IrExpr::Binary {
            op: op.clone(),
            lhs: Box::new(expression(lhs, language, visible)),
            rhs: Box::new(expression(rhs, language, visible)),
        },
    }
}

fn lower_block(
    statements: &[Statement],
    language: &str,
    outer: &BTreeSet<String>,
) -> Vec<IrStatement> {
    // Sequential visibility: a declaration shadows a builtin only after
    // its initializer has been lowered. Functions see their own names so
    // recursive calls remain user symbols. This is still a preliminary scope
    // model; runtime lexical binding rules must be specified separately.
    let mut visible = outer.clone();
    statements.iter().map(|statement| {
        let lowered = match statement {
            Statement::Let { name, type_name, expr } => IrStatement::Let {
                name: name.clone(),
                // Normalize localized primitive annotations into canonical IR.
                // Unknown annotation names remain unchanged for future validation.
                type_name: type_name.as_ref().map(|name|
                    profiles::semantic(language, name)
                        .filter(|semantic| matches!(*semantic, "int"|"float"|"str"|"bool"))
                        .unwrap_or(name.as_str()).to_string()
                ),
                expr: expression(expr, language, &visible),
            },
            Statement::Assign { name, expr } => IrStatement::Assign {
                name: name.clone(),
                expr: expression(expr, language, &visible),
            },
            Statement::If { branches, otherwise } => IrStatement::If {
                branches: branches.iter().map(|(cond, body)| (
                    expression(cond, language, &visible),
                    lower_block(body, language, &visible),
                )).collect(),
                otherwise: otherwise.as_ref().map(|body| lower_block(body, language, &visible)),
            },
            Statement::While { condition, body } => IrStatement::While {
                condition: expression(condition, language, &visible),
                body: lower_block(body, language, &visible),
            },
            Statement::For { variable, iterable, body } => {
                let mut body_scope = visible.clone();
                body_scope.insert(variable.clone());
                IrStatement::For {
                    variable: variable.clone(),
                    iterable: expression(iterable, language, &visible),
                    body: lower_block(body, language, &body_scope),
                }
            }
            Statement::Function { name, parameters, body } => {
                let mut function_scope = visible.clone();
                function_scope.insert(name.clone()); // recursion
                function_scope.extend(parameters.iter().cloned());
                IrStatement::Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: lower_block(body, language, &function_scope),
                }
            }
            Statement::Return(expr) =>
                IrStatement::Return(expression(expr, language, &visible)),
            Statement::Break => IrStatement::Break,
            Statement::Continue => IrStatement::Continue,
            Statement::Expression(expr) =>
                IrStatement::Expression(expression(expr, language, &visible)),
        };
        match statement {
            Statement::Let { name, .. } | Statement::Function { name, .. } => {
                visible.insert(name.clone());
            }
            _ => {}
        }
        lowered
    }).collect()
}

/// Parse, validate and lower Nobody source to experimental IR v1.
/// Source syntax, keywords and language selection are not stored in IR nodes.
pub fn lower_source(source: &str, language: &str) -> Result<Module, Diagnostic> {
    let parsed = parse_validated(source, language)?;
    Ok(Module {
        version: IR_VERSION,
        statements: lower_block(&parsed.statements, language, &BTreeSet::new()),
    })
}

/// Compare canonical IR shape. `true` means identical normalized structure,
/// NOT an independently proved runtime/observational equivalence.
pub fn same_ir(
    left: &str, left_language: &str,
    right: &str, right_language: &str,
) -> Result<bool, Diagnostic> {
    Ok(lower_source(left, left_language)? == lower_source(right, right_language)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate_source;

    #[test]
    fn sk_ai_structural_match() {
        let sk = "funkcia dvoj(x) { vrat x * 2 }\nnech n: cele = 3\nak pravda { vypis(dvoj(n)) }";
        let ai = translate_source(sk, "sk", "ai").unwrap();
        assert!(same_ir(sk, "sk", &ai, "ai").unwrap());
    }

    #[test]
    fn distinct_operations_do_not_match() {
        assert!(!same_ir("nech x = 2 + 1", "sk", "let x = 2 * 1", "ai").unwrap());
    }

    #[test]
    fn builtin_is_semantically_normalized() {
        let sk = lower_source("vypis(1)", "sk").unwrap();
        let ai = lower_source("print(1)", "ai").unwrap();
        assert_eq!(sk, ai);
        let IrStatement::Expression(IrExpr::Call { callee, .. }) = &sk.statements[0]
        else { panic!("expected call"); };
        assert_eq!(**callee, IrExpr::Name(Name::Builtin("print")));
    }

    #[test]
    fn shadowed_builtin_is_a_user_name() {
        let ir = lower_source("fn f(print) { return print }", "ai").unwrap();
        let IrStatement::Function { body, .. } = &ir.statements[0]
        else { panic!("expected function"); };
        assert_eq!(body[0], IrStatement::Return(IrExpr::Name(Name::User("print".into()))));
    }

    #[test]
    fn builtin_shadowing_starts_after_declaration(){
        let ir=lower_source("print(1)\nlet print = 7\nprint(2)","ai").unwrap();
        let IrStatement::Expression(IrExpr::Call { callee:first, .. })=&ir.statements[0]
            else {panic!("expected first call")};
        assert_eq!(**first,IrExpr::Name(Name::Builtin("print")));
        let IrStatement::Expression(IrExpr::Call { callee:last, .. })=&ir.statements[2]
            else {panic!("expected last call")};
        assert_eq!(**last,IrExpr::Name(Name::User("print".into())));
    }
    #[test]
    fn initializer_uses_previous_binding_not_new_variable(){
        let ir=lower_source("let print = print","ai").unwrap();
        let IrStatement::Let{expr,..}=&ir.statements[0] else {panic!("expected let")};
        assert_eq!(*expr,IrExpr::Name(Name::Builtin("print")));
    }
    #[test]
    fn function_name_is_visible_for_recursion(){
        let ir=lower_source("fn print(n) { return print(n) }","ai").unwrap();
        let IrStatement::Function{body,..}=&ir.statements[0] else {panic!("expected function")};
        let IrStatement::Return(IrExpr::Call{callee,..})=&body[0] else {panic!("expected call")};
        assert_eq!(**callee,IrExpr::Name(Name::User("print".into())));
    }
    #[test]
    fn nested_loop_control_is_represented() {
        let ir = lower_source("while true { if true { break } }", "ai").unwrap();
        assert!(matches!(&ir.statements[0], IrStatement::While{body,..} if body.len()==1));
    }

    #[test]
    fn invalid_context_keeps_numbered_error() {
        let err = lower_source("return 1", "ai").unwrap_err();
        assert_eq!(err.code, 3008);
    }

    #[test]
    fn invalid_profile_keeps_numbered_error() {
        let err = lower_source("let n = 1", "zz").unwrap_err();
        assert_eq!(err.code, 3007);
    }

    #[test]
    fn localized_primitive_annotation_is_canonical() {
        let sk=lower_source("nech vek: cele = 3","sk").unwrap();
        let ai=lower_source("let vek: int = 3","ai").unwrap();
        assert_eq!(sk,ai);
        let IrStatement::Let{type_name,..}=&sk.statements[0] else {panic!("expected let")};
        assert_eq!(type_name.as_deref(),Some("int"));
    }

    #[test]
    fn canonical_annotations_match_all_profiles() {
        let source="nech vek: cele = 3";
        let base=lower_source(source,"sk").unwrap();
        for profile in profiles::PROFILES {
            let converted=translate_source(source,"sk",profile.code).unwrap();
            let other=lower_source(&converted,profile.code).unwrap();
            assert_eq!(base,other,"profile {}",profile.code);
        }
    }

    #[test]
    fn all_keyword_profiles_lower_to_common_ir() {
        let original = "nech n: cele = 2\nvypis(n)";
        let base = lower_source(original, "sk").unwrap();
        for p in profiles::PROFILES {
            let converted = translate_source(original, "sk", p.code).unwrap();
            let other = lower_source(&converted, p.code)
                .unwrap_or_else(|e| panic!("profile {}: {}", p.code, e));
            assert_eq!(base, other, "profile {}", p.code);
        }
    }
}
