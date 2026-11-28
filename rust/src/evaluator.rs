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
    Int(i128),
    Float(u64),
    Bool(bool),
    Text(String),
    Function(usize),
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
            Self::Function(_) => "<function>".into(),
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
                    _ => Err(unsupported("Indexing requires a list or text")),
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
            IrExpr::Name(Name::Builtin(_)) => Err(unsupported(
                "A built-in is not a first-class value in the initial evaluator",
            )),
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
                    let equal = numeric_cmp(&left, &right)
                        .map_or_else(|| left == right, |order| order.is_eq());
                    return Ok(RuntimeValue::Bool(if op == "==" { equal } else { !equal }));
                }
                if let Some(order) = numeric_cmp(&left, &right) {
                    let comparison = match op.as_str() {
                        "<" => Some(order.is_lt()),
                        ">" => Some(order.is_gt()),
                        "<=" => Some(!order.is_gt()),
                        ">=" => Some(!order.is_lt()),
                        _ => None,
                    };
                    if let Some(value) = comparison {
                        return Ok(RuntimeValue::Bool(value));
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
                    if as_float(&value).is_none() {
                        return Err(unsupported(format!("{name} requires numeric arguments")));
                    }
                    values.push(value);
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
                        "min" if numeric_cmp(&value, &result).is_some_and(|o| o.is_lt()) => value,
                        "max" if numeric_cmp(&value, &result).is_some_and(|o| o.is_gt()) => value,
                        _ => result,
                    };
                }
                Ok(result)
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
                    _ => Err(unsupported("len requires a list or string")),
                }
            }
            IrExpr::Call { callee, args } => {
                let label = match callee.as_ref() {
                    IrExpr::Name(Name::User(name)) => name.as_str(),
                    _ => "<expression>",
                };
                let name = label;
                let RuntimeValue::Function(id) = self.eval(callee, depth + 1)? else {
                    return Err(unsupported("Value is not callable"));
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
                // Only the canonical, unshadowed range built-in is supported
                // in the initial Rust for-loop slice. Range arguments are
                // evaluated once, in the enclosing lexical environment.
                let IrExpr::Call { callee, args } = iterable else {
                    return Err(unsupported("For loops currently require range(...)"));
                };
                if !matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("range"))) {
                    return Err(unsupported(
                        "For loops currently require unshadowed range(...)",
                    ));
                }
                if !(1..=3).contains(&args.len()) {
                    return Err(unsupported(
                        "range requires one, two or three integer arguments",
                    ));
                }
                let mut numbers = Vec::with_capacity(args.len());
                for arg in args {
                    let RuntimeValue::Int(value) = self.eval(arg, 1)? else {
                        return Err(unsupported("range arguments must be integers"));
                    };
                    numbers.push(value);
                }
                let (mut current, stop, step) = match numbers.as_slice() {
                    [stop] => (0, *stop, 1),
                    [start, stop] => (*start, *stop, 1),
                    [start, stop, step] => (*start, *stop, *step),
                    _ => unreachable!("range argument count validated"),
                };
                if step == 0 {
                    return Err(error(
                        4006,
                        "range step cannot be zero",
                        "Use a nonzero step argument for range(start, stop, step).",
                    ));
                }
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
