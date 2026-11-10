//! First real Linux x86-64 machine-code backend for a deliberately small Nobody
//! subset. Emits GNU assembler source, invokes binutils as+ld, and produces an
//! ELF64 binary with its own Linux sys_write/sys_exit routines.
//!
//! No Python, Nobody VM, libc, embedded interpreter or Rust runtime is linked
//! into the output. This is not a full language backend or an OS sandbox.
//!
//! Supported: signed i64 numbers, boolean conditions, checked + - *,
//! comparisons, lexical variables, if/elif/else, while, signed for-range,
//! break/continue, lazy boolean and/or, earlier-declared integer/bool
//! parameter functions with i64/bool returns, and print(one integer or boolean).
//! Unsupported features fail before output.
use crate::ir::{IrExpr, IrStatement, Module, Name};
use crate::{lower_source, Diagnostic};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MAX_SLOTS: usize = 1024;
const MAX_ASM_BYTES: usize = 1_048_576;
const MAX_DEPTH: usize = 64;
const RUNTIME_STEPS: usize = 10_000;

fn issue(code: u16, detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair: "Use the documented native Linux x86-64 i64 subset, or use the Rust bytecode VM for other supported Nobody constructs.",
    }
}
fn unsupported(detail: impl Into<String>) -> Diagnostic {
    issue(4004, detail)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
    // A borrowed UTF-8 byte slice: pointer in rax, length in rdx.
    Text,
}

#[derive(Clone, Copy)]
struct Binding {
    slot: usize,
    kind: Type,
}

struct LoopTargets {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
struct FunctionInfo {
    label: usize,
    arity: usize,
    parameters: [Type; 32],
    result: Type,
}

struct NativeCompiler {
    asm: String,
    scopes: Vec<BTreeMap<String, Binding>>,
    slots: usize,
    labels: usize,
    loops: Vec<LoopTargets>,
    functions: BTreeMap<String, FunctionInfo>,
    in_function: bool,
    return_kind: Option<Type>,
    // Every literal is stored in read-only data as numeric bytes, never raw assembler.
    static_texts: Vec<(usize, Vec<u8>)>,
}

/// Infer only conservative parameter constraints from their direct typed
/// uses. Nobody function syntax does not yet declare parameter types.
/// Unconstrained parameters remain i64, preserving the initial native ABI.
/// There is no speculative conversion, implicit truthiness or Python fallback.
struct ParameterConstraints<'a> {
    parameters: &'a [String],
    inferred: [Option<Type>; 32],
    scopes: Vec<BTreeSet<String>>,
    functions: &'a BTreeMap<String, FunctionInfo>,
}

impl ParameterConstraints<'_> {
    fn constrain_name(&mut self, name: &str, required: Type) -> Result<(), Diagnostic> {
        if self.scopes.iter().rev().any(|scope| scope.contains(name)) {
            return Ok(());
        }
        let Some(index) = self.parameters.iter().position(|param| param == name) else {
            return Ok(());
        };
        match self.inferred[index] {
            Some(previous) if previous != required => Err(unsupported(format!(
                "Conflicting native int/bool/str constraints on function parameter {name:?}"
            ))),
            _ => {
                self.inferred[index] = Some(required);
                Ok(())
            }
        }
    }

    fn expression(
        &mut self,
        expr: &IrExpr,
        required: Option<Type>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(issue(
                4005,
                "Native parameter inference expression depth exceeded",
            ));
        }
        match expr {
            IrExpr::Name(Name::User(name)) => {
                if let Some(required) = required {
                    self.constrain_name(name, required)?;
                }
            }
            IrExpr::Unary { op, value } => {
                let expected = match op.as_str() {
                    "not" => Some(Type::Bool),
                    "+" | "-" => Some(Type::Int),
                    _ => None,
                };
                self.expression(value, expected, depth + 1)?;
            }
            IrExpr::Binary { op, lhs, rhs } => {
                let expected = match op.as_str() {
                    "and" | "or" => Some(Type::Bool),
                    "+" | "-" | "*" | "/" | "//" | "%" | "<" | "<=" | ">" | ">=" => Some(Type::Int),
                    _ => None,
                };
                if op == "+" {
                    // Addition is shared by integers and UTF-8 strings.
                    // Text evidence overrides the old numeric-only inference.
                    let named_text = |expr: &IrExpr| match expr {
                        IrExpr::Name(Name::User(name)) => self
                            .parameters
                            .iter()
                            .position(|parameter| parameter == name)
                            .is_some_and(|idx| self.inferred[idx] == Some(Type::Text)),
                        _ => false,
                    };
                    let text_context = required == Some(Type::Text)
                        || matches!(lhs.as_ref(), IrExpr::String(_))
                        || matches!(rhs.as_ref(), IrExpr::String(_))
                        || named_text(lhs)
                        || named_text(rhs);
                    let kind = if text_context { Type::Text } else { Type::Int };
                    self.expression(lhs, Some(kind), depth + 1)?;
                    self.expression(rhs, Some(kind), depth + 1)?;
                } else if matches!(op.as_str(), "==" | "!=") {
                    // A boolean literal provides a reliable type constraint
                    // on the other operand without requiring whole-program
                    // flow-sensitive type inference.
                    let left = if matches!(rhs.as_ref(), IrExpr::Boolean(_)) {
                        Some(Type::Bool)
                    } else if matches!(rhs.as_ref(), IrExpr::Number(_)) {
                        Some(Type::Int)
                    } else {
                        None
                    };
                    let right = if matches!(lhs.as_ref(), IrExpr::Boolean(_)) {
                        Some(Type::Bool)
                    } else if matches!(lhs.as_ref(), IrExpr::Number(_)) {
                        Some(Type::Int)
                    } else {
                        None
                    };
                    self.expression(lhs, left, depth + 1)?;
                    self.expression(rhs, right, depth + 1)?;
                } else {
                    self.expression(lhs, expected, depth + 1)?;
                    self.expression(rhs, expected, depth + 1)?;
                }
            }
            IrExpr::Call { callee, args } => {
                let known = match callee.as_ref() {
                    IrExpr::Name(Name::User(name)) => self.functions.get(name).copied(),
                    _ => None,
                };
                if let Some(function) = known {
                    for (index, argument) in args.iter().enumerate() {
                        self.expression(
                            argument,
                            function.parameters.get(index).copied(),
                            depth + 1,
                        )?;
                    }
                } else {
                    let is_range = matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("range")));
                    for argument in args {
                        self.expression(
                            argument,
                            if is_range { Some(Type::Int) } else { None },
                            depth + 1,
                        )?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn block(&mut self, body: &[IrStatement], depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(issue(
                4005,
                "Native parameter inference block depth exceeded",
            ));
        }
        self.scopes.push(BTreeSet::new());
        let result = body
            .iter()
            .try_for_each(|stmt| self.statement(stmt, depth + 1));
        self.scopes.pop();
        result
    }

    fn statement(&mut self, stmt: &IrStatement, depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(issue(
                4005,
                "Native parameter inference statement depth exceeded",
            ));
        }
        match stmt {
            IrStatement::Let { name, expr, .. } => {
                self.expression(expr, None, depth + 1)?;
                self.scopes
                    .last_mut()
                    .expect("inference lexical scope")
                    .insert(name.clone());
            }
            IrStatement::Assign { name, expr } => {
                self.expression(expr, None, depth + 1)?;
                if matches!(expr, IrExpr::Boolean(_)) {
                    self.constrain_name(name, Type::Bool)?;
                } else if matches!(expr, IrExpr::Number(_)) {
                    self.constrain_name(name, Type::Int)?;
                }
            }
            IrStatement::If {
                branches,
                otherwise,
            } => {
                for (condition, body) in branches {
                    self.expression(condition, Some(Type::Bool), depth + 1)?;
                    self.block(body, depth + 1)?;
                }
                if let Some(body) = otherwise {
                    self.block(body, depth + 1)?;
                }
            }
            IrStatement::While { condition, body } => {
                self.expression(condition, Some(Type::Bool), depth + 1)?;
                self.block(body, depth + 1)?;
            }
            IrStatement::For {
                variable,
                iterable,
                body,
            } => {
                self.expression(iterable, None, depth + 1)?;
                self.scopes.push(BTreeSet::from([variable.clone()]));
                let result = self.block(body, depth + 1);
                self.scopes.pop();
                result?;
            }
            IrStatement::Return(value) | IrStatement::Expression(value) => {
                self.expression(value, None, depth + 1)?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn explicit_native_type(name: &str) -> Result<Type, Diagnostic> {
    match name {
        "int" => Ok(Type::Int),
        "bool" => Ok(Type::Bool),
        "str" => Ok(Type::Text),
        _ => Err(unsupported(format!(
            "The native x86-64 function signature currently supports int, bool and str, not {name:?}"
        ))),
    }
}

fn infer_native_parameters(
    parameters: &[String],
    annotations: &[Option<String>],
    body: &[IrStatement],
    functions: &BTreeMap<String, FunctionInfo>,
) -> Result<[Type; 32], Diagnostic> {
    if parameters.len() > 32 {
        return Err(issue(
            4005,
            "Too many native function parameters for inference",
        ));
    }
    if parameters.len() != annotations.len() {
        return Err(unsupported(
            "Parameter annotation count does not match function parameters",
        ));
    }
    let mut annotated = [None; 32];
    for (index, ty) in annotations.iter().enumerate() {
        if let Some(ty) = ty {
            annotated[index] = Some(explicit_native_type(ty)?);
        }
    }
    let mut constraints = ParameterConstraints {
        parameters,
        inferred: annotated,
        scopes: vec![BTreeSet::new()],
        functions,
    };
    for statement in body {
        constraints.statement(statement, 0)?;
    }
    let mut kinds = [Type::Int; 32];
    for (index, inferred) in constraints.inferred.iter().enumerate() {
        if let Some(value) = inferred {
            kinds[index] = *value;
        }
    }
    Ok(kinds)
}

impl NativeCompiler {
    fn emit(&mut self, line: &str) -> Result<(), Diagnostic> {
        if self.asm.len().saturating_add(line.len()).saturating_add(1) > MAX_ASM_BYTES {
            return Err(issue(4005, "Generated assembler exceeds the 1 MiB budget"));
        }
        self.asm.push_str(line);
        self.asm.push('\n');
        Ok(())
    }

    fn emit_static_text(&mut self, value: &str) -> Result<(), Diagnostic> {
        if value.len() > MAX_ASM_BYTES / 4 {
            return Err(issue(
                4005,
                "Native text literal exceeds static data budget",
            ));
        }
        let label = self.new_label();
        self.static_texts.push((label, value.as_bytes().to_vec()));
        self.emit(&format!("    lea rax, [rip+.L{label}]"))?;
        self.emit(&format!("    mov edx, {}", value.len()))
    }

    fn new_label(&mut self) -> usize {
        let id = self.labels;
        self.labels += 1;
        id
    }

    fn label(&mut self, id: usize) -> Result<(), Diagnostic> {
        self.emit(&format!(".L{id}:"))
    }

    fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn allocate_slot(&mut self) -> Result<usize, Diagnostic> {
        if self.slots >= MAX_SLOTS {
            return Err(issue(4005, "Native variable slot count exceeds limit"));
        }
        let slot = self.slots;
        self.slots += 1;
        Ok(slot)
    }

    fn tick(&mut self) -> Result<(), Diagnostic> {
        self.emit("    dec r15")?;
        self.emit("    jz .Lnobody_budget")
    }

    fn expression(&mut self, expr: &IrExpr, depth: usize) -> Result<Type, Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(issue(4005, "Native expression depth exceeded"));
        }
        match expr {
            IrExpr::Number(number) => {
                let normalized = number.replace('_', "");
                if normalized.is_empty() || !normalized.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(unsupported("Native backend supports only signed i64 integer literals"));
                }
                let value = normalized.parse::<i64>().map_err(|_| {
                    unsupported("Native backend integer literal is outside the signed i64 range")
                })?;
                self.emit(&format!("    mov rax, {value}"))?;
                Ok(Type::Int)
            }
            IrExpr::Call { callee, args }
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("len"))) =>
            {
                if args.len() != 1 {
                    return Err(issue(
                        4007,
                        format!("len expects one UTF-8 string argument, received {}", args.len()),
                    ));
                }
                if self.expression(&args[0], depth + 1)? != Type::Text {
                    return Err(unsupported("Native len requires a text argument"));
                }
                self.emit("    call nobody_text_length")?;
                Ok(Type::Int)
            }
            IrExpr::Call { callee, args } => {
                let IrExpr::Name(Name::User(name)) = callee.as_ref() else {
                    return Err(unsupported("Native calls require a previously declared named function"));
                };
                if self.lookup(name).is_some() {
                    return Err(unsupported("Variable values are not native callable functions"));
                }
                let function = self.functions.get(name).copied().ok_or_else(|| {
                    issue(4001, format!("Unknown or forward native function {name:?}"))
                })?;
                if args.len() != function.arity {
                    return Err(issue(4007, format!("Function {name:?} expects {} argument(s), received {}", function.arity, args.len())));
                }
                let mut pushed_words = 0usize;
                for (argument, expected) in args.iter().zip(function.parameters.iter()) {
                    if self.expression(argument, depth + 1)? != *expected {
                        return Err(unsupported("Native function argument type does not match its inferred parameter type"));
                    }
                    self.emit("    push rax")?;
                    pushed_words += 1;
                    if *expected == Type::Text {
                        // Text arguments carry immutable pointer + byte length.
                        self.emit("    push rdx")?;
                        pushed_words += 1;
                    }
                }
                self.emit(&format!("    call .L{}", function.label))?;
                if pushed_words > 0 {
                    self.emit(&format!("    add rsp, {}", pushed_words * 8))?;
                }
                Ok(function.result)
            }
            IrExpr::Boolean(value) => {
                self.emit(if *value { "    mov eax, 1" } else { "    xor eax, eax" })?;
                Ok(Type::Bool)
            }
            IrExpr::String(raw) => {
                let decoded = crate::evaluator::literal_text(raw)?;
                self.emit_static_text(&decoded)?;
                Ok(Type::Text)
            }
            IrExpr::Name(Name::User(name)) => {
                let binding = self.lookup(name).ok_or_else(|| issue(4001, format!("Unknown native variable {name:?}")))?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (binding.slot + 1) * 8))?;
                if binding.kind == Type::Text {
                    self.emit(&format!("    mov rdx, QWORD PTR [rbp-{}]", (binding.slot + 2) * 8))?;
                }
                Ok(binding.kind)
            }
            IrExpr::Unary { op, value } => {
                let ty = self.expression(value, depth + 1)?;
                match op.as_str() {
                    "+" if ty == Type::Int => Ok(Type::Int),
                    "-" if ty == Type::Int => {
                        self.emit("    neg rax")?;
                        self.emit("    jo .Lnobody_overflow")?;
                        Ok(Type::Int)
                    }
                    "not" if ty == Type::Bool => {
                        self.emit("    test rax, rax")?;
                        self.emit("    sete al")?;
                        self.emit("    movzx rax, al")?;
                        Ok(Type::Bool)
                    }
                    _ => Err(unsupported("Unsupported native unary operator or operand type")),
                }
            }
            IrExpr::Binary { op, lhs, rhs } => {
                if op == "and" || op == "or" {
                    if self.expression(lhs, depth + 1)? != Type::Bool {
                        return Err(unsupported("Native logical operators require boolean operands"));
                    }
                    let skip = self.new_label();
                    self.emit("    test rax, rax")?;
                    self.emit(&format!("    {} .L{skip}", if op == "and" { "jz" } else { "jnz" }))?;
                    if self.expression(rhs, depth + 1)? != Type::Bool {
                        return Err(unsupported("Native logical operators require boolean operands"));
                    }
                    self.label(skip)?;
                    return Ok(Type::Bool);
                }
                let left = self.expression(lhs, depth + 1)?;
                self.emit("    push rax")?;
                if left == Type::Text {
                    // Preserve the UTF-8 byte length across RHS evaluation.
                    self.emit("    push rdx")?;
                }
                let right = self.expression(rhs, depth + 1)?;
                if left == Type::Text {
                    self.emit("    pop r8")?;
                }
                self.emit("    pop rcx")?;
                let numeric = left == Type::Int && right == Type::Int;
                if left == Type::Text && right == Type::Text {
                    if !matches!(op.as_str(), "+" | "==" | "!=") {
                        return Err(unsupported(
                            "Native text values support only concatenation and equality",
                        ));
                    }
                    // Inputs: left ptr=rcx, len=r8, right ptr=rax, len=rdx.
                    // Helpers return a text pointer/length pair or boolean.
                    self.emit("    mov r9, rdx")?;
                    self.emit("    mov rdx, rax")?;
                    self.emit("    mov rdi, rcx")?;
                    self.emit("    mov rsi, r8")?;
                    self.emit("    mov rcx, r9")?;
                    if op == "+" {
                        self.emit("    call nobody_text_concat")?;
                        return Ok(Type::Text);
                    }
                    self.emit("    call nobody_text_equal")?;
                    if op == "!=" {
                        self.emit("    xor eax, 1")?;
                    }
                    return Ok(Type::Bool);
                }
                match op.as_str() {
                    "+" | "-" | "*" if numeric => {
                        match op.as_str() {
                            "+" => self.emit("    add rax, rcx")?,
                            "-" => {
                                self.emit("    sub rcx, rax")?;
                                self.emit("    mov rax, rcx")?;
                            }
                            _ => self.emit("    imul rax, rcx")?,
                        }
                        self.emit("    jo .Lnobody_overflow")?;
                        Ok(Type::Int)
                    }
                    "==" | "!=" if left == right && left != Type::Text => {
                        self.emit("    cmp rcx, rax")?;
                        self.emit(if op == "==" { "    sete al" } else { "    setne al" })?;
                        self.emit("    movzx rax, al")?;
                        Ok(Type::Bool)
                    }
                    "<" | "<=" | ">" | ">=" if numeric => {
                        self.emit("    cmp rcx, rax")?;
                        let opcode = match op.as_str() {
                            "<" => "    setl al",
                            "<=" => "    setle al",
                            ">" => "    setg al",
                            _ => "    setge al",
                        };
                        self.emit(opcode)?;
                        self.emit("    movzx rax, al")?;
                        Ok(Type::Bool)
                    }
                    _ => Err(unsupported("Native expression operator or operand types are unsupported")),
                }
            }
            _ => Err(unsupported("Native backend does not yet compile strings, null, first-class values or other complex expressions")),
        }
    }

    // Deliberately conservative: this initial native backend only accepts
    // functions whose every possible fallthrough path returns an i64 value.
    // An implicit null return remains available in the bytecode VM instead.
    fn definitely_returns(body: &[IrStatement]) -> bool {
        for statement in body {
            match statement {
                IrStatement::Return(_) => return true,
                IrStatement::If {
                    branches,
                    otherwise: Some(otherwise),
                } if !branches.is_empty()
                    && branches
                        .iter()
                        .all(|(_, body)| Self::definitely_returns(body))
                    && Self::definitely_returns(otherwise) =>
                {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    fn block(&mut self, statements: &[IrStatement]) -> Result<(), Diagnostic> {
        self.scopes.push(BTreeMap::new());
        let result = statements
            .iter()
            .try_for_each(|statement| self.statement(statement));
        self.scopes.pop();
        result
    }

    fn statement(&mut self, stmt: &IrStatement) -> Result<(), Diagnostic> {
        self.tick()?;
        match stmt {
            IrStatement::Let { name, type_name: None, expr } => {
                if self.scopes.last().is_some_and(|scope| scope.contains_key(name)) {
                    return Err(unsupported(format!("Duplicate native variable declaration {name:?}")));
                }
                let kind = self.expression(expr, 0)?;
                let slot = self.allocate_slot()?;
                if kind == Type::Text {
                    // Text bindings consume two adjacent stack words.
                    self.allocate_slot()?;
                }
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (slot + 1) * 8))?;
                if kind == Type::Text {
                    self.emit(&format!("    mov QWORD PTR [rbp-{}], rdx", (slot + 2) * 8))?;
                }
                self.scopes.last_mut().expect("root or nested lexical scope exists")
                    .insert(name.clone(), Binding { slot, kind });
                Ok(())
            }
            IrStatement::Assign { name, expr } => {
                let binding = self.lookup(name).ok_or_else(|| issue(4001, format!("Unknown assignment target {name:?}")))?;
                let ty = self.expression(expr, 0)?;
                if ty != binding.kind {
                    return Err(unsupported("Native backend does not support changing a variable's value type"));
                }
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (binding.slot + 1) * 8))?;
                if ty == Type::Text {
                    self.emit(&format!("    mov QWORD PTR [rbp-{}], rdx", (binding.slot + 2) * 8))?;
                }
                Ok(())
            }
            IrStatement::Expression(IrExpr::Call { callee, args })
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("print"))) =>
            {
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        self.emit("    lea rsi, [rip+.Lnobody_space]")?;
                        self.emit("    mov edx, 1")?;
                        self.emit("    call nobody_write_raw")?;
                    }
                    let kind = self.expression(arg, 0)?;
                    match kind {
                        Type::Int => {
                            self.emit("    mov rdi, rax")?;
                            self.emit("    call nobody_print_i64")?;
                        }
                        Type::Bool => {
                            self.emit("    mov rdi, rax")?;
                            self.emit("    call nobody_print_bool")?;
                        }
                        Type::Text => {
                            self.emit("    mov rsi, rax")?;
                            self.emit("    call nobody_write_raw")?;
                        }
                    }
                }
                self.emit("    lea rsi, [rip+.Lnobody_newline]")?;
                self.emit("    mov edx, 1")?;
                self.emit("    call nobody_write_raw")
            }
            IrStatement::Expression(expression) => {
                self.expression(expression, 0)?;
                Ok(())
            }
            IrStatement::If { branches, otherwise } => {
                let end = self.new_label();
                for (cond, body) in branches {
                    if self.expression(cond, 0)? != Type::Bool {
                        return Err(unsupported("Native if conditions must be boolean"));
                    }
                    let next = self.new_label();
                    self.emit("    test rax, rax")?;
                    self.emit(&format!("    jz .L{next}"))?;
                    self.block(body)?;
                    self.emit(&format!("    jmp .L{end}"))?;
                    self.label(next)?;
                }
                if let Some(body) = otherwise {
                    self.block(body)?;
                }
                self.label(end)
            }
            IrStatement::While { condition, body } => {
                let start = self.new_label();
                let end = self.new_label();
                self.label(start)?;
                self.tick()?; // an empty infinite loop also consumes its budget
                if self.expression(condition, 0)? != Type::Bool {
                    return Err(unsupported("Native while conditions must be boolean"));
                }
                self.emit("    test rax, rax")?;
                self.emit(&format!("    jz .L{end}"))?;
                self.loops.push(LoopTargets { start, end });
                let result = self.block(body);
                self.loops.pop();
                result?;
                self.emit(&format!("    jmp .L{start}"))?;
                self.label(end)
            }
            IrStatement::For {
                variable,
                iterable,
                body,
            } => {
                // The native backend supports precisely the canonical range
                // built-in, not arbitrary iterables or shadowed user calls.
                let IrExpr::Call { callee, args } = iterable else {
                    return Err(unsupported("Native for loops currently require range(...)"));
                };
                if !matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("range"))) {
                    return Err(unsupported("Native for loops require unshadowed range(...)"));
                }
                if !(1..=3).contains(&args.len()) {
                    return Err(unsupported("Native range requires one, two or three integer arguments"));
                }

                // Evaluate all range arguments once, left-to-right, before the
                // loop variable is introduced into its own lexical scope.
                let mut inputs = Vec::with_capacity(args.len());
                for arg in args {
                    if self.expression(arg, 0)? != Type::Int {
                        return Err(unsupported("Native range arguments must be integers"));
                    }
                    let slot = self.allocate_slot()?;
                    self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (slot + 1) * 8))?;
                    inputs.push(slot);
                }
                let current = self.allocate_slot()?;
                let stop = self.allocate_slot()?;
                let step = self.allocate_slot()?;
                match inputs.as_slice() {
                    [limit] => {
                        self.emit("    xor eax, eax")?;
                        self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (current + 1) * 8))?;
                        self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (limit + 1) * 8))?;
                        self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (stop + 1) * 8))?;
                    }
                    [start, limit] | [start, limit, _] => {
                        self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (start + 1) * 8))?;
                        self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (current + 1) * 8))?;
                        self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (limit + 1) * 8))?;
                        self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (stop + 1) * 8))?;
                    }
                    _ => unreachable!("native range argument count checked"),
                }
                if let [_, _, stride] = inputs.as_slice() {
                    self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (stride + 1) * 8))?;
                } else {
                    self.emit("    mov eax, 1")?;
                }
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (step + 1) * 8))?;

                // Like the Rust evaluator, zero step is an error even when
                // the requested range would otherwise be empty.
                self.emit("    test rax, rax")?;
                self.emit("    jz .Lnobody_range_zero_step")?;

                let check = self.new_label();
                let negative = self.new_label();
                let execute_body = self.new_label();
                let advance = self.new_label();
                let end = self.new_label();

                self.scopes.push(BTreeMap::new());
                let iterator = self.allocate_slot()?;
                self.scopes.last_mut().expect("native range scope exists")
                    .insert(variable.clone(), Binding { slot: iterator, kind: Type::Int });

                self.label(check)?;
                self.tick()?; // empty and very long loops remain bounded
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (step + 1) * 8))?;
                self.emit("    test rax, rax")?;
                self.emit(&format!("    js .L{negative}"))?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (current + 1) * 8))?;
                self.emit(&format!("    cmp rax, QWORD PTR [rbp-{}]", (stop + 1) * 8))?;
                self.emit(&format!("    jge .L{end}"))?;
                self.emit(&format!("    jmp .L{execute_body}"))?;
                self.label(negative)?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (current + 1) * 8))?;
                self.emit(&format!("    cmp rax, QWORD PTR [rbp-{}]", (stop + 1) * 8))?;
                self.emit(&format!("    jle .L{end}"))?;

                self.label(execute_body)?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (current + 1) * 8))?;
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (iterator + 1) * 8))?;
                self.loops.push(LoopTargets { start: advance, end });
                let result = self.block(body);
                self.loops.pop();
                self.scopes.pop();
                result?;
                self.label(advance)?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (current + 1) * 8))?;
                self.emit(&format!("    add rax, QWORD PTR [rbp-{}]", (step + 1) * 8))?;
                // A final range-iteration overflow TERMINATES, rather than
                // wrapping or using the arithmetic-overflow process exit.
                self.emit(&format!("    jo .L{end}"))?;
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (current + 1) * 8))?;
                self.emit(&format!("    jmp .L{check}"))?;
                self.label(end)
            }
            IrStatement::Function {
                name,
                parameters,
                parameter_types: annotations,
                return_type,
                body,
            } => {
                if self.in_function || self.scopes.len() != 1 {
                    return Err(unsupported("Native nested function definitions and closures are unsupported"));
                }
                if self.functions.contains_key(name) || self.lookup(name).is_some() {
                    return Err(unsupported(format!("Duplicate native function declaration {name:?}")));
                }
                if self.functions.len() >= 32 {
                    return Err(issue(4005, "The native backend supports at most 32 functions"));
                }
                if parameters.len() > 32 {
                    return Err(issue(4005, "Native functions support at most 32 parameters"));
                }
                if !Self::definitely_returns(body) {
                    return Err(unsupported("Native functions must explicitly return a value on every path; implicit null is unsupported"));
                }
                let parameter_types = infer_native_parameters(parameters, annotations, body, &self.functions)?;
                let declared_result = return_type.as_deref().map(explicit_native_type).transpose()?;
                let mut local_scope = BTreeMap::new();
                let mut parameter_words = 0usize;
                for (index, parameter) in parameters.iter().enumerate() {
                    let kind = parameter_types[index];
                    if local_scope.insert(
                        parameter.clone(),
                        Binding { slot: parameter_words, kind },
                    ).is_some() {
                        return Err(unsupported("Duplicate native function parameter"));
                    }
                    parameter_words += if kind == Type::Text { 2 } else { 1 };
                }
                if parameter_words > MAX_SLOTS {
                    return Err(issue(4005, "Native function parameter storage exceeds stack budget"));
                }
                let label = self.new_label();
                let skip = self.new_label();
                self.emit(&format!("    jmp .L{skip}"))?;
                self.label(label)?;
                self.emit("    push rbp")?;
                self.emit("    mov rbp, rsp")?;
                let prologue = self.asm.len();

                // Independent call frames: no caller local state or global
                // bindings are visible inside a function. Arguments live
                // above the saved rbp/return address on the caller stack.
                let saved_scopes = std::mem::replace(&mut self.scopes, vec![local_scope]);
                let saved_slots = std::mem::replace(&mut self.slots, parameter_words);
                let saved_loops = std::mem::take(&mut self.loops);
                let saved_function = std::mem::replace(&mut self.in_function, true);
                let saved_return = self.return_kind.take();

                let mut first_word = 0usize;
                for kind in parameter_types.iter().take(parameters.len()) {
                    let width = if *kind == Type::Text { 2 } else { 1 };
                    let destination = first_word;
                    for word in 0..width {
                        // Last-pushed argument word is closest to rbp+16.
                        let offset = 16 + (parameter_words - 1 - first_word - word) * 8;
                        self.emit(&format!("    mov rax, QWORD PTR [rbp+{offset}]"))?;
                        self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (destination + word + 1) * 8))?;
                    }
                    first_word += width;
                }
                for statement in body {
                    self.statement(statement)?;
                }
                // Every path must contain a signed integer or boolean return.
                let result_kind = self.return_kind.ok_or_else(|| unsupported(
                    "Native functions require an explicit int, bool or str return value"
                ))?;
                if declared_result.is_some_and(|kind| kind != result_kind) {
                    return Err(unsupported(format!(
                        "Native function {name:?} returns a value incompatible with its explicit return type"
                    )));
                }
                // Defensive fallback if a future codegen change violates the
                // conservative all-paths-return static check.
                self.emit("    mov edi, 7")?;
                self.emit("    mov eax, 60")?;
                self.emit("    syscall")?;

                let used = self.slots * 8;
                let reserve = used.div_ceil(16) * 16;
                self.asm.insert_str(prologue, &format!("    sub rsp, {reserve}\n"));
                self.scopes = saved_scopes;
                self.slots = saved_slots;
                self.loops = saved_loops;
                self.in_function = saved_function;
                self.return_kind = saved_return;
                self.label(skip)?;
                // Adding only after compilation disallows direct/indirect
                // recursion and forward references in this first subset.
                self.functions.insert(name.clone(), FunctionInfo {
                    label, arity: parameters.len(), parameters: parameter_types, result: result_kind
                });
                Ok(())
            }
            IrStatement::Return(expr) => {
                if !self.in_function {
                    return Err(unsupported("Native return requires a function body"));
                }
                let ty = self.expression(expr, 0)?;
                if self.return_kind.is_some_and(|kind| kind != ty) {
                    return Err(unsupported("All native function returns must have the same int, bool or str type"));
                }
                self.return_kind = Some(ty);
                self.emit("    leave")?;
                self.emit("    ret")
            }
            IrStatement::Break => {
                let target = self.loops.last().ok_or_else(|| unsupported("break outside native loop"))?.end;
                self.emit(&format!("    jmp .L{target}"))
            }
            IrStatement::Continue => {
                let target = self.loops.last().ok_or_else(|| unsupported("continue outside native loop"))?.start;
                self.emit(&format!("    jmp .L{target}"))
            }
            _ => Err(unsupported("Native x86-64 subset does not yet support typed declarations or other complex statements")),
        }
    }
}

/// Produce GNU Intel-syntax x86-64 assembler directly from canonical Nobody IR.
/// Only internally generated identifiers enter the output: user-provided names,
/// strings, file paths and comments are never interpolated as assembler text.
pub fn generate_assembly(module: &Module) -> Result<String, Diagnostic> {
    let mut compiler = NativeCompiler {
        asm: String::new(),
        scopes: vec![BTreeMap::new()],
        slots: 0,
        labels: 0,
        loops: Vec::new(),
        functions: BTreeMap::new(),
        in_function: false,
        return_kind: None,
        static_texts: Vec::new(),
    };
    compiler.emit(".intel_syntax noprefix")?;
    compiler.emit(".global _start")?;
    compiler.emit(".section .text")?;
    compiler.emit("_start:")?;
    compiler.emit("    mov rbp, rsp")?;
    compiler.emit(&format!("    mov r15, {RUNTIME_STEPS}"))?;
    let prologue = compiler.asm.len();
    for statement in &module.statements {
        compiler.statement(statement)?;
    }
    compiler.emit("    xor edi, edi")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit(".Lnobody_overflow:")?;
    compiler.emit("    mov edi, 3")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit(".Lnobody_range_zero_step:")?;
    compiler.emit("    mov edi, 6")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit(".Lnobody_text_budget:")?;
    compiler.emit("    mov edi, 7")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit(".Lnobody_budget:")?;
    compiler.emit("    mov edi, 5")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit("nobody_print_i64:")?;
    // The native output does not use libc or the bytecode interpreter.
    // Unsigned division handles i64::MIN after two's-complement negation.
    compiler.emit("    sub rsp, 64")?;
    compiler.emit("    lea rsi, [rsp+64]")?;
    compiler.emit("    xor r10d, r10d")?;
    compiler.emit("    mov rax, rdi")?;
    compiler.emit("    xor r9d, r9d")?;
    compiler.emit("    test rax, rax")?;
    compiler.emit("    jns .Lnobody_digits")?;
    compiler.emit("    mov r9d, 1")?;
    compiler.emit("    neg rax")?;
    compiler.emit(".Lnobody_digits:")?;
    compiler.emit("    mov r8, 10")?;
    compiler.emit(".Lnobody_digit_loop:")?;
    compiler.emit("    xor edx, edx")?;
    compiler.emit("    div r8")?;
    compiler.emit("    add dl, 48")?;
    compiler.emit("    dec rsi")?;
    compiler.emit("    mov BYTE PTR [rsi], dl")?;
    compiler.emit("    inc r10")?;
    compiler.emit("    test rax, rax")?;
    compiler.emit("    jnz .Lnobody_digit_loop")?;
    compiler.emit("    test r9d, r9d")?;
    compiler.emit("    jz .Lnobody_write")?;
    compiler.emit("    dec rsi")?;
    compiler.emit("    mov BYTE PTR [rsi], 45")?;
    compiler.emit("    inc r10")?;
    compiler.emit(".Lnobody_write:")?;
    compiler.emit("    mov eax, 1")?;
    compiler.emit("    mov edi, 1")?;
    compiler.emit("    mov rdx, r10")?;
    compiler.emit("    syscall")?;
    compiler.emit("    add rsp, 64")?;
    compiler.emit("    ret")?;
    compiler.emit("nobody_print_bool:")?;
    compiler.emit("    test rdi, rdi")?;
    compiler.emit("    jz .Lnobody_bool_false")?;
    compiler.emit("    lea rsi, [rip+.Lnobody_bool_true_text]")?;
    compiler.emit("    mov edx, 4")?;
    compiler.emit("    jmp .Lnobody_bool_write")?;
    compiler.emit(".Lnobody_bool_false:")?;
    compiler.emit("    lea rsi, [rip+.Lnobody_bool_false_text]")?;
    compiler.emit("    mov edx, 5")?;
    compiler.emit(".Lnobody_bool_write:")?;
    compiler.emit("    mov edi, 1")?;
    compiler.emit("    mov eax, 1")?;
    compiler.emit("    syscall")?;
    compiler.emit("    ret")?;
    // Raw writes are shared by text literals, separators and newlines.
    // Helpers use a bounded, fixed-size UTF-8 arena in the output ELF.
    // No libc, Rust allocator, or Nobody VM is required at runtime.
    // Count UTF-8 Unicode scalar values: non-continuation bytes start a scalar.
    // Validity of the UTF-8 input is established by the lexer, evaluator
    // decoding and bounded concat routines; this is not grapheme-cluster count.
    compiler.emit("nobody_text_length:")?;
    compiler.emit("    mov rsi, rax")?;
    compiler.emit("    mov rcx, rdx")?;
    compiler.emit("    xor eax, eax")?;
    compiler.emit(".Lnobody_text_length_loop:")?;
    compiler.emit("    test rcx, rcx")?;
    compiler.emit("    jz .Lnobody_text_length_done")?;
    compiler.emit("    movzx r8d, BYTE PTR [rsi]")?;
    compiler.emit("    and r8d, 192")?;
    compiler.emit("    cmp r8d, 128")?;
    compiler.emit("    sete r8b")?;
    compiler.emit("    xor r8b, 1")?;
    compiler.emit("    movzx r8d, r8b")?;
    compiler.emit("    add rax, r8")?;
    compiler.emit("    inc rsi")?;
    compiler.emit("    dec rcx")?;
    compiler.emit("    jmp .Lnobody_text_length_loop")?;
    compiler.emit(".Lnobody_text_length_done:")?;
    compiler.emit("    ret")?;
    compiler.emit("nobody_text_equal:")?;
    compiler.emit("    cmp rsi, rcx")?;
    compiler.emit("    jne .Lnobody_text_not_equal")?;
    compiler.emit("    mov r8, rdx")?;
    compiler.emit("    mov rdx, rsi")?;
    compiler.emit("    mov rsi, rdi")?;
    compiler.emit("    mov rdi, r8")?;
    compiler.emit("    cld")?;
    compiler.emit("    repe cmpsb")?;
    compiler.emit("    sete al")?;
    compiler.emit("    movzx rax, al")?;
    compiler.emit("    ret")?;
    compiler.emit(".Lnobody_text_not_equal:")?;
    compiler.emit("    xor eax, eax")?;
    compiler.emit("    ret")?;
    compiler.emit("nobody_text_concat:")?;
    // (rdi: left ptr, rsi: left bytes, rdx: right ptr, rcx: right bytes)
    // returns rax: joined ptr, rdx: joined byte length.
    compiler.emit("    mov r8, rsi")?;
    compiler.emit("    add r8, rcx")?;
    compiler.emit("    jc .Lnobody_text_budget")?;
    compiler.emit("    cmp r8, 1048576")?;
    compiler.emit("    ja .Lnobody_text_budget")?;
    compiler.emit("    mov r9, QWORD PTR [rip+.Lnobody_text_used]")?;
    compiler.emit("    mov r10, r9")?;
    compiler.emit("    add r10, r8")?;
    compiler.emit("    jc .Lnobody_text_budget")?;
    compiler.emit("    cmp r10, 1048576")?;
    compiler.emit("    ja .Lnobody_text_budget")?;
    compiler.emit("    mov QWORD PTR [rip+.Lnobody_text_used], r10")?;
    compiler.emit("    lea r11, [rip+.Lnobody_text_arena]")?;
    compiler.emit("    add r11, r9")?;
    compiler.emit("    push rdx")?;
    compiler.emit("    push rcx")?;
    compiler.emit("    mov rcx, rsi")?;
    compiler.emit("    mov rsi, rdi")?;
    compiler.emit("    mov rdi, r11")?;
    compiler.emit("    cld")?;
    compiler.emit("    rep movsb")?;
    compiler.emit("    pop rcx")?;
    compiler.emit("    pop rsi")?;
    compiler.emit("    rep movsb")?;
    compiler.emit("    mov rax, r11")?;
    compiler.emit("    mov rdx, r8")?;
    compiler.emit("    ret")?;
    compiler.emit("nobody_write_raw:")?;
    compiler.emit("    mov eax, 1")?;
    compiler.emit("    mov edi, 1")?;
    compiler.emit("    syscall")?;
    compiler.emit("    ret")?;
    compiler.emit(".section .rodata")?;
    compiler.emit(".Lnobody_bool_true_text:")?;
    compiler.emit("    .ascii \"true\"")?;
    compiler.emit(".Lnobody_bool_false_text:")?;
    compiler.emit("    .ascii \"false\"")?;
    compiler.emit(".Lnobody_space:")?;
    compiler.emit("    .byte 32")?;
    compiler.emit(".Lnobody_newline:")?;
    compiler.emit("    .byte 10")?;
    for (label, bytes) in std::mem::take(&mut compiler.static_texts) {
        compiler.emit(&format!(".L{label}:"))?;
        if bytes.is_empty() {
            compiler.emit("    .byte 0")?;
        } else {
            // A string containing '.global' stays plain data, never assembly.
            let sequence = bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            compiler.emit(&format!("    .byte {sequence}"))?;
        }
    }
    compiler.emit(".section .bss")?;
    compiler.emit("    .balign 8")?;
    compiler.emit(".Lnobody_text_used:")?;
    compiler.emit("    .zero 8")?;
    compiler.emit(".Lnobody_text_arena:")?;
    compiler.emit("    .zero 1048576")?;
    compiler.emit(".section .note.GNU-stack,\"\",@progbits")?;

    // Allocate stack storage before the first statement, but after frame setup.
    // Round to 16 bytes so call-site stack alignment stays stable.
    let used = compiler.slots * 8;
    let reserve = used.div_ceil(16) * 16;
    let mut result = compiler.asm;
    result.insert_str(prologue, &format!("    sub rsp, {reserve}\n"));
    if result.len() > MAX_ASM_BYTES {
        return Err(issue(4005, "Native assembly exceeds generated-code budget"));
    }
    Ok(result)
}

pub fn compile_to_assembly(source: &str, profile: &str) -> Result<String, Diagnostic> {
    if source.len() > MAX_ASM_BYTES {
        return Err(issue(4005, "Native source exceeds the 1 MiB budget"));
    }
    generate_assembly(&lower_source(source, profile)?)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
struct TemporaryBuild(std::path::PathBuf);
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
impl Drop for TemporaryBuild {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn run_tool(name: &str, args: &[&std::ffi::OsStr]) -> Result<(), Diagnostic> {
    use std::process::Command;
    let output = Command::new(name).args(args).output().map_err(|error| {
        issue(
            9001,
            format!("Cannot launch required native tool {name:?}: {error}"),
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.chars().take(1500).collect::<String>();
        return Err(issue(
            4004,
            format!("Native tool {name:?} failed: {detail}"),
        ));
    }
    Ok(())
}

/// Compile a source file to a real Linux ELF64/x86-64 executable by invoking
/// GNU assembler and linker directly, without a shell or C runtime.
/// The target is atomically created with a hard-link only after linking passes;
/// an existing file (including symlink) is never overwritten.
pub fn compile_native(source: &str, profile: &str, target: &Path) -> Result<(), Diagnostic> {
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = (source, profile, target);
        Err(unsupported(
            "The first native backend only runs on Linux x86-64",
        ))
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        use std::ffi::OsStr;
        use std::os::unix::fs::DirBuilderExt;
        let assembler = compile_to_assembly(source, profile)?;
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut directory = None;
        for attempt in 0..128 {
            let candidate = parent.join(format!(".nobody-native-{}-{attempt}", std::process::id()));
            match std::fs::DirBuilder::new().mode(0o700).create(&candidate) {
                Ok(()) => {
                    directory = Some(candidate);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(issue(
                        9001,
                        format!("Cannot create temporary native-build directory: {error}"),
                    ))
                }
            }
        }
        let scratch = TemporaryBuild(
            directory
                .ok_or_else(|| issue(9001, "No safe native-build directory name is available"))?,
        );
        let assembly = scratch.0.join("program.s");
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program.elf");
        std::fs::write(&assembly, assembler)
            .map_err(|error| issue(9001, format!("Cannot write native assembly: {error}")))?;
        run_tool(
            "as",
            &[
                OsStr::new("--64"),
                OsStr::new("-o"),
                object.as_os_str(),
                assembly.as_os_str(),
            ],
        )?;
        run_tool(
            "ld",
            &[
                OsStr::new("-m"),
                OsStr::new("elf_x86_64"),
                OsStr::new("-o"),
                executable.as_os_str(),
                object.as_os_str(),
            ],
        )?;
        std::fs::hard_link(&executable, target)
            .map_err(|error| issue(9001, format!("Cannot create new native executable {:?}; existing files are never overwritten: {error}", target)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembler_has_real_machine_code_entrypoint_and_syscalls() {
        let source = "let a = 4\nlet b = 5\nprint(a + b)";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains(".global _start"));
        assert!(asm.contains("call nobody_print_i64"));
        assert!(asm.contains("syscall"));
        assert!(asm.contains("mov QWORD PTR [rbp-8], rax"));
    }

    #[test]
    fn native_backend_rejects_out_of_range_and_unsupported_features() {
        assert_eq!(
            compile_to_assembly("print(9223372036854775808)", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("let text = \"hello\"\ntext = 7\nprint(text)", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert!(compile_to_assembly("print(\"a\" + \"b\")", "ai")
            .unwrap()
            .contains("call nobody_text_concat"));
        assert_eq!(
            compile_to_assembly("if 1 { print(2) }", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("for n in range(true) { print(n) }", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("fn f(x) { print(x) }", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("let x = 2\nx = false", "ai")
                .unwrap_err()
                .code,
            4004
        );
    }

    #[test]
    fn native_text_bindings_use_two_slots_and_compare_by_bytes() {
        let asm = compile_to_assembly(
            "let greeting = \"Ahoj\"\nlet saved = greeting\ngreeting = \"Zmena\"\nprint(greeting, saved, greeting != saved)",
            "ai",
        )
        .unwrap();
        assert!(asm.contains("mov rdx, QWORD PTR [rbp-"));
        assert!(asm.contains("mov QWORD PTR [rbp-16], rdx"));
        assert!(asm.contains("call nobody_write_raw"));
        assert!(asm.contains("call nobody_text_equal"));
        assert!(asm.contains("repe cmpsb"));
    }

    #[test]
    fn native_utf8_len_uses_scalar_byte_boundaries() {
        let asm = compile_to_assembly(
            "let t = \"Život ☸\"\nprint(len(t), len(\"\"), len(t + \"!\"))",
            "ai",
        )
        .unwrap();
        assert!(asm.contains("call nobody_text_length"));
        assert!(asm.contains("and r8d, 192"));
        assert_eq!(
            compile_to_assembly("print(len(3))", "ai").unwrap_err().code,
            4004
        );
        assert_eq!(
            compile_to_assembly("print(len())", "ai").unwrap_err().code,
            4007
        );
    }

    #[test]
    fn native_text_arena_and_bytewise_equality_have_real_codegen() {
        let asm = compile_to_assembly(
            "let left = \"Život\"\nlet right = \" ☸\"\nlet both = left + right\nprint(both, both == \"Život ☸\", left != right)",
            "ai",
        )
        .unwrap();
        assert!(asm.contains("call nobody_text_concat"));
        assert!(asm.contains("call nobody_text_equal"));
        assert!(asm.contains("repe cmpsb"));
        assert!(asm.contains(".zero 1048576"));
        assert!(asm.contains(".Lnobody_text_budget:"));
        assert!(!asm.contains("Život ☸"));
        assert_eq!(
            compile_to_assembly("let t = \"x\"\nprint(t - \"y\")", "ai")
                .unwrap_err()
                .code,
            4004
        );
    }

    #[test]
    fn native_signed_range_uses_checked_advance_and_independent_iterator_slots() {
        let asm = compile_to_assembly(
            "let n = 99\nfor n in range(4, 0, -2) { print(n) }\nprint(n)",
            "ai",
        )
        .unwrap();
        assert!(asm.contains("jz .Lnobody_range_zero_step"));
        assert!(asm.contains("js .L"));
        assert!(asm.contains("jo .L"));
        assert!(asm.contains("cmp rax, QWORD PTR [rbp-"));
        assert!(asm.contains("    mov QWORD PTR [rbp-"));
        assert!(!asm.contains("range("));
    }

    #[test]
    fn native_range_rejects_shadowed_builtin_and_unsupported_iterable() {
        let shadowed = "let range = 2\nfor n in range(4) { print(n) }";
        assert_eq!(compile_to_assembly(shadowed, "ai").unwrap_err().code, 4004);
        assert_eq!(
            compile_to_assembly("for n in 7 { print(n) }", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("for n in range(1, 3, true) { print(n) }", "ai")
                .unwrap_err()
                .code,
            4004
        );
    }

    #[test]
    fn explicit_native_function_signatures_override_ambiguous_inference() {
        let source = "fn identity(flag: bool): bool { return flag }
fn choose(flag: bool, a: int, b: int): int {
  if flag { return a } else { return b }
}
print(identity(true))
print(choose(false, 3, 9))";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("call nobody_print_bool"));
        assert!(asm.contains("call nobody_print_i64"));
        assert!(asm.contains("    call .L"));
    }

    #[test]
    fn typed_native_functions_reject_wrong_parameter_and_return_types() {
        for source in [
            "fn identity(flag: bool): bool { return flag }\nprint(identity(4))",
            "fn identity(flag: bool): int { return flag }\nprint(identity(true))",
            "fn f(n: int): bool { return n > 0 }\nprint(f(true))",
            "fn f(n: str): int { return 4 }\nprint(f(true))",
            "fn f(n: float): int { return 4 }\nprint(f(3))",
            "fn f(n: bool): bool { return n + 1 }\nprint(f(true))",
        ] {
            assert_eq!(
                compile_to_assembly(source, "ai").unwrap_err().code,
                4004,
                "{source}"
            );
        }
    }

    #[test]
    fn native_typed_string_function_abi_allocates_two_words_per_string() {
        let source = "fn pair(first: str, count: int, second: str): str { return first + second }\nprint(pair(\"Život\", 42, \" ☸\"))";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("call nobody_text_concat"));
        assert!(asm.contains("mov rax, QWORD PTR [rbp+"));
        assert!(asm.contains("push rdx"));
        assert!(asm.contains("add rsp, 40"));
        assert!(asm.contains("mov rdx, QWORD PTR [rbp-"));
    }

    #[test]
    fn native_typed_text_function_calls_reject_wrong_types() {
        for source in [
            "fn identity(text: str): str { return text }\nprint(identity(2))",
            "fn identity(text: str): str { return text }\nprint(identity(true))",
            "fn identity(text: str): int { return text }\nprint(identity(\"ok\"))",
            "fn bad(text: str): str { return text + true }\nprint(bad(\"ok\"))",
        ] {
            assert_eq!(
                compile_to_assembly(source, "ai").unwrap_err().code,
                4004,
                "{source}"
            );
        }
    }

    #[test]
    fn native_function_assembly_uses_independent_frames() {
        let source = "fn twice(x) { return x * 2 }
fn combine(a, b) { return twice(a) + b }
print(combine(4, 3))";
        let assembly = compile_to_assembly(source, "ai").unwrap();
        assert!(assembly.contains("    push rbp"));
        assert!(assembly.contains("    mov rbp, rsp"));
        assert!(assembly.contains("    leave"));
        assert!(assembly.contains("    ret"));
        assert!(assembly.contains("    call .L"));
        assert!(assembly.contains("QWORD PTR [rbp+"));
        assert!(!assembly.contains("combine("));
        assert!(!assembly.contains("twice("));
    }

    #[test]
    fn native_function_rejects_forward_calls_and_incomplete_returns() {
        let forward = "fn a(x) { return b(x) }\nfn b(x) { return x }\nprint(a(4))";
        assert_eq!(compile_to_assembly(forward, "ai").unwrap_err().code, 4001);
        let recursion = "fn recurse(x) { return recurse(x) }\nprint(recurse(2))";
        assert_eq!(compile_to_assembly(recursion, "ai").unwrap_err().code, 4001);
        let implicit_null = "fn only_print(x) { print(x) }\nonly_print(2)";
        assert_eq!(
            compile_to_assembly(implicit_null, "ai").unwrap_err().code,
            4004
        );
        let missing_path = "fn missing(x) { if x > 0 { return x } }\nprint(missing(2))";
        assert_eq!(
            compile_to_assembly(missing_path, "ai").unwrap_err().code,
            4004
        );
    }

    #[test]
    fn native_function_checks_argument_types_and_local_boundaries() {
        let wrong_arity = "fn twice(x) { return x * 2 }\nprint(twice(1, 2))";
        assert_eq!(
            compile_to_assembly(wrong_arity, "ai").unwrap_err().code,
            4007
        );
        let bool_arg = "fn twice(x) { return x * 2 }\nprint(twice(true))";
        assert_eq!(compile_to_assembly(bool_arg, "ai").unwrap_err().code, 4004);
        let mixed_return =
            "fn bigger(a, b) { if a > b { return true } else { return a } }\nprint(bigger(4, 2))";
        assert_eq!(
            compile_to_assembly(mixed_return, "ai").unwrap_err().code,
            4004
        );
        let global = "let global_value = 10\nfn f() { return global_value }\nprint(f())";
        assert_eq!(compile_to_assembly(global, "ai").unwrap_err().code, 4001);
        let nested = "fn outer(x) { fn inner(y) { return y } return x }";
        assert_eq!(compile_to_assembly(nested, "ai").unwrap_err().code, 4004);
        let shadow = "fn f(x) { return x }\nlet f = 2\nprint(f(2))";
        assert_eq!(compile_to_assembly(shadow, "ai").unwrap_err().code, 4004);
    }

    #[test]
    fn boolean_values_short_circuit_without_evaluating_overflowing_rhs() {
        let source = "print(true)
print(false)
print(false and (9223372036854775807 + 1 > 0))
print(true or (9223372036854775807 + 1 > 0))
print(true and false)
print(false or true)";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("call nobody_print_bool"));
        assert!(asm.contains("nobody_print_bool:"));
        assert!(asm.contains(".Lnobody_bool_true_text:"));
        assert!(asm.contains(".Lnobody_bool_false_text:"));
        assert!(asm.contains("    jz .L"));
        assert!(asm.contains("    jnz .L"));
    }

    #[test]
    fn native_boolean_function_returns_are_typed_and_reusable() {
        let source = "fn positive(x) { return x > 0 }
fn greater(a, b) {
  if a > b { return true } else { return false }
}
print(positive(4))
print(greater(3, 8))
if positive(2) and greater(9, 2) { print(1) } else { print(0) }";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("call nobody_print_bool"));
        assert!(asm.contains("    call .L"));
        assert!(asm.contains("    ret"));
    }

    #[test]
    fn invalid_logical_operators_and_mixed_returns_fail_closed() {
        for source in [
            "print(true and 4)",
            "print(7 or true)",
            "print(false or 9)",
            "fn f(x) { if x > 0 { return true } else { return 2 } }\nprint(f(3))",
        ] {
            assert_eq!(
                compile_to_assembly(source, "ai").unwrap_err().code,
                4004,
                "{source}"
            );
        }
    }

    #[test]
    fn native_function_parameters_infer_direct_boolean_constraints() {
        let source = "fn choose(flag, a, b) { if flag { return a } else { return b } }
fn invert(flag) { return not flag }
fn conjunction(a, b) { return a and b }
print(choose(true, 3, 9))
print(invert(false))
print(conjunction(true, false))";
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("    call .L"));
        assert!(asm.contains("call nobody_print_bool"));
        assert!(asm.contains("call nobody_print_i64"));
        assert!(asm.contains("    jz .L"));
    }

    #[test]
    fn boolean_param_inference_rejects_wrong_types_and_conflicts() {
        for source in [
            include_str!("../examples/native_bool_params_wrong_ai.nbd"),
            include_str!("../examples/native_bool_params_conflict_ai.nbd"),
            include_str!("../examples/native_bool_params_unconstrained_ai.nbd"),
        ] {
            assert_eq!(
                compile_to_assembly(source, "ai").unwrap_err().code,
                4004,
                "{source}"
            );
        }
    }

    #[test]
    fn boolean_param_inference_respects_nested_scope_shadowing() {
        let source = include_str!("../examples/native_bool_params_shadow_ai.nbd");
        let asm = compile_to_assembly(source, "ai").unwrap();
        assert!(asm.contains("    call .L"));
        assert!(asm.contains("call nobody_print_bool"));
        assert!(asm.contains("call nobody_print_i64"));
    }

    #[test]
    fn native_backend_does_not_interpolate_user_variable_names_into_assembly() {
        let asm =
            compile_to_assembly("let slovensky_nazov = 5\nprint(slovensky_nazov)", "ai").unwrap();
        assert!(!asm.contains("slovensky_nazov"));
        assert!(asm.contains("[rbp-8]"));
    }
}
