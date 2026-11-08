//! Deliberately limited, opt-in Nobody IR evaluator.
//! This is not a bytecode VM, native compiler or security sandbox.
use crate::ir::{IrExpr, IrStatement, Module, Name};
use crate::{lower_source, Diagnostic};
use std::collections::BTreeMap;

const MAX_STEPS: usize = 10_000;
const MAX_DEPTH: usize = 64;
const MAX_OUTPUT: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeValue {
    Int(i128),
    Bool(bool),
    Text(String),
    Null,
}
impl RuntimeValue {
    fn display(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Text(s) => s.clone(),
            Self::Null => "null".into(),
        }
    }
}
fn error(code: u16, detail: impl Into<String>, repair: &'static str) -> Diagnostic {
    Diagnostic {
        code,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair,
    }
}
fn unsupported(detail: impl Into<String>) -> Diagnostic {
    error(
        4004,
        detail,
        "Use only integer, boolean, null, text, declarations, assignments, arithmetic and print in the initial Rust evaluator.",
    )
}
fn literal_text(raw: &str) -> Result<String, Diagnostic> {
    if raw.len() < 2 {
        return Err(unsupported("Invalid text literal"));
    }
    let first = raw.chars().next().unwrap_or('"');
    if !matches!(first, '"' | '\'') || !raw.ends_with(first) {
        return Err(unsupported(
            "Multiline and nonstandard string delimiters are not supported yet",
        ));
    }
    if raw.starts_with("\"\"\"") || raw.starts_with("'''") {
        return Err(unsupported("Triple-quoted strings are not yet supported"));
    }
    let inner = &raw[1..raw.len() - 1];
    let mut result = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let escape = chars
                .next()
                .ok_or_else(|| unsupported("Dangling string escape"))?;
            result.push(match escape {
                'n' => '\n',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                _ => {
                    return Err(unsupported(format!(
                        "Unsupported string escape: {escape:?}"
                    )))
                }
            });
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}
// Floor division/remainder agree with the sign convention used in the
// Python reference, rather than Rust's truncation-toward-zero semantics.
fn floor_div_rem(a: i128, b: i128) -> Option<(i128, i128)> {
    let quotient = a.checked_div(b)?;
    let remainder = a.checked_rem(b)?;
    if remainder != 0 && ((remainder < 0) != (b < 0)) {
        Some((quotient.checked_sub(1)?, remainder.checked_add(b)?))
    } else {
        Some((quotient, remainder))
    }
}
struct Evaluator {
    vars: BTreeMap<String, RuntimeValue>,
    steps: usize,
    output: String,
}
impl Evaluator {
    fn tick(&mut self, depth: usize) -> Result<(), Diagnostic> {
        self.steps += 1;
        if self.steps > MAX_STEPS || depth > MAX_DEPTH {
            return Err(error(
                4005,
                "Evaluator step or expression-depth budget exhausted",
                "Simplify the expression or program. Resource counters are not a hard security sandbox.",
            ));
        }
        Ok(())
    }
    fn eval(&mut self, expr: &IrExpr, depth: usize) -> Result<RuntimeValue, Diagnostic> {
        self.tick(depth)?;
        match expr {
            IrExpr::Number(text) => {
                if !text.bytes().all(|b| b.is_ascii_digit() || b == b'_') {
                    return Err(unsupported(
                        "The first evaluator slice supports integer literals only",
                    ));
                }
                text.replace('_', "")
                    .parse::<i128>()
                    .map(RuntimeValue::Int)
                    .map_err(|_| {
                        error(
                            4003,
                            "Integer literal is outside i128 range",
                            "Use an integer in the signed 128-bit range.",
                        )
                    })
            }
            IrExpr::String(text) => Ok(RuntimeValue::Text(literal_text(text)?)),
            IrExpr::Boolean(value) => Ok(RuntimeValue::Bool(*value)),
            IrExpr::Null => Ok(RuntimeValue::Null),
            IrExpr::Name(Name::User(name)) => self.vars.get(name).cloned().ok_or_else(|| {
                error(
                    4001,
                    format!("Unknown variable {name:?}"),
                    "Declare the variable before using it.",
                )
            }),
            IrExpr::Name(Name::Builtin(_)) => Err(unsupported(
                "A built-in is not a first-class value in the initial evaluator",
            )),
            IrExpr::Unary { op, value } => {
                let value = self.eval(value, depth + 1)?;
                match (op.as_str(), value) {
                    ("+", RuntimeValue::Int(n)) => Ok(RuntimeValue::Int(n)),
                    ("-", RuntimeValue::Int(n)) => {
                        n.checked_neg().map(RuntimeValue::Int).ok_or_else(|| {
                            error(
                                4003,
                                "Integer unary negation overflow",
                                "Use a smaller integer.",
                            )
                        })
                    }
                    ("not", RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(!b)),
                    _ => Err(unsupported(format!(
                        "Unsupported unary operator {op:?} or operand"
                    ))),
                }
            }
            IrExpr::Binary { op, lhs, rhs } => {
                let left = self.eval(lhs, depth + 1)?;
                let right = self.eval(rhs, depth + 1)?;
                match (left, right) {
                    (RuntimeValue::Int(a), RuntimeValue::Int(b)) => {
                        let number =
                            match op.as_str() {
                                "+" => a.checked_add(b),
                                "-" => a.checked_sub(b),
                                "*" => a.checked_mul(b),
                                "/" | "//" | "%" if b == 0 => {
                                    return Err(error(
                                        4002,
                                        "Division by zero at runtime",
                                        "Ensure the divisor is nonzero before dividing.",
                                    ))
                                }
                                "/" => return Err(unsupported(
                                    "Nonzero true division requires floating-point runtime support",
                                )),
                                "//" => floor_div_rem(a, b).map(|(q, _)| q),
                                "%" => floor_div_rem(a, b).map(|(_, r)| r),
                                _ => {
                                    return Err(unsupported(format!(
                                        "Unsupported numeric operator {op:?}"
                                    )))
                                }
                            };
                        number.map(RuntimeValue::Int).ok_or_else(|| {
                            error(
                                4003,
                                "Integer arithmetic overflow",
                                "Use smaller integer operands.",
                            )
                        })
                    }
                    (RuntimeValue::Text(a), RuntimeValue::Text(b)) if op == "+" => {
                        if a.len().saturating_add(b.len()) > MAX_OUTPUT {
                            return Err(error(
                                4005,
                                "Text value exceeds the evaluator budget",
                                "Use shorter text values.",
                            ));
                        }
                        Ok(RuntimeValue::Text(a + &b))
                    }
                    _ => Err(unsupported(format!("Unsupported operands for {op:?}"))),
                }
            }
            IrExpr::Call { callee, args }
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("print"))) =>
            {
                let mut values = Vec::new();
                for arg in args {
                    values.push(self.eval(arg, depth + 1)?.display());
                }
                let line = values.join(" ") + "\n";
                if self.output.len().saturating_add(line.len()) > MAX_OUTPUT {
                    return Err(error(
                        4005,
                        "Output exceeds the evaluator budget",
                        "Produce less output.",
                    ));
                }
                self.output.push_str(&line);
                Ok(RuntimeValue::Null)
            }
            IrExpr::Call { .. } => Err(unsupported(
                "Only the unshadowed print built-in can be called",
            )),
        }
    }
    fn statement(&mut self, stmt: &IrStatement) -> Result<(), Diagnostic> {
        self.tick(0)?;
        match stmt {
            IrStatement::Let {
                name,
                type_name: None,
                expr,
            } => {
                if self.vars.contains_key(name) {
                    return Err(unsupported(format!("Duplicate declaration {name:?}")));
                }
                let value = self.eval(expr, 1)?;
                self.vars.insert(name.clone(), value);
                Ok(())
            }
            IrStatement::Assign { name, expr } => {
                if !self.vars.contains_key(name) {
                    return Err(error(
                        4001,
                        format!("Assignment to unknown variable {name:?}"),
                        "Declare the variable first.",
                    ));
                }
                let value = self.eval(expr, 1)?;
                self.vars.insert(name.clone(), value);
                Ok(())
            }
            IrStatement::Expression(expr) => {
                self.eval(expr, 1)?;
                Ok(())
            }
            _ => Err(unsupported(
                "This statement is not supported by the first Rust evaluator slice",
            )),
        }
    }
}
/// Explicit execution entrypoint. Parsing and type-checking never invoke this.
pub fn evaluate_ir(module: &Module) -> Result<String, Diagnostic> {
    let mut evaluator = Evaluator {
        vars: BTreeMap::new(),
        steps: 0,
        output: String::new(),
    };
    for stmt in &module.statements {
        evaluator.statement(stmt)?;
    }
    Ok(evaluator.output)
}
pub fn evaluate_source(source: &str, profile: &str) -> Result<String, Diagnostic> {
    evaluate_ir(&lower_source(source, profile)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_for_ai_and_slovak_matches() {
        let ai = "let a = 4\nlet b = 5\nprint(a + b)\na = a * 2\nprint(a)";
        let sk = "nech a = 4\nnech b = 5\nvypis(a + b)\na = a * 2\nvypis(a)";
        assert_eq!(evaluate_source(ai, "ai").unwrap(), "9\n8\n");
        assert_eq!(evaluate_source(sk, "sk").unwrap(), "9\n8\n");
    }
    #[test]
    fn unknown_name_is_numbered() {
        assert_eq!(
            evaluate_source("print(missing)", "ai").unwrap_err().code,
            4001
        );
    }
    #[test]
    fn literal_zero_divisor_is_caught_at_runtime() {
        assert_eq!(
            evaluate_source("print(10 / 0)", "ai").unwrap_err().code,
            4002
        );
    }
    #[test]
    fn unsupported_function_is_not_executed() {
        assert_eq!(
            evaluate_source("fn f() { return 1 }", "ai")
                .unwrap_err()
                .code,
            4004
        );
    }
    #[test]
    fn floor_division_and_modulo_keep_python_sign_rules() {
        assert_eq!(
            evaluate_source("print(-7 // 3, -7 % 3, 7 // -3, 7 % -3)", "ai").unwrap(),
            "-3 2 -3 -2\n"
        );
    }
    #[test]
    fn true_division_is_explicitly_deferred() {
        assert_eq!(
            evaluate_source("print(3 / 2)", "ai").unwrap_err().code,
            4004
        );
    }
    #[test]
    fn integer_overflow_is_numbered() {
        assert_eq!(
            evaluate_source("print(170141183460469231731687303715884105727 + 1)", "ai")
                .unwrap_err()
                .code,
            4003
        );
    }
    #[test]
    fn text_and_boolean_literals() {
        assert_eq!(
            evaluate_source("print(\"hello\", true, null)", "ai").unwrap(),
            "hello true null\n"
        );
    }
}
