//! Conservative return-type inference ported from nobody/return_types.py.
//! Unsupported control flow is unknown, never a claim of whole-program safety.
use crate::syntax::{Expr, Statement};
use std::collections::BTreeMap;

const MAX_PATHS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnType {
    Int,
    Float,
    Str,
    Bool,
    None,
    List,
    Unknown,
}

type Environment = BTreeMap<String, ReturnType>;

fn annotation(name: &str) -> ReturnType {
    match name {
        "int" => ReturnType::Int,
        "float" => ReturnType::Float,
        "str" => ReturnType::Str,
        "bool" => ReturnType::Bool,
        _ => ReturnType::Unknown,
    }
}

fn expression(expr: &Expr, env: &Environment) -> ReturnType {
    use ReturnType::*;
    match expr {
        Expr::Number(value) => if value.contains('.') { Float } else { Int },
        Expr::String(_) => Str,
        Expr::Boolean(_) => Bool,
        Expr::Null => None,
        Expr::Name(name) => *env.get(name).unwrap_or(&Unknown),
        Expr::Unary { op, value } => {
            if op == "not" { Bool } else {
                let operand = expression(value, env);
                if matches!(operand, Int | Float) { operand } else { Unknown }
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let a = expression(lhs, env);
            let b = expression(rhs, env);
            match op.as_str() {
                "==" | "!=" | "<" | "<=" | ">" | ">=" =>
                    if a != Unknown && b != Unknown { Bool } else { Unknown },
                "and" | "or" => if a == b { a } else { Unknown },
                "+" if a == Str && b == Str => Str,
                _ if matches!(a, Int | Float) && matches!(b, Int | Float) => {
                    if op == "/" { Float }
                    else if op == "**" { Unknown }
                    else if a == Float || b == Float { Float }
                    else { Int }
                }
                _ => Unknown,
            }
        }
        Expr::Call { .. } => Unknown,
    }
}

fn walk(statements: &[Statement], start: &Environment) -> (Vec<ReturnType>, Vec<Environment>) {
    use ReturnType::*;
    let mut active = vec![start.clone()];
    let mut returns = Vec::new();
    for statement in statements {
        let mut future = Vec::new();
        for env in active {
            match statement {
                Statement::Let { name, type_name, expr } => {
                    if env.contains_key(name) { return (vec![Unknown], vec![]); }
                    let mut next = env.clone();
                    next.insert(name.clone(), type_name.as_deref().map(annotation)
                        .unwrap_or_else(|| expression(expr, &env)));
                    future.push(next);
                }
                Statement::Assign { name, expr } => {
                    if !env.contains_key(name) { return (vec![Unknown], vec![]); }
                    let mut next = env.clone();
                    next.insert(name.clone(), expression(expr, &env));
                    future.push(next);
                }
                Statement::Return(expr) => returns.push(expression(expr, &env)),
                Statement::If { branches, otherwise } => {
                    for (_, body) in branches {
                        let (r, fallthrough) = walk(body, &env);
                        returns.extend(r);
                        future.extend(fallthrough);
                    }
                    if let Some(body) = otherwise {
                        let (r, fallthrough) = walk(body, &env);
                        returns.extend(r);
                        future.extend(fallthrough);
                    } else {
                        future.push(env.clone());
                    }
                }
                Statement::Function { name, .. } => {
                    let mut next = env.clone();
                    next.insert(name.clone(), Unknown);
                    future.push(next);
                }
                _ => return (vec![Unknown], vec![]),
            }
        }
        active = future;
        if active.len() + returns.len() > MAX_PATHS {
            return (vec![Unknown], vec![]);
        }
        if active.is_empty() { break; }
    }
    (returns, active)
}

/// Infer a function's return type only when every possible exit agrees.
/// The original Python API raises for a non-function node; this typed Rust API
/// instead returns Unknown for that input.
pub fn infer_function_return(statement: &Statement) -> ReturnType {
    let Statement::Function { parameters, body, .. } = statement else {
        return ReturnType::Unknown;
    };
    let initial: Environment = parameters.iter()
        .map(|p| (p.clone(), ReturnType::Unknown)).collect();
    let (mut returns, fallthrough) = walk(body, &initial);
    if !fallthrough.is_empty() { returns.push(ReturnType::None); }
    let Some(first) = returns.first().copied() else {
        return ReturnType::Unknown;
    };
    if first != ReturnType::Unknown && returns.iter().all(|t| *t == first) {
        first
    } else {
        ReturnType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn function(body: Vec<Statement>) -> Statement {
        Statement::Function {
            name: "f".into(), parameters: vec![], parameter_types: vec![],
            return_type: None, body,
        }
    }
    fn number(value: &str) -> Expr { Expr::Number(value.into()) }
    fn declaration(name: &str, expr: Expr) -> Statement {
        Statement::Let { name: name.into(), type_name: None, expr }
    }
    #[test]
    fn infers_literal_and_assigned_variable() {
        let f = function(vec![declaration("x", number("2")),
            Statement::Assign { name: "x".into(), expr: number("3") },
            Statement::Return(Expr::Name("x".into()))]);
        assert_eq!(infer_function_return(&f), ReturnType::Int);
    }
    #[test]
    fn recognizes_fallthrough_and_conflicting_branches() {
        let f = function(vec![Statement::If {
            branches: vec![(Expr::Boolean(true), vec![Statement::Return(number("1"))])],
            otherwise: None,
        }]);
        assert_eq!(infer_function_return(&f), ReturnType::Unknown);
        let g = function(vec![Statement::If {
            branches: vec![(Expr::Boolean(true), vec![Statement::Return(number("1"))])],
            otherwise: Some(vec![Statement::Return(number("2"))]),
        }]);
        assert_eq!(infer_function_return(&g), ReturnType::Int);
    }
    #[test]
    fn refuses_unsupported_loops_and_duplicate_declarations() {
        let loop_body = function(vec![Statement::While {
            condition: Expr::Boolean(false), body: vec![],
        }, Statement::Return(number("1"))]);
        assert_eq!(infer_function_return(&loop_body), ReturnType::Unknown);
        let duplicate = function(vec![declaration("x", number("1")),
            declaration("x", number("2")), Statement::Return(number("3"))]);
        assert_eq!(infer_function_return(&duplicate), ReturnType::Unknown);
    }
    #[test]
    fn respects_annotation_and_unknown_parameters() {
        let annotated = function(vec![Statement::Let {
            name: "x".into(), type_name: Some("str".into()), expr: number("1"),
        }, Statement::Return(Expr::Name("x".into()))]);
        assert_eq!(infer_function_return(&annotated), ReturnType::Str);
        let f = Statement::Function {
            name: "f".into(), parameters: vec!["x".into()],
            parameter_types: vec![None], return_type: None,
            body: vec![Statement::Return(Expr::Name("x".into()))],
        };
        assert_eq!(infer_function_return(&f), ReturnType::Unknown);
    }
    #[test]
    fn matches_conservative_binary_rules() {
        let divide = function(vec![Statement::Return(Expr::Binary {
            op: "/".into(), lhs: Box::new(number("2")), rhs: Box::new(number("1")),
        })]);
        assert_eq!(infer_function_return(&divide), ReturnType::Float);
        let exponent = function(vec![Statement::Return(Expr::Binary {
            op: "**".into(), lhs: Box::new(number("2")), rhs: Box::new(number("3")),
        })]);
        assert_eq!(infer_function_return(&exponent), ReturnType::Unknown);
    }
}
