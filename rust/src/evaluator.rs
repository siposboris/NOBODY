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
    List(std::rc::Rc<Vec<RuntimeValue>>),
    Range(i128, i128, i128),
    Int(i128),
    Float(u64),
    Bool(bool),
    Text(String),
    Function(usize),
    Builtin(&'static str),
    Null,
}
impl RuntimeValue {
    fn display(&self) -> String {
        match self {
            Self::List(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(Self::display)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Range(start, stop, step) => format!("range({start}, {stop}, {step})"),
            Self::Int(n) => n.to_string(),
            Self::Float(bits) => {
                let value = f64::from_bits(*bits);
                let mut text = value.to_string();
                if !text.contains('.') && !text.contains('e') {
                    text.push_str(".0");
                }
                text
            }
            Self::Bool(b) => b.to_string(),
            Self::Text(s) => s.clone(),
            Self::Function(_) | Self::Builtin(_) => "<function>".into(),
            Self::Null => "null".into(),
        }
    }
}
fn matches_explicit_type(value: &RuntimeValue, annotation: &str) -> bool {
    matches!(
        (annotation, value),
        ("int", RuntimeValue::Int(_))
            | ("float", RuntimeValue::Float(_))
            | ("bool", RuntimeValue::Bool(_))
            | ("str", RuntimeValue::Text(_))
    )
}

fn float_value(value: f64) -> Result<RuntimeValue, Diagnostic> {
    if !value.is_finite() {
        return Err(error(
            4003,
            "Non-finite floating-point result",
            "Use smaller finite numeric operands.",
        ));
    }
    Ok(RuntimeValue::Float(if value == 0.0 {
        0.0_f64.to_bits()
    } else {
        value.to_bits()
    }))
}

fn range_len(start: i128, stop: i128, step: i128) -> u128 {
    let distance = if step > 0 && start < stop {
        stop.abs_diff(start)
    } else if step < 0 && start > stop {
        start.abs_diff(stop)
    } else {
        return 0;
    };
    (distance - 1) / step.unsigned_abs() + 1
}

// Within a valid range the unsigned displacement fits in u128, even
// when the signed difference between the endpoints exceeds i128::MAX.
fn range_at(start: i128, stop: i128, step: i128, index: i128) -> Option<i128> {
    let len = range_len(start, stop, step);
    let offset = if index < 0 {
        len.checked_sub(index.unsigned_abs())?
    } else {
        index as u128
    };
    if offset >= len {
        return None;
    }
    let displacement = offset.checked_mul(step.unsigned_abs())?;
    Some(if step > 0 {
        (start as u128).wrapping_add(displacement) as i128
    } else {
        (start as u128).wrapping_sub(displacement) as i128
    })
}

fn as_float(value: &RuntimeValue) -> Option<f64> {
    match value {
        RuntimeValue::Int(n) => Some(*n as f64),
        RuntimeValue::Float(bits) => Some(f64::from_bits(*bits)),
        _ => None,
    }
}

// Compare mixed numbers without rounding the integer to f64.
fn numeric_cmp(left: &RuntimeValue, right: &RuntimeValue) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    fn int_float(n: i128, f: f64) -> Ordering {
        if f >= 2.0_f64.powi(127) {
            return Ordering::Less;
        }
        if f < -2.0_f64.powi(127) {
            return Ordering::Greater;
        }
        let truncated = f as i128;
        match n.cmp(&truncated) {
            Ordering::Equal if f.fract() > 0.0 => Ordering::Less,
            Ordering::Equal if f.fract() < 0.0 => Ordering::Greater,
            order => order,
        }
    }
    match (left, right) {
        (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Some(a.cmp(b)),
        (RuntimeValue::Int(a), RuntimeValue::Float(b)) => Some(int_float(*a, f64::from_bits(*b))),
        (RuntimeValue::Float(a), RuntimeValue::Int(b)) => {
            Some(int_float(*b, f64::from_bits(*a)).reverse())
        }
        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
            f64::from_bits(*a).partial_cmp(&f64::from_bits(*b))
        }
        _ => None,
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
        "Use only the documented Rust evaluator subset: basic values, declarations, assignments, conditionals, bounded loops, arithmetic and print.",
    )
}
pub(crate) fn literal_text(raw: &str) -> Result<String, Diagnostic> {
    let first = raw
        .chars()
        .next()
        .ok_or_else(|| unsupported("Empty text literal"))?;
    if !matches!(first, '"' | '\'') {
        return Err(unsupported("Invalid string delimiter"));
    }
    let triple = first.to_string().repeat(3);
    let delimiter = if raw.starts_with(&triple) { 3 } else { 1 };
    if raw.len() < delimiter * 2 || !raw.ends_with(&first.to_string().repeat(delimiter)) {
        return Err(unsupported("Unterminated text literal"));
    }
    let inner = &raw[delimiter..raw.len() - delimiter];
    let mut result = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        let escape = chars
            .next()
            .ok_or_else(|| unsupported("Dangling string escape"))?;
        match escape {
            '\n' => continue,
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                continue;
            }
            'n' => result.push('\n'),
            't' => result.push('\t'),
            'r' => result.push('\r'),
            'a' => result.push('\u{7}'),
            'b' => result.push('\u{8}'),
            'f' => result.push('\u{c}'),
            'v' => result.push('\u{b}'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            '\'' => result.push('\''),
            'x' | 'u' | 'U' => {
                let digits = match escape {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let mut value = 0_u32;
                for _ in 0..digits {
                    let digit = chars
                        .next()
                        .and_then(|ch| ch.to_digit(16))
                        .ok_or_else(|| unsupported("Invalid hexadecimal string escape"))?;
                    value = value * 16 + digit;
                }
                result.push(
                    char::from_u32(value)
                        .ok_or_else(|| unsupported("Escape is not a Unicode scalar value"))?,
                );
            }
            '0'..='7' => {
                let mut value = escape.to_digit(8).expect("octal digit");
                for _ in 0..2 {
                    let Some(digit) = chars.peek().and_then(|ch| ch.to_digit(8)) else {
                        break;
                    };
                    chars.next();
                    value = value * 8 + digit;
                }
                result.push(char::from_u32(value).expect("three octal digits fit in Unicode"));
            }
            _ => {
                return Err(unsupported(format!(
                    "Unsupported string escape: {escape:?}"
                )))
            }
        }
        if result.len() > MAX_OUTPUT {
            return Err(unsupported("Text literal exceeds evaluator budget"));
        }
    }
    if result.len() > MAX_OUTPUT {
        return Err(unsupported("Text literal exceeds evaluator budget"));
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
#[derive(Clone, Debug, PartialEq, Eq)]
enum Flow {
    Next,
    Break,
    Continue,
    Return(RuntimeValue),
}
#[derive(Clone, Debug)]
struct Function {
    parameters: Vec<String>,
    parameter_types: Vec<Option<String>>,
    return_type: Option<String>,
    body: Vec<IrStatement>,
    closure: usize,
}
#[derive(Default)]
struct Scope {
    parent: Option<usize>,
    bindings: BTreeMap<String, (RuntimeValue, Option<String>)>,
}
struct Evaluator {
    // Arena indices avoid reference cycles between closures and their environments.
    // The entire arena is released at the end of one bounded evaluation.
    scopes: Vec<Scope>,
    current: usize,
    functions: Vec<Function>,
    active_calls: Vec<String>,
    steps: usize,
    max_steps: usize,
    output: String,
}
impl Evaluator {
    fn tick(&mut self, depth: usize) -> Result<(), Diagnostic> {
        self.steps += 1;
        if self.steps > self.max_steps || depth > MAX_DEPTH {
            return Err(error(
                4005,
                "Evaluator step or expression-depth budget exhausted",
                "Simplify the expression or program. Resource counters are not a hard security sandbox.",
            ));
        }
        Ok(())
    }
    fn owner(&self, name: &str) -> Option<usize> {
        let mut current = Some(self.current);
        while let Some(index) = current {
            let scope = &self.scopes[index];
            if scope.bindings.contains_key(name) {
                return Some(index);
            }
            current = scope.parent;
        }
        None
    }
    fn lookup(&self, name: &str) -> Option<&RuntimeValue> {
        self.owner(name)
            .and_then(|index| self.scopes[index].bindings.get(name))
            .map(|(value, _)| value)
    }
    fn assign(&mut self, name: &str, value: RuntimeValue) -> Result<(), Diagnostic> {
        let index = self.owner(name).ok_or_else(|| {
            error(
                4001,
                format!("Unknown variable {name:?}"),
                "Declare the variable first.",
            )
        })?;
        let (old, annotation) = self.scopes[index]
            .bindings
            .get_mut(name)
            .expect("owner contains binding");
        if annotation
            .as_deref()
            .is_some_and(|kind| !matches_explicit_type(&value, kind))
        {
            return Err(unsupported(format!(
                "Assignment to {name:?} violates its explicit type annotation"
            )));
        }
        *old = value;
        Ok(())
    }
    fn new_scope(&mut self, parent: usize) -> usize {
        let index = self.scopes.len();
        self.scopes.push(Scope {
            parent: Some(parent),
            bindings: BTreeMap::new(),
        });
        index
    }
    fn block(&mut self, statements: &[IrStatement]) -> Result<Flow, Diagnostic> {
        let saved = self.current;
        self.current = self.new_scope(saved);
        let result = self.statements(statements);
        self.current = saved;
        result
    }
    fn statements(&mut self, statements: &[IrStatement]) -> Result<Flow, Diagnostic> {
        for stmt in statements {
            let control = self.statement(stmt)?;
            if control != Flow::Next {
                return Ok(control);
            }
        }
        Ok(Flow::Next)
    }
    fn values_equal(
        &mut self,
        left: &RuntimeValue,
        right: &RuntimeValue,
        depth: usize,
    ) -> Result<bool, Diagnostic> {
        self.tick(depth)?;
        if let Some(order) = numeric_cmp(left, right) {
            return Ok(order.is_eq());
        }
        match (left, right) {
            (RuntimeValue::List(a), RuntimeValue::List(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                if std::rc::Rc::ptr_eq(a, b) {
                    return Ok(true);
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    if !self.values_equal(x, y, depth + 1)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (RuntimeValue::Range(a, b, c), RuntimeValue::Range(x, y, z)) => {
                let len = range_len(*a, *b, *c);
                Ok(len == range_len(*x, *y, *z) && (len == 0 || (a == x && (len == 1 || c == z))))
            }
            _ => Ok(left == right),
        }
    }

    fn values_order(
        &mut self,
        left: &RuntimeValue,
        right: &RuntimeValue,
        depth: usize,
    ) -> Result<Option<std::cmp::Ordering>, Diagnostic> {
        self.tick(depth)?;
        if let Some(order) = numeric_cmp(left, right) {
            return Ok(Some(order));
        }
        match (left, right) {
            (RuntimeValue::Text(a), RuntimeValue::Text(b)) => Ok(Some(a.cmp(b))),
            (RuntimeValue::List(a), RuntimeValue::List(b)) => {
                if std::rc::Rc::ptr_eq(a, b) {
                    return Ok(Some(std::cmp::Ordering::Equal));
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    if !self.values_equal(x, y, depth + 1)? {
                        return self.values_order(x, y, depth + 1);
                    }
                }
                Ok(Some(a.len().cmp(&b.len())))
            }
            _ => Ok(None),
        }
    }

    fn eval(&mut self, expr: &IrExpr, depth: usize) -> Result<RuntimeValue, Diagnostic> {
        self.tick(depth)?;
        match expr {
            IrExpr::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item, depth + 1)?);
                }
                Ok(RuntimeValue::List(std::rc::Rc::new(values)))
            }
            IrExpr::Index { target, index } => {
                let target = self.eval(target, depth + 1)?;
                let RuntimeValue::Int(index) = self.eval(index, depth + 1)? else {
                    return Err(unsupported("Index must be an integer"));
                };
                let position = |len: usize| {
                    let i = if index < 0 {
                        index + len as i128
                    } else {
                        index
                    };
                    usize::try_from(i)
                        .ok()
                        .filter(|i| *i < len)
                        .ok_or_else(|| unsupported("Index outside sequence bounds"))
                };
                match target {
                    RuntimeValue::List(items) => Ok(items[position(items.len())?].clone()),
                    RuntimeValue::Text(text) => {
                        let i = position(text.chars().count())?;
                        Ok(RuntimeValue::Text(
                            text.chars().nth(i).expect("validated index").to_string(),
                        ))
                    }
                    RuntimeValue::Range(start, stop, step) => range_at(start, stop, step, index)
                        .map(RuntimeValue::Int)
                        .ok_or_else(|| unsupported("Index outside sequence bounds")),
                    _ => Err(unsupported("Indexing requires a list, text or range")),
                }
            }
            IrExpr::Number(text) => {
                if !text.bytes().all(|b| b.is_ascii_digit() || b == b'_') {
                    let value = text
                        .replace('_', "")
                        .parse::<f64>()
                        .map_err(|_| unsupported("Invalid floating-point literal"))?;
                    return float_value(value);
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
            IrExpr::Name(Name::User(name)) => self.lookup(name).cloned().ok_or_else(|| {
                error(
                    4001,
                    format!("Unknown variable {name:?}"),
                    "Declare the variable before using it.",
                )
            }),
            IrExpr::Name(Name::Builtin(name)) => Ok(RuntimeValue::Builtin(name)),
            IrExpr::Unary { op, value } => {
                let value = self.eval(value, depth + 1)?;
                match (op.as_str(), value) {
                    ("+", RuntimeValue::Int(n)) => Ok(RuntimeValue::Int(n)),
                    ("+", RuntimeValue::Float(bits)) => float_value(f64::from_bits(bits)),
                    ("-", RuntimeValue::Float(bits)) => float_value(-f64::from_bits(bits)),
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
                // Boolean operations are lazy, so a skipped branch never
                // evaluates its expressions or produces their side effects.
                if op == "and" || op == "or" {
                    let left = self.eval(lhs, depth + 1)?;
                    let RuntimeValue::Bool(a) = left else {
                        return Err(unsupported("Logical operators require boolean operands"));
                    };
                    if (op == "and" && !a) || (op == "or" && a) {
                        return Ok(RuntimeValue::Bool(a));
                    }
                    let right = self.eval(rhs, depth + 1)?;
                    let RuntimeValue::Bool(b) = right else {
                        return Err(unsupported("Logical operators require boolean operands"));
                    };
                    return Ok(RuntimeValue::Bool(b));
                }
                let left = self.eval(lhs, depth + 1)?;
                let right = self.eval(rhs, depth + 1)?;
                if op == "==" || op == "!=" {
                    let equal = self.values_equal(&left, &right, depth + 1)?;
                    return Ok(RuntimeValue::Bool(if op == "==" { equal } else { !equal }));
                }
                if matches!(op.as_str(), "<" | ">" | "<=" | ">=") {
                    let order = self
                        .values_order(&left, &right, depth + 1)?
                        .ok_or_else(|| unsupported("Values cannot be ordered"))?;
                    return Ok(RuntimeValue::Bool(match op.as_str() {
                        "<" => order.is_lt(),
                        ">" => order.is_gt(),
                        "<=" => !order.is_gt(),
                        _ => !order.is_lt(),
                    }));
                }
                if op == "**" {
                    if let Some(exponent) = as_float(&right) {
                        if exponent.abs() > 100.0 {
                            return Err(error(
                                4005,
                                "Exponent exceeds the evaluator limit",
                                "Use an exponent between -100 and 100.",
                            ));
                        }
                    }
                    if let (RuntimeValue::Int(a), RuntimeValue::Int(b)) = (&left, &right) {
                        if *b >= 0 {
                            return a.checked_pow(*b as u32).map(RuntimeValue::Int).ok_or_else(
                                || error(4003, "Integer power overflow", "Use smaller operands."),
                            );
                        }
                    }
                    if let (Some(a), Some(b)) = (as_float(&left), as_float(&right)) {
                        if a == 0.0 && b < 0.0 {
                            return Err(error(
                                4002,
                                "Zero cannot be raised to a negative exponent",
                                "Use a nonzero base.",
                            ));
                        }
                        return float_value(a.powf(b));
                    }
                }
                if op == "/"
                    || matches!(&left, RuntimeValue::Float(_))
                    || matches!(&right, RuntimeValue::Float(_))
                {
                    if let (Some(a), Some(b)) = (as_float(&left), as_float(&right)) {
                        if b == 0.0 && matches!(op.as_str(), "/" | "//" | "%") {
                            return Err(error(
                                4002,
                                "Division by zero at runtime",
                                "Ensure the divisor is nonzero before dividing.",
                            ));
                        }
                        let value = match op.as_str() {
                            "+" => a + b,
                            "-" => a - b,
                            "*" => a * b,
                            "/" => a / b,
                            "//" => (a / b).floor(),
                            "%" => {
                                let r = a % b;
                                if r != 0.0 && r.is_sign_negative() != b.is_sign_negative() {
                                    r + b
                                } else {
                                    r
                                }
                            }
                            _ => {
                                return Err(unsupported(format!(
                                    "Unsupported floating-point operator {op:?}"
                                )))
                            }
                        };
                        return float_value(value);
                    }
                }
                match (left, right) {
                    (RuntimeValue::Int(a), RuntimeValue::Int(b)) => {
                        if matches!(op.as_str(), "<" | ">" | "<=" | ">=") {
                            let yes = match op.as_str() {
                                "<" => a < b,
                                ">" => a > b,
                                "<=" => a <= b,
                                _ => a >= b,
                            };
                            return Ok(RuntimeValue::Bool(yes));
                        }
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
                    (RuntimeValue::List(a), RuntimeValue::List(b)) if op == "+" => {
                        if a.len().saturating_add(b.len()) > MAX_STEPS {
                            return Err(error(
                                4005,
                                "List exceeds the evaluator item budget",
                                "Use shorter lists.",
                            ));
                        }
                        let mut items = a.as_ref().clone();
                        items.extend(b.iter().cloned());
                        Ok(RuntimeValue::List(std::rc::Rc::new(items)))
                    }
                    (RuntimeValue::Text(a), RuntimeValue::Int(b))
                    | (RuntimeValue::Int(b), RuntimeValue::Text(a))
                        if op == "*" =>
                    {
                        if b <= 0 || a.is_empty() {
                            return Ok(RuntimeValue::Text(String::new()));
                        }
                        let count = usize::try_from(b)
                            .ok()
                            .filter(|count| a.len().saturating_mul(*count) <= MAX_OUTPUT)
                            .ok_or_else(|| {
                                error(
                                    4005,
                                    "Text repetition exceeds budget",
                                    "Use a smaller count.",
                                )
                            })?;
                        Ok(RuntimeValue::Text(a.repeat(count)))
                    }
                    (RuntimeValue::List(a), RuntimeValue::Int(b))
                    | (RuntimeValue::Int(b), RuntimeValue::List(a))
                        if op == "*" =>
                    {
                        if b <= 0 || a.is_empty() {
                            return Ok(RuntimeValue::List(std::rc::Rc::new(Vec::new())));
                        }
                        let count = usize::try_from(b)
                            .ok()
                            .filter(|count| a.len().saturating_mul(*count) <= MAX_STEPS)
                            .ok_or_else(|| {
                                error(
                                    4005,
                                    "List repetition exceeds budget",
                                    "Use a smaller count.",
                                )
                            })?;
                        let mut items = Vec::with_capacity(a.len() * count);
                        for _ in 0..count {
                            items.extend(a.iter().cloned());
                        }
                        Ok(RuntimeValue::List(std::rc::Rc::new(items)))
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
            IrExpr::Call { callee, args }
                if matches!(
                    callee.as_ref(),
                    IrExpr::Name(Name::Builtin("abs" | "sum" | "min" | "max"))
                ) =>
            {
                let IrExpr::Name(Name::Builtin(name)) = callee.as_ref() else {
                    unreachable!()
                };
                if args.is_empty() || (*name == "abs" && args.len() != 1) {
                    return Err(error(
                        4007,
                        format!("Invalid argument count for {name}"),
                        "Pass numeric arguments.",
                    ));
                }
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval(arg, depth + 1)?;
                    values.push(value);
                }
                if *name != "abs" && values.len() == 1 {
                    match &values[0] {
                        RuntimeValue::List(items) => values = items.as_ref().clone(),
                        RuntimeValue::Text(text) if *name == "min" || *name == "max" => {
                            let selected = if *name == "min" {
                                text.chars().min()
                            } else {
                                text.chars().max()
                            };
                            return selected
                                .map(|ch| RuntimeValue::Text(ch.to_string()))
                                .ok_or_else(|| unsupported("Aggregate requires nonempty text"));
                        }
                        RuntimeValue::Range(start, stop, step) => {
                            let (start, stop, step) = (*start, *stop, *step);
                            let count = range_len(start, stop, step);
                            if count == 0 && *name == "sum" {
                                return Ok(RuntimeValue::Int(0));
                            }
                            if count == 0 {
                                return Err(error(
                                    4007,
                                    "Aggregate requires a nonempty range",
                                    "Pass a nonempty range.",
                                ));
                            }
                            let last = range_at(start, stop, step, -1)
                                .expect("nonempty range has a last element");
                            if *name == "min" {
                                return Ok(RuntimeValue::Int(start.min(last)));
                            }
                            if *name == "max" {
                                return Ok(RuntimeValue::Int(start.max(last)));
                            }
                            // Bound summation work, retaining checked integer addition.
                            let mut total = 0_i128;
                            let mut current = start;
                            loop {
                                self.tick(depth)?;
                                total = total.checked_add(current).ok_or_else(|| {
                                    error(4003, "Integer sum overflow", "Use smaller integers.")
                                })?;
                                if current == last {
                                    return Ok(RuntimeValue::Int(total));
                                }
                                current = current.checked_add(step).ok_or_else(|| {
                                    error(
                                        4003,
                                        "Range arithmetic overflow",
                                        "Use smaller integers.",
                                    )
                                })?;
                            }
                        }
                        _ => {}
                    }
                }
                if matches!(*name, "abs" | "sum")
                    && values.iter().any(|value| as_float(value).is_none())
                {
                    return Err(unsupported(format!("{name} requires numeric arguments")));
                }
                if values.is_empty() {
                    if *name == "sum" {
                        return Ok(RuntimeValue::Int(0));
                    }
                    return Err(error(
                        4007,
                        format!("{name} requires a nonempty sequence"),
                        "Pass at least one numeric value.",
                    ));
                }
                if *name == "abs" {
                    return match values.remove(0) {
                        RuntimeValue::Int(n) => {
                            n.checked_abs().map(RuntimeValue::Int).ok_or_else(|| {
                                error(
                                    4003,
                                    "Integer absolute value overflow",
                                    "Use a smaller integer.",
                                )
                            })
                        }
                        RuntimeValue::Float(bits) => float_value(f64::from_bits(bits).abs()),
                        _ => unreachable!(),
                    };
                }
                let mut iter = values.into_iter();
                let mut result = iter.next().expect("nonempty arguments");
                for value in iter {
                    result = match *name {
                        "sum" => match (&result, &value) {
                            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => {
                                RuntimeValue::Int(a.checked_add(*b).ok_or_else(|| {
                                    error(
                                        4003,
                                        "Integer sum overflow",
                                        "Use smaller integer operands.",
                                    )
                                })?)
                            }
                            _ => {
                                float_value(as_float(&result).unwrap() + as_float(&value).unwrap())?
                            }
                        },
                        "min" | "max" => {
                            let order = self
                                .values_order(&value, &result, depth + 1)?
                                .ok_or_else(|| unsupported("Aggregate values cannot be ordered"))?;
                            if (*name == "min" && order.is_lt())
                                || (*name == "max" && order.is_gt())
                            {
                                value
                            } else {
                                result
                            }
                        }
                        _ => result,
                    };
                }
                Ok(result)
            }
            IrExpr::Call { callee, args }
                if matches!(
                    callee.as_ref(),
                    IrExpr::Name(Name::Builtin("int" | "float" | "str" | "bool"))
                ) =>
            {
                let IrExpr::Name(Name::Builtin(name)) = callee.as_ref() else {
                    unreachable!()
                };
                if args.len() > 1 {
                    return Err(error(
                        4007,
                        format!("{name} expects zero or one argument"),
                        "Pass at most one value.",
                    ));
                }
                let value = match args.first() {
                    Some(arg) => self.eval(arg, depth + 1)?,
                    None => match *name {
                        "int" => RuntimeValue::Int(0),
                        "float" => return float_value(0.0),
                        "str" => RuntimeValue::Text(String::new()),
                        _ => RuntimeValue::Bool(false),
                    },
                };
                match *name {
                    "str" => Ok(RuntimeValue::Text(value.display())),
                    "bool" => Ok(RuntimeValue::Bool(match value {
                        RuntimeValue::Null => false,
                        RuntimeValue::Bool(b) => b,
                        RuntimeValue::Int(n) => n != 0,
                        RuntimeValue::Float(bits) => f64::from_bits(bits) != 0.0,
                        RuntimeValue::Text(text) => !text.is_empty(),
                        RuntimeValue::List(items) => !items.is_empty(),
                        RuntimeValue::Range(start, stop, step) => range_len(start, stop, step) != 0,
                        RuntimeValue::Function(_) | RuntimeValue::Builtin(_) => true,
                    })),
                    "int" => match value {
                        RuntimeValue::Int(n) => Ok(RuntimeValue::Int(n)),
                        RuntimeValue::Bool(b) => Ok(RuntimeValue::Int(i128::from(b))),
                        RuntimeValue::Float(bits) => {
                            let n = f64::from_bits(bits);
                            if n >= 2.0_f64.powi(127) || n < -2.0_f64.powi(127) {
                                return Err(unsupported("Integer conversion outside i128 range"));
                            }
                            Ok(RuntimeValue::Int(n as i128))
                        }
                        RuntimeValue::Text(text) => text
                            .trim()
                            .parse::<i128>()
                            .map(RuntimeValue::Int)
                            .map_err(|_| unsupported("Invalid integer conversion")),
                        _ => Err(unsupported("Invalid integer conversion")),
                    },
                    "float" => match value {
                        RuntimeValue::Bool(b) => float_value(if b { 1.0 } else { 0.0 }),
                        RuntimeValue::Text(text) => float_value(
                            text.trim()
                                .parse::<f64>()
                                .map_err(|_| unsupported("Invalid float conversion"))?,
                        ),
                        _ => float_value(
                            as_float(&value)
                                .ok_or_else(|| unsupported("Invalid float conversion"))?,
                        ),
                    },
                    _ => unreachable!(),
                }
            }
            IrExpr::Call { callee, args }
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("round"))) =>
            {
                if !(1..=2).contains(&args.len()) {
                    return Err(error(
                        4007,
                        "round expects one or two arguments",
                        "Pass a number and optionally an integer precision.",
                    ));
                }
                let value = self.eval(&args[0], depth + 1)?;
                let precision = if args.len() == 2 {
                    let RuntimeValue::Int(n) = self.eval(&args[1], depth + 1)? else {
                        return Err(unsupported("round precision must be an integer"));
                    };
                    Some(n)
                } else {
                    None
                };
                if let RuntimeValue::Int(n) = value {
                    let digits = precision.unwrap_or(0);
                    if digits >= 0 {
                        return Ok(RuntimeValue::Int(n));
                    }
                    if digits < -38 {
                        return Ok(RuntimeValue::Int(0));
                    }
                    let factor = 10_i128.pow((-digits) as u32);
                    let quotient = n / factor;
                    let remainder = (n % factor).abs();
                    let half = factor / 2;
                    let adjustment = if remainder > half || (remainder == half && quotient % 2 != 0)
                    {
                        n.signum()
                    } else {
                        0
                    };
                    return quotient
                        .checked_add(adjustment)
                        .and_then(|q| q.checked_mul(factor))
                        .map(RuntimeValue::Int)
                        .ok_or_else(|| {
                            error(4003, "Rounded integer overflow", "Use a smaller number.")
                        });
                }
                let n = as_float(&value).ok_or_else(|| unsupported("round requires a number"))?;
                match precision {
                    None => {
                        let rounded = n.round_ties_even();
                        if rounded >= 2.0_f64.powi(127) || rounded < -2.0_f64.powi(127) {
                            return Err(error(
                                4003,
                                "Rounded integer overflow",
                                "Use a smaller number.",
                            ));
                        }
                        Ok(RuntimeValue::Int(rounded as i128))
                    }
                    Some(digits) if digits >= 324 => float_value(n),
                    Some(digits) if digits <= -309 => float_value(0.0),
                    Some(digits) if digits >= 0 => {
                        // Rust's decimal formatter rounds the actual binary value,
                        // avoiding the double rounding caused by multiplying first.
                        let text = format!("{:.*}", digits as usize, n);
                        float_value(
                            text.parse::<f64>().map_err(|_| {
                                unsupported("Invalid rounded floating-point result")
                            })?,
                        )
                    }
                    Some(digits) => {
                        let factor = 10_f64.powi((-digits) as i32);
                        float_value((n / factor).round_ties_even() * factor)
                    }
                }
            }
            IrExpr::Call { callee, args }
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("range"))) =>
            {
                if !(1..=3).contains(&args.len()) {
                    return Err(unsupported("range requires one to three integer arguments"));
                }
                let mut numbers = Vec::with_capacity(args.len());
                for arg in args {
                    let RuntimeValue::Int(value) = self.eval(arg, depth + 1)? else {
                        return Err(unsupported("range arguments must be integers"));
                    };
                    numbers.push(value);
                }
                let (start, stop, step) = match numbers.as_slice() {
                    [stop] => (0, *stop, 1),
                    [start, stop] => (*start, *stop, 1),
                    [start, stop, step] => (*start, *stop, *step),
                    _ => unreachable!("range arity validated"),
                };
                if step == 0 {
                    return Err(error(
                        4006,
                        "range step cannot be zero",
                        "Use a nonzero step argument.",
                    ));
                }
                Ok(RuntimeValue::Range(start, stop, step))
            }
            IrExpr::Call { callee, args }
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("len"))) =>
            {
                if args.len() != 1 {
                    return Err(error(
                        4007,
                        format!(
                            "len expects exactly one string argument, received {}",
                            args.len()
                        ),
                        "Pass one UTF-8 string value to len.",
                    ));
                }
                let value = self.eval(&args[0], depth + 1)?;
                match value {
                    RuntimeValue::Text(text) => Ok(RuntimeValue::Int(text.chars().count() as i128)),
                    RuntimeValue::List(items) => Ok(RuntimeValue::Int(items.len() as i128)),
                    RuntimeValue::Range(start, stop, step) => {
                        let count = i128::try_from(range_len(start, stop, step)).map_err(|_| {
                            error(4003, "Range length overflow", "Use a shorter range.")
                        })?;
                        Ok(RuntimeValue::Int(count))
                    }
                    _ => Err(unsupported("len requires a list or string")),
                }
            }
            IrExpr::Call { callee, args } => {
                let label = match callee.as_ref() {
                    IrExpr::Name(Name::User(name)) => name.as_str(),
                    _ => "<expression>",
                };
                let name = label;
                let id = match self.eval(callee, depth + 1)? {
                    RuntimeValue::Function(id) => id,
                    RuntimeValue::Builtin(builtin) => {
                        let call = IrExpr::Call {
                            callee: Box::new(IrExpr::Name(Name::Builtin(builtin))),
                            args: args.clone(),
                        };
                        // Only dispatch builtins implemented by the direct call path.
                        // Reject other names instead of recursively redispatching them.
                        if !matches!(
                            builtin,
                            "print"
                                | "len"
                                | "abs"
                                | "sum"
                                | "min"
                                | "max"
                                | "int"
                                | "float"
                                | "str"
                                | "bool"
                                | "round"
                                | "range"
                        ) {
                            return Err(unsupported("Builtin is not callable as a value yet"));
                        }
                        return self.eval(&call, depth + 1);
                    }
                    _ => return Err(unsupported("Value is not callable")),
                };
                let function = self
                    .functions
                    .get(id)
                    .cloned()
                    .ok_or_else(|| unsupported("Invalid function reference"))?;
                if args.len() != function.parameters.len() {
                    return Err(error(
                        4007,
                        format!(
                            "Function {name:?} expects {} argument(s), received {}",
                            function.parameters.len(),
                            args.len()
                        ),
                        "Pass exactly the declared number of arguments.",
                    ));
                }
                if self.active_calls.len() >= 32 {
                    return Err(error(
                        4005,
                        format!("Recursive or excessive nested function call to {name:?}"),
                        "Reduce recursive call depth or ensure the function reaches a base case.",
                    ));
                }
                let mut parameters = BTreeMap::new();
                for ((parameter, annotation), arg) in function
                    .parameters
                    .iter()
                    .zip(&function.parameter_types)
                    .zip(args)
                {
                    let value = self.eval(arg, depth + 1)?;
                    if annotation
                        .as_deref()
                        .is_some_and(|annotation| !matches_explicit_type(&value, annotation))
                    {
                        return Err(unsupported(format!(
                            "Function {name:?} argument {parameter:?} violates its explicit type annotation"
                        )));
                    }
                    parameters.insert(parameter.clone(), (value, annotation.clone()));
                }
                // Use the declaration environment, never the caller environment.
                let saved = self.current;
                self.current = self.new_scope(function.closure);
                self.scopes[self.current].bindings = parameters;
                self.active_calls.push(name.to_string());
                let result = self.statements(&function.body);
                self.active_calls.pop();
                self.current = saved;
                let returned = match result? {
                    Flow::Next => RuntimeValue::Null,
                    Flow::Return(value) => value,
                    Flow::Break | Flow::Continue => {
                        return Err(unsupported("Loop control escaped a function body"));
                    }
                };
                if function
                    .return_type
                    .as_deref()
                    .is_some_and(|annotation| !matches_explicit_type(&returned, annotation))
                {
                    return Err(unsupported(format!(
                        "Function {name:?} returned a value incompatible with its explicit return type"
                    )));
                }
                Ok(returned)
            }
        }
    }
    fn statement(&mut self, stmt: &IrStatement) -> Result<Flow, Diagnostic> {
        self.tick(0)?;
        match stmt {
            IrStatement::Let {
                name,
                type_name,
                expr,
            } => {
                if self.scopes[self.current].bindings.contains_key(name) {
                    return Err(unsupported(format!("Duplicate declaration {name:?}")));
                }
                let value = self.eval(expr, 1)?;
                if type_name
                    .as_deref()
                    .is_some_and(|annotation| !matches_explicit_type(&value, annotation))
                {
                    return Err(unsupported(format!(
                        "Declaration {name:?} violates its explicit type annotation"
                    )));
                }
                self.scopes[self.current]
                    .bindings
                    .insert(name.clone(), (value, type_name.clone()));
                Ok(Flow::Next)
            }
            IrStatement::Assign { name, expr } => {
                // Evaluate the RHS before mutating an existing binding.
                if self.lookup(name).is_none() {
                    return Err(error(
                        4001,
                        format!("Assignment to unknown variable {name:?}"),
                        "Declare the variable first.",
                    ));
                }
                let value = self.eval(expr, 1)?;
                self.assign(name, value)?;
                Ok(Flow::Next)
            }
            IrStatement::Expression(expr) => {
                self.eval(expr, 1)?;
                Ok(Flow::Next)
            }
            IrStatement::If {
                branches,
                otherwise,
            } => {
                for (condition, body) in branches {
                    let value = self.eval(condition, 1)?;
                    let RuntimeValue::Bool(yes) = value else {
                        return Err(unsupported(
                            "Conditional expressions must evaluate to a boolean",
                        ));
                    };
                    if yes {
                        return self.block(body);
                    }
                }
                if let Some(body) = otherwise {
                    return self.block(body);
                }
                Ok(Flow::Next)
            }
            IrStatement::While { condition, body } => {
                loop {
                    // Charge each iteration even if the condition contains
                    // no user expressions or the body is empty.
                    self.tick(0)?;
                    let value = self.eval(condition, 1)?;
                    let RuntimeValue::Bool(yes) = value else {
                        return Err(unsupported("While conditions must evaluate to a boolean"));
                    };
                    if !yes {
                        break;
                    }
                    match self.block(body)? {
                        Flow::Next | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                    }
                }
                Ok(Flow::Next)
            }
            IrStatement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable = self.eval(iterable, 1)?;
                let (mut current, stop, step) = match iterable {
                    RuntimeValue::Range(start, stop, step) => (start, stop, step),
                    other => {
                        let values = match other {
                            RuntimeValue::List(items) => items,
                            RuntimeValue::Text(text) => {
                                if text.chars().count() > MAX_STEPS {
                                    return Err(error(
                                        4005,
                                        "Text iteration exceeds the evaluator item budget",
                                        "Use a shorter string.",
                                    ));
                                }
                                std::rc::Rc::new(
                                    text.chars()
                                        .map(|ch| RuntimeValue::Text(ch.to_string()))
                                        .collect(),
                                )
                            }
                            _ => {
                                return Err(unsupported("For loops require a list, text or range"))
                            }
                        };
                        for value in values.iter() {
                            self.tick(0)?;
                            let saved = self.current;
                            self.current = self.new_scope(saved);
                            self.scopes[self.current]
                                .bindings
                                .insert(variable.clone(), (value.clone(), None));
                            let result = self.statements(body);
                            self.current = saved;
                            match result? {
                                Flow::Break => break,
                                Flow::Next | Flow::Continue => {}
                                Flow::Return(value) => return Ok(Flow::Return(value)),
                            }
                        }
                        return Ok(Flow::Next);
                    }
                };
                while if step > 0 {
                    current < stop
                } else {
                    current > stop
                } {
                    // An empty body is still charged for each iteration.
                    self.tick(0)?;
                    let saved = self.current;
                    self.current = self.new_scope(saved);
                    self.scopes[self.current]
                        .bindings
                        .insert(variable.clone(), (RuntimeValue::Int(current), None));
                    let result = self.statements(body);
                    self.current = saved;
                    match result? {
                        Flow::Break => break,
                        Flow::Next | Flow::Continue => {}
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                    }
                    // Last-value overflow can occur near i128 bounds; when
                    // that happens, the next value is beyond the supported
                    // integer domain, so the range terminates safely.
                    let Some(next) = current.checked_add(step) else {
                        break;
                    };
                    current = next;
                }
                Ok(Flow::Next)
            }
            IrStatement::Function {
                name,
                parameters,
                parameter_types,
                return_type,
                body,
            } => {
                if self.scopes[self.current].bindings.contains_key(name) {
                    return Err(unsupported(format!(
                        "Duplicate function declaration {name:?}"
                    )));
                }
                let id = self.functions.len();
                self.functions.push(Function {
                    parameters: parameters.clone(),
                    parameter_types: parameter_types.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                    closure: self.current,
                });
                self.scopes[self.current]
                    .bindings
                    .insert(name.clone(), (RuntimeValue::Function(id), None));
                Ok(Flow::Next)
            }
            IrStatement::Return(expr) => Ok(Flow::Return(self.eval(expr, 1)?)),
            IrStatement::Break => Ok(Flow::Break),
            IrStatement::Continue => Ok(Flow::Continue),
        }
    }
}
/// Explicit execution entrypoint. Parsing and type-checking never invoke this.
pub fn evaluate_ir(module: &Module) -> Result<String, Diagnostic> {
    evaluate_ir_with_max_steps(module, MAX_STEPS)
}

pub fn evaluate_ir_with_max_steps(module: &Module, max_steps: usize) -> Result<String, Diagnostic> {
    let mut evaluator = Evaluator {
        scopes: vec![Scope::default()],
        current: 0,
        functions: Vec::new(),
        active_calls: Vec::new(),
        steps: 0,
        max_steps,
        output: String::new(),
    };
    match evaluator.statements(&module.statements)? {
        Flow::Next => Ok(evaluator.output),
        Flow::Break | Flow::Continue | Flow::Return(_) => {
            Err(unsupported("Flow control outside a valid lexical context"))
        }
    }
}
pub fn evaluate_source(source: &str, profile: &str) -> Result<String, Diagnostic> {
    evaluate_source_with_max_steps(source, profile, MAX_STEPS)
}

pub fn evaluate_source_with_max_steps(
    source: &str,
    profile: &str,
    max_steps: usize,
) -> Result<String, Diagnostic> {
    evaluate_ir_with_max_steps(&lower_source(source, profile)?, max_steps)
}

#[cfg(test)]
mod len_tests {
    use super::*;

    #[test]
    fn direct_recursion_computes_factorial() {
        let source = "fn factorial(n: int): int { if n <= 1 { return 1 } else { return n * factorial(n - 1) } }\nprint(factorial(5))\n";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "120\n");
    }

    #[test]
    fn unbounded_recursion_stops_at_call_budget() {
        let source = "fn recurse(n: int): int { return recurse(n) }\nprint(recurse(1))\n";
        assert_eq!(evaluate_source(source, "ai").unwrap_err().code, 4005);
    }

    #[test]
    fn unicode_length_counts_scalars_not_utf8_bytes() {
        for (profile, source, expected) in [
            (
                "ai",
                "print(len(\"Život ☸\"), len(\"\"), len(\"👩‍💻\"))",
                "7 0 3\n",
            ),
            (
                "sk",
                "vypis(dlzka(\"Život ☸\"), dlzka(\"\"), dlzka(\"👩‍💻\"))",
                "7 0 3\n",
            ),
            (
                "en",
                "print(len(\"Život ☸\"), len(\"\"), len(\"👩‍💻\"))",
                "7 0 3\n",
            ),
        ] {
            assert_eq!(evaluate_source(source, profile).unwrap(), expected);
        }
    }
    #[test]
    fn len_invalid_arity_and_type_fail_with_numbered_diagnostics() {
        assert_eq!(
            evaluate_source("print(len())", "ai").unwrap_err().code,
            4007
        );
        assert_eq!(
            evaluate_source("print(len(\"a\", \"b\"))", "ai")
                .unwrap_err()
                .code,
            4007
        );
        assert_eq!(
            evaluate_source("print(len(2))", "ai").unwrap_err().code,
            4004
        );
    }
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
    fn explicit_function_signatures_enforce_runtime_types() {
        let program =
            "fn choose(flag: bool, a: int, b: int): int { if flag { return a } else { return b } }
print(choose(true, 7, 3))
print(choose(false, 7, 3))";
        assert_eq!(evaluate_source(program, "ai").unwrap(), "7\n3\n");
        assert_eq!(
            evaluate_source(
                "funkcia choose(flag: logicke, a: cele, b: cele): cele { ak flag { vrat a } inak { vrat b } }\nvypis(choose(pravda, 7, 3))",
                "sk",
            ).unwrap(),
            "7\n",
        );
        assert_eq!(
            evaluate_source("fn f(flag: bool): bool { return flag }\nprint(f(4))", "ai")
                .unwrap_err()
                .code,
            4004,
        );
        assert_eq!(
            evaluate_source("fn f(x: int): bool { return x }\nprint(f(4))", "ai")
                .unwrap_err()
                .code,
            4004,
        );
        assert_eq!(
            evaluate_source("fn f(x: int): bool { print(x) }\nprint(f(4))", "ai")
                .unwrap_err()
                .code,
            4004,
        );
    }

    #[test]
    fn named_function_returns_a_value() {
        assert_eq!(
            evaluate_source("fn f() { return 1 }\nprint(f())", "ai").unwrap(),
            "1\n"
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
    fn true_division_and_float_arithmetic() {
        assert_eq!(
            evaluate_source("print(3 / 2, 1.5 + 2, -2.5 * 2)", "ai").unwrap(),
            "1.5 3.5 -5.0\n"
        );
    }
    #[test]
    fn float_comparison_and_zero_divisor() {
        assert_eq!(
            evaluate_source(
                "print(2 == 2.0, 9007199254740993 > 9007199254740992.0)",
                "ai"
            )
            .unwrap(),
            "true true\n"
        );
        assert_eq!(
            evaluate_source("print(1.0 / 0.0)", "ai").unwrap_err().code,
            4002
        );
    }
    #[test]
    fn float_annotations_are_enforced() {
        assert_eq!(
            evaluate_source("let x: float = 1.5\nprint(x)", "ai").unwrap(),
            "1.5\n"
        );
        assert_eq!(
            evaluate_source("let x: float = 1.5\nx = 2", "ai")
                .unwrap_err()
                .code,
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

#[cfg(test)]
mod numeric_builtin_tests {
    use super::*;

    #[test]
    fn numeric_builtins() {
        assert_eq!(
            evaluate_source("print(abs(-7), sum(1, 2, 3), min(4, -2), max(1, 3))", "ai").unwrap(),
            "7 6 -2 3\n"
        );
    }

    #[test]
    fn numeric_builtins_reject_invalid_arguments() {
        assert_eq!(evaluate_source("abs()", "ai").unwrap_err().code, 4007);
        assert_eq!(
            evaluate_source("sum(1, \"x\")", "ai").unwrap_err().code,
            4004
        );
        assert_eq!(evaluate_source("max()", "ai").unwrap_err().code, 4007);
    }

    #[test]
    fn mixed_numeric_builtins() {
        assert_eq!(
            evaluate_source("print(sum(1, 2.5), min(2, 1.5), abs(-1.5))", "ai").unwrap(),
            "3.5 1.5 1.5\n"
        );
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;
    #[test]
    fn list_index_and_length() {
        assert_eq!(
            evaluate_source(
                "let x = [1, [2, 3], 4]\nprint(len(x), x[1][0], x[-1])",
                "ai"
            )
            .unwrap(),
            "3 2 4\n"
        );
    }
    #[test]
    fn index_bounds_and_unicode() {
        assert!(evaluate_source("print([][0])", "ai").is_err());
        assert!(evaluate_source("print([1][-2])", "ai").is_err());
        assert_eq!(evaluate_source("print(\"Ža\"[0])", "ai").unwrap(), "Ž\n");
    }
}

#[cfg(test)]
mod sequence_operation_tests {
    use super::*;

    #[test]
    fn concatenate_lists_without_mutating_inputs() {
        let source = "let a = [1, 2]\nlet b = a + [3]\nprint(len(a), len(b), b[2])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "2 3 3\n");
    }

    #[test]
    fn numeric_aggregates_accept_lists() {
        let source = "print(sum([1, 2, 3]), min([4, 2]), max([1, 7]), sum([]))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "6 2 7 0\n");
    }

    #[test]
    fn invalid_aggregates_reject_empty_or_nonnumeric_sequences() {
        assert_eq!(evaluate_source("min([])", "ai").unwrap_err().code, 4007);
        assert_eq!(
            evaluate_source("sum([1, \"x\"])", "ai").unwrap_err().code,
            4004
        );
    }
}

#[cfg(test)]
mod sequence_iteration_tests {
    use super::*;

    #[test]
    fn list_iteration_and_loop_control() {
        let source = "let total = 0\nfor x in [1, 2, 3, 4] { if x == 2 { continue } if x == 4 { break } total = total + x }\nprint(total)";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "4\n");
    }

    #[test]
    fn text_iteration_uses_unicode_scalars() {
        let source = "for ch in \"Ža\" { print(ch) }";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "Ž\na\n");
    }

    #[test]
    fn empty_list_and_invalid_iterable() {
        assert_eq!(
            evaluate_source("for x in [] { print(x) }", "ai").unwrap(),
            ""
        );
        assert!(evaluate_source("for x in 3 { print(x) }", "ai").is_err());
    }
}

#[cfg(test)]
mod legacy_numeric_operations {
    use super::*;

    #[test]
    fn bounded_powers() {
        assert_eq!(
            evaluate_source("print(2 ** 10, 2 ** -2, 4.0 ** 0.5)", "ai").unwrap(),
            "1024 0.25 2.0\n"
        );
        assert_eq!(evaluate_source("2 ** 101", "ai").unwrap_err().code, 4005);
    }

    #[test]
    fn basic_conversions() {
        assert_eq!(
            evaluate_source("print(int(3.9), float(\"2.5\"), str(42), bool([]))", "ai").unwrap(),
            "3 2.5 42 false\n"
        );
    }
}

#[cfg(test)]
mod rounding_tests {
    use super::*;

    #[test]
    fn ties_round_to_even() {
        assert_eq!(
            evaluate_source("print(round(2.5), round(3.5), round(-2.5))", "ai").unwrap(),
            "2 4 -2\n"
        );
    }

    #[test]
    fn decimal_precision_and_integer_rounding() {
        assert_eq!(
            evaluate_source(
                "print(round(2.675, 2), round(125, -1), round(135, -1))",
                "ai"
            )
            .unwrap(),
            "2.67 120 140\n"
        );
    }

    #[test]
    fn invalid_round_arguments() {
        assert_eq!(evaluate_source("round()", "ai").unwrap_err().code, 4007);
        assert_eq!(
            evaluate_source("round(1, 0.5)", "ai").unwrap_err().code,
            4004
        );
    }
}

#[cfg(test)]
mod legacy_string_tests {
    use super::*;

    #[test]
    fn escaped_unicode_and_control_characters() {
        assert_eq!(literal_text(r#""\u017d\x61\U0001f600""#).unwrap(), "Ža😀");
        assert_eq!(literal_text(r#""\101\r\t""#).unwrap(), "A\r\t");
    }

    #[test]
    fn multiline_literal_decoding() {
        assert_eq!(literal_text("\"\"\"one\ntwo\"\"\"").unwrap(), "one\ntwo");
    }

    #[test]
    fn source_pipeline_decodes_multiline_and_unicode_strings() {
        let source = "print(\"\"\"one\ntwo\"\"\")\nprint(\"\\u017d\\x61\")";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "one\ntwo\nŽa\n");
    }

    #[test]
    fn invalid_unicode_escape_is_rejected() {
        assert!(literal_text(r#""\uD800""#).is_err());
        assert!(literal_text(r#""\xQ1""#).is_err());
    }
}

#[cfg(test)]
mod builtin_value_tests {
    use super::*;

    #[test]
    fn builtin_can_be_passed_to_user_function() {
        let source = "fn apply(f, x) { return f(x) }\nprint(apply(abs, -7))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "7\n");
    }

    #[test]
    fn builtin_can_be_stored_and_indexed() {
        let source = "let f = len\nlet ops = [abs, f]\nprint(ops[0](-3), ops[1]([1, 2]))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "3 2\n");
    }
}

#[cfg(test)]
mod range_value_tests {
    use super::*;

    #[test]
    fn stored_range_and_alias_remain_lazy() {
        let source = "let f = range\nlet r = f(5, 0, -2)\nfor x in r { print(x) }";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "5\n3\n1\n");
    }

    #[test]
    fn range_length_and_empty_range() {
        let source = "print(len(range(10)), len(range(5, 0, -2)), bool(range(0)))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "10 3 false\n");
    }

    #[test]
    fn huge_range_can_break_without_allocation() {
        let source = "let r = range(1000000000000)\nfor x in r { print(x) break }";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "0\n");
    }
}

#[cfg(test)]
mod range_operations_tests {
    use super::*;

    #[test]
    fn range_indexing_and_bounds() {
        let source = "let r = range(10, 0, -2)\nprint(r[0], r[-1], r[2])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "10 2 6\n");
        assert!(evaluate_source("range(0)[0]", "ai").is_err());
        assert!(evaluate_source("range(3)[-4]", "ai").is_err());
    }

    #[test]
    fn range_aggregates() {
        let source = "print(sum(range(5)), min(range(8, 0, -2)), max(range(9)))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "10 2 8\n");
    }

    #[test]
    fn range_indexing_handles_full_signed_domain() {
        assert_eq!(range_at(i128::MIN, i128::MAX, 1, -1), Some(i128::MAX - 1));
        assert_eq!(range_at(i128::MAX, i128::MIN, -1, -1), Some(i128::MIN + 1));
    }
}

#[cfg(test)]
mod sequence_repetition_tests {
    use super::*;

    #[test]
    fn repeat_sequences_in_both_operand_orders() {
        let source = "print(\"ab\" * 3, 2 * \"x\", len([1, 2] * 3), len(0 * [1]))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "ababab xx 6 0\n");
    }

    #[test]
    fn oversized_sequence_repetition_is_rejected() {
        assert_eq!(evaluate_source("[1] * 10001", "ai").unwrap_err().code, 4005);
    }

    #[test]
    fn text_ordering() {
        assert_eq!(
            evaluate_source("print(\"a\" < \"b\", \"z\" >= \"z\")", "ai").unwrap(),
            "true true\n"
        );
    }
}

#[cfg(test)]
mod sequence_equality_tests {
    use super::*;

    #[test]
    fn nested_numeric_lists_compare_by_value() {
        let source = "print([1, [2]] == [1.0, [2.0]], [1] != [2])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "true true\n");
    }

    #[test]
    fn equivalent_ranges_compare_by_elements() {
        let source = "print(range(0) == range(4, 2), range(0, 3, 2) == range(0, 4, 2))";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "true true\n");
    }
}

#[cfg(test)]
mod sequence_order_tests {
    use super::*;

    #[test]
    fn lists_compare_lexicographically() {
        let source = "print([1, 2] < [1, 3], [1] < [1, 0], [1.0] >= [1])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "true true true\n");
    }

    #[test]
    fn nested_lists_and_text_are_ordered() {
        let source = "print([[1], [2]] < [[1], [3]], [\"a\"] < [\"b\"])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "true true\n");
    }

    #[test]
    fn incomparable_elements_are_rejected() {
        assert!(evaluate_source("[1] < [\"a\"]", "ai").is_err());
    }
}

#[cfg(test)]
mod ordered_aggregate_tests {
    use super::*;

    #[test]
    fn extrema_support_text_and_nested_lists() {
        let source = "print(min(\"zba\"), max([\"a\", \"z\"]), min([[2], [1]])[0])";
        assert_eq!(evaluate_source(source, "ai").unwrap(), "a z 1\n");
    }

    #[test]
    fn extrema_reject_incomparable_values() {
        assert!(evaluate_source("min([1, \"a\"])", "ai").is_err());
        assert!(evaluate_source("max(\"\")", "ai").is_err());
    }
}
