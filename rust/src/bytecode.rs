//! Experimental Phase 4 bytecode compiler + verifier + deterministic stack VM.
//!
//! This is an IN-MEMORY, integer-and-boolean subset, not a portable file format,
//! native compiler, security sandbox, or self-hosting implementation.
//! Unsupported programs fail explicitly rather than falling back to Python
//! or to the existing AST evaluator.
use crate::ir::{IrExpr, IrStatement, Module, Name};
use crate::{lower_source, Diagnostic};
use std::collections::BTreeMap;

pub const BYTECODE_VERSION: &str = "nobody.bytecode.experimental.v1";
const MAX_INSTRUCTIONS: usize = 10_000;
const MAX_STACK: usize = 1024;
const MAX_OUTPUT: usize = 1_048_576;
const MAX_EXPR_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Push(i128),
    PushBool(bool),
    PushText(String),
    PushNull,
    Load(usize),
    Define(usize),
    Store(usize),
    Add,
    Sub,
    Mul,
    Neg,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Jump(usize),
    JumpIfFalse(usize),
    // A signed range keeps current/stop/step in compiler-owned slots.
    // Check verifies a nonzero step; Advance exits on i128 overflow.
    RangeCheck {
        current: usize,
        stop: usize,
        step: usize,
        exit: usize,
    },
    RangeAdvance {
        current: usize,
        step: usize,
        exit: usize,
    },
    Drop,
    Print(usize),
    Call {
        function: usize,
        argc: usize,
    },
    Return,
    Halt,
}

/// Supported runtime primitive signatures in the experimental bytecode VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Bool,
    Str,
}

impl ValueType {
    fn from_annotation(value: &str) -> Result<Self, Diagnostic> {
        match value {
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            "str" => Ok(Self::Str),
            _ => Err(unsupported(format!(
                "Bytecode functions currently support int/bool/str annotations, not {value:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFunction {
    pub name: String,
    pub arity: usize,
    pub slots: usize,
    // Each function argument has an independently serialized optional type.
    pub parameter_types: Vec<Option<ValueType>>,
    pub return_type: Option<ValueType>,
    pub code: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub version: &'static str,
    pub slots: usize,
    pub code: Vec<Instruction>,
    pub functions: Vec<CompiledFunction>,
}

fn diagnostic(code: u16, detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        line: 0,
        column: 0,
        detail: detail.into(),
        repair:
            "Use the documented integer bytecode subset; verify the program before executing it.",
    }
}

fn unsupported(detail: impl Into<String>) -> Diagnostic {
    diagnostic(4004, detail)
}

struct Compiler {
    code: Vec<Instruction>,
    // Lexical scope maps prevent a branch-local name from leaking after the
    // block. Every declaration gets a fresh slot, even inside a loop.
    scopes: Vec<BTreeMap<String, usize>>,
    slots: usize,
    loops: Vec<LoopContext>,
    function_ids: BTreeMap<String, usize>,
    functions: Vec<CompiledFunction>,
    in_function: bool,
    blocked_function: Option<String>,
}

struct LoopContext {
    continue_target: usize,
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

impl Compiler {
    fn emit(&mut self, instruction: Instruction) -> Result<(), Diagnostic> {
        if self.code.len() >= MAX_INSTRUCTIONS {
            return Err(diagnostic(4005, "Bytecode instruction budget exceeded"));
        }
        self.code.push(instruction);
        Ok(())
    }

    fn mark(&self) -> usize {
        self.code.len()
    }

    fn patch(&mut self, position: usize, target: usize) -> Result<(), Diagnostic> {
        if target >= MAX_INSTRUCTIONS {
            return Err(diagnostic(
                4005,
                "Bytecode jump target exceeds instruction budget",
            ));
        }
        match self.code.get_mut(position) {
            Some(Instruction::Jump(address) | Instruction::JumpIfFalse(address)) => {
                *address = target;
                Ok(())
            }
            Some(Instruction::RangeCheck { exit, .. } | Instruction::RangeAdvance { exit, .. }) => {
                *exit = target;
                Ok(())
            }
            _ => Err(unsupported("Bytecode branch patch location is invalid")),
        }
    }

    fn temp(&mut self) -> Result<usize, Diagnostic> {
        if self.slots >= MAX_STACK {
            return Err(diagnostic(4005, "Bytecode slot budget exceeded"));
        }
        let slot = self.slots;
        self.slots += 1;
        Ok(slot)
    }

    fn variable(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn block(&mut self, body: &[IrStatement]) -> Result<(), Diagnostic> {
        self.scopes.push(BTreeMap::new());
        let result = body
            .iter()
            .try_for_each(|statement| self.statement(statement));
        self.scopes.pop();
        result
    }

    fn expression(&mut self, expression: &IrExpr, depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_EXPR_DEPTH {
            return Err(diagnostic(4005, "Bytecode expression depth exceeded"));
        }
        match expression {
            IrExpr::Number(text) => {
                if !text.bytes().all(|ch| ch.is_ascii_digit() || ch == b'_') {
                    return Err(unsupported(
                        "Bytecode currently supports integer literals only",
                    ));
                }
                let value = text
                    .replace('_', "")
                    .parse::<i128>()
                    .map_err(|_| diagnostic(4003, "Integer literal outside signed i128 range"))?;
                self.emit(Instruction::Push(value))
            }
            IrExpr::Boolean(value) => self.emit(Instruction::PushBool(*value)),
            IrExpr::String(text) => {
                let decoded = crate::evaluator::literal_text(text)?;
                if decoded.len() > MAX_OUTPUT {
                    return Err(diagnostic(4005, "Bytecode text literal exceeds 1 MiB"));
                }
                self.emit(Instruction::PushText(decoded))
            }
            IrExpr::Call { callee, args } => {
                let IrExpr::Name(Name::User(name)) = callee.as_ref() else {
                    return Err(unsupported("Bytecode only calls declared named functions"));
                };
                if self.blocked_function.as_deref() == Some(name.as_str()) {
                    return Err(diagnostic(
                        4005,
                        "Recursive bytecode function calls are not yet supported",
                    ));
                }
                let function = *self
                    .function_ids
                    .get(name)
                    .ok_or_else(|| diagnostic(4001, format!("Unknown function {name:?}")))?;
                let expected = self.functions[function].arity;
                if args.len() != expected {
                    return Err(diagnostic(
                        4007,
                        format!(
                            "Function {name:?} expects {expected} argument(s), received {}",
                            args.len()
                        ),
                    ));
                }
                for arg in args {
                    self.expression(arg, depth + 1)?;
                }
                self.emit(Instruction::Call {
                    function,
                    argc: args.len(),
                })
            }

            IrExpr::Name(Name::User(name)) => {
                let slot = self
                    .variable(name)
                    .ok_or_else(|| diagnostic(4001, format!("Unknown variable {name:?}")))?;
                self.emit(Instruction::Load(slot))
            }
            IrExpr::Unary { op, value } => {
                self.expression(value, depth + 1)?;
                match op.as_str() {
                    "+" => Ok(()),
                    "-" => self.emit(Instruction::Neg),
                    "not" => self.emit(Instruction::Not),
                    _ => Err(unsupported(format!(
                        "Unsupported bytecode unary operator {op:?}"
                    ))),
                }
            }
            IrExpr::Binary { op, lhs, rhs } => {
                let instruction = match op.as_str() {
                    "+" => Instruction::Add,
                    "-" => Instruction::Sub,
                    "*" => Instruction::Mul,
                    "==" => Instruction::Eq,
                    "!=" => Instruction::Ne,
                    "<" => Instruction::Lt,
                    "<=" => Instruction::Le,
                    ">" => Instruction::Gt,
                    ">=" => Instruction::Ge,
                    _ => {
                        return Err(unsupported(format!(
                            "Unsupported bytecode binary operator {op:?}"
                        )))
                    }
                };
                self.expression(lhs, depth + 1)?;
                self.expression(rhs, depth + 1)?;
                self.emit(instruction)
            }
            _ => Err(unsupported("Bytecode expression type not yet implemented")),
        }
    }

    fn statement(&mut self, statement: &IrStatement) -> Result<(), Diagnostic> {
        match statement {
            IrStatement::Let { name, type_name: None, expr } => {
                if self.scopes.last().is_some_and(|scope| scope.contains_key(name)) {
                    return Err(unsupported(format!("Duplicate declaration {name:?}")));
                }
                self.expression(expr, 0)?;
                let slot = self.temp()?;
                self.emit(Instruction::Define(slot))?;
                self.scopes.last_mut().expect("compiler root scope exists")
                    .insert(name.clone(), slot);
                Ok(())
            }
            IrStatement::Assign { name, expr } => {
                let slot = self.variable(name).ok_or_else(|| {
                    diagnostic(4001, format!("Unknown assignment target {name:?}"))
                })?;
                self.expression(expr, 0)?;
                self.emit(Instruction::Store(slot))
            }
            IrStatement::Expression(IrExpr::Call { callee, args })
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("print"))) =>
            {
                for arg in args {
                    self.expression(arg, 0)?;
                }
                self.emit(Instruction::Print(args.len()))
            }
            IrStatement::Expression(expr) => {
                self.expression(expr, 0)?;
                self.emit(Instruction::Drop)
            }
            IrStatement::If { branches, otherwise } => {
                let mut exits = Vec::with_capacity(branches.len());
                for (condition, body) in branches {
                    self.expression(condition, 0)?;
                    let skip = self.mark();
                    self.emit(Instruction::JumpIfFalse(usize::MAX))?;
                    self.block(body)?;
                    let exit = self.mark();
                    self.emit(Instruction::Jump(usize::MAX))?;
                    exits.push(exit);
                    self.patch(skip, self.mark())?;
                }
                if let Some(body) = otherwise {
                    self.block(body)?;
                }
                let end = self.mark();
                for exit in exits {
                    self.patch(exit, end)?;
                }
                Ok(())
            }
            IrStatement::While { condition, body } => {
                let start = self.mark();
                self.expression(condition, 0)?;
                let exit = self.mark();
                self.emit(Instruction::JumpIfFalse(usize::MAX))?;
                self.loops.push(LoopContext {
                    continue_target: start,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                let result = self.block(body);
                let control = self.loops.pop().expect("loop context exists");
                result?;
                self.emit(Instruction::Jump(start))?;
                let end = self.mark();
                self.patch(exit, end)?;
                for branch in control.breaks {
                    self.patch(branch, end)?;
                }
                Ok(())
            }
            IrStatement::For {
                variable,
                iterable,
                body,
            } => {
                let IrExpr::Call { callee, args } = iterable else {
                    return Err(unsupported("Bytecode for loops require range(...)"));
                };
                if !matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("range"))) {
                    return Err(unsupported("Bytecode for loops require unshadowed range(...)"));
                }
                if !(1..=3).contains(&args.len()) {
                    return Err(unsupported("range requires one, two or three integer arguments"));
                }

                // Evaluate range arguments in source order and exactly once.
                // Hidden slots are deliberately unreachable from user source.
                let mut argument_slots = Vec::with_capacity(args.len());
                for arg in args {
                    self.expression(arg, 0)?;
                    let slot = self.temp()?;
                    self.emit(Instruction::Define(slot))?;
                    argument_slots.push(slot);
                }

                let current = self.temp()?;
                let stop = self.temp()?;
                let step = self.temp()?;
                match argument_slots.as_slice() {
                    [end] => {
                        self.emit(Instruction::Push(0))?;
                        self.emit(Instruction::Define(current))?;
                        self.emit(Instruction::Load(*end))?;
                        self.emit(Instruction::Define(stop))?;
                    }
                    [start, end] | [start, end, _] => {
                        self.emit(Instruction::Load(*start))?;
                        self.emit(Instruction::Define(current))?;
                        self.emit(Instruction::Load(*end))?;
                        self.emit(Instruction::Define(stop))?;
                    }
                    _ => unreachable!("range argument arity checked"),
                }
                if let [_, _, stride] = argument_slots.as_slice() {
                    self.emit(Instruction::Load(*stride))?;
                } else {
                    self.emit(Instruction::Push(1))?;
                }
                self.emit(Instruction::Define(step))?;

                // The loop's iterator shadows an outer binding and is never
                // visible after the loop. Body declarations have a child scope.
                self.scopes.push(BTreeMap::new());
                let iterator = self.temp()?;
                self.scopes.last_mut().expect("range scope exists")
                    .insert(variable.clone(), iterator);

                let start = self.mark();
                self.emit(Instruction::RangeCheck {
                    current,
                    stop,
                    step,
                    exit: usize::MAX,
                })?;
                self.emit(Instruction::Load(current))?;
                self.emit(Instruction::Define(iterator))?;

                self.loops.push(LoopContext {
                    continue_target: usize::MAX,
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                let result = self.block(body);
                let loop_context = self.loops.pop().expect("range loop context exists");
                self.scopes.pop();
                result?;

                let advance = self.mark();
                for jump in loop_context.continues {
                    self.patch(jump, advance)?;
                }
                self.emit(Instruction::RangeAdvance {
                    current,
                    step,
                    exit: usize::MAX,
                })?;
                let advance_instruction = advance;
                self.emit(Instruction::Jump(start))?;
                let end = self.mark();
                self.patch(start, end)?;
                self.patch(advance_instruction, end)?;
                for jump in loop_context.breaks {
                    self.patch(jump, end)?;
                }
                Ok(())
            }
            IrStatement::Function {
                name, parameters, parameter_types, return_type, body,
            } => {
                if parameters.len() != parameter_types.len() {
                    return Err(unsupported("Function signature parameter metadata is inconsistent"));
                }
                let expected_types = parameter_types
                    .iter()
                    .map(|ty| ty.as_deref().map(ValueType::from_annotation).transpose())
                    .collect::<Result<Vec<_>, _>>()?;
                let expected_return = return_type.as_deref()
                    .map(ValueType::from_annotation)
                    .transpose()?;
                if self.in_function || self.scopes.len() != 1 {
                    return Err(unsupported("Nested bytecode functions and closures are not supported"));
                }
                if self.function_ids.contains_key(name) || self.variable(name).is_some() {
                    return Err(unsupported(format!("Duplicate function declaration {name:?}")));
                }
                if parameters.len() > MAX_STACK {
                    return Err(diagnostic(4005, "Function has too many parameters"));
                }
                let mut locals = BTreeMap::new();
                for (index, parameter) in parameters.iter().enumerate() {
                    if locals.insert(parameter.clone(), index).is_some() {
                        return Err(unsupported("Duplicate function parameter"));
                    }
                }
                // A function sees its parameters and locals and only earlier
                // declared callable functions; it never borrows caller locals.
                let mut child = Compiler {
                    code: Vec::new(),
                    scopes: vec![locals],
                    slots: parameters.len(),
                    loops: Vec::new(),
                    function_ids: self.function_ids.clone(),
                    functions: self.functions.clone(),
                    in_function: true,
                    blocked_function: Some(name.clone()),
                };
                for stmt in body {
                    child.statement(stmt)?;
                }
                child.emit(Instruction::PushNull)?;
                child.emit(Instruction::Return)?;
                let id = self.functions.len();
                self.functions.push(CompiledFunction {
                    name: name.clone(),
                    arity: parameters.len(),
                    slots: child.slots,
                    parameter_types: expected_types,
                    return_type: expected_return,
                    code: child.code,
                });
                self.function_ids.insert(name.clone(), id);
                Ok(())
            }
            IrStatement::Return(value) => {
                if !self.in_function {
                    return Err(unsupported("Return can only occur in a bytecode function"));
                }
                self.expression(value, 0)?;
                self.emit(Instruction::Return)
            }
            IrStatement::Break => {
                if self.loops.is_empty() {
                    return Err(unsupported("Break is only valid inside a bytecode loop"));
                }
                let at = self.mark();
                self.emit(Instruction::Jump(usize::MAX))?;
                self.loops.last_mut().expect("loop exists").breaks.push(at);
                Ok(())
            }
            IrStatement::Continue => {
                let target = self.loops.last().ok_or_else(|| {
                    unsupported("Continue is only valid inside a bytecode loop")
                })?.continue_target;
                let at = self.mark();
                self.emit(Instruction::Jump(target))?;
                if target == usize::MAX {
                    self.loops.last_mut().expect("loop exists").continues.push(at);
                }
                Ok(())
            }
            _ => Err(unsupported("Bytecode currently supports untyped declarations, integer/boolean expressions, conditionals, while, for-range, break/continue and print")),
        }
    }
}

/// Compile an IR module to a bounded, in-memory integer bytecode prototype.
/// Always verify before executing; this is not a serialized, portable format.
pub fn compile_ir(module: &Module) -> Result<Bytecode, Diagnostic> {
    let mut compiler = Compiler {
        code: Vec::new(),
        scopes: vec![BTreeMap::new()],
        slots: 0,
        loops: Vec::new(),
        function_ids: BTreeMap::new(),
        functions: Vec::new(),
        in_function: false,
        blocked_function: None,
    };
    for statement in &module.statements {
        compiler.statement(statement)?;
    }
    compiler.emit(Instruction::Halt)?;
    let bytecode = Bytecode {
        version: BYTECODE_VERSION,
        slots: compiler.slots,
        code: compiler.code,
        functions: compiler.functions,
    };
    verify(&bytecode)?;
    Ok(bytecode)
}

pub fn compile_source(source: &str, language: &str) -> Result<Bytecode, Diagnostic> {
    compile_ir(&lower_source(source, language)?)
}

/// Verify every reachable control-flow path with a fixed-point worklist.
/// Identical stack heights and definitely initialized slots are required at
/// merges, including loop back-edges. This is not yet static type checking.
pub fn verify(program: &Bytecode) -> Result<(), Diagnostic> {
    if program.version != BYTECODE_VERSION {
        return Err(unsupported("Unknown bytecode version"));
    }
    if program.functions.len() > MAX_STACK {
        return Err(diagnostic(4005, "Bytecode function count exceeds limit"));
    }
    for (id, function) in program.functions.iter().enumerate() {
        if function.name.is_empty()
            || function.arity > function.slots
            || function.parameter_types.len() != function.arity
        {
            return Err(unsupported("Invalid bytecode function metadata"));
        }
        verify_unit(
            &function.code,
            function.slots,
            function.arity,
            &program.functions,
            Some(id),
        )?;
    }
    verify_unit(&program.code, program.slots, 0, &program.functions, None)
}

fn verify_unit(
    code: &[Instruction],
    slots: usize,
    params: usize,
    functions: &[CompiledFunction],
    function_id: Option<usize>,
) -> Result<(), Diagnostic> {
    use std::collections::VecDeque;

    if code.is_empty() || code.len() > MAX_INSTRUCTIONS {
        return Err(unsupported(
            "Bytecode instruction count outside verifier limits",
        ));
    }
    if slots > MAX_STACK || params > slots {
        return Err(unsupported("Bytecode slot count outside verifier limits"));
    }
    let last = code.len() - 1;
    match (function_id, &code[last]) {
        (None, Instruction::Halt) | (Some(_), Instruction::Return) => {}
        _ => return Err(unsupported("Bytecode unit has the wrong terminal opcode")),
    }
    for (pc, instruction) in code.iter().enumerate() {
        match instruction {
            Instruction::Halt if function_id.is_some() || pc != last => {
                return Err(unsupported(
                    "Halt is only allowed at the end of top-level bytecode",
                ));
            }
            Instruction::Return if function_id.is_none() => {
                return Err(unsupported("Return is only allowed inside a function"));
            }
            Instruction::Jump(target) | Instruction::JumpIfFalse(target)
                if *target >= code.len() =>
            {
                return Err(unsupported(
                    "Bytecode jump destination is outside the program",
                ));
            }
            Instruction::RangeCheck { exit, .. } | Instruction::RangeAdvance { exit, .. }
                if *exit >= code.len() =>
            {
                return Err(unsupported(
                    "Bytecode range jump destination is outside the program",
                ));
            }
            Instruction::Call { function, argc } => {
                let Some(target) = functions.get(*function) else {
                    return Err(unsupported("Call references an invalid function"));
                };
                if target.arity != *argc {
                    return Err(unsupported(
                        "Call argument count differs from function metadata",
                    ));
                }
                if function_id.is_some_and(|current| *function >= current) {
                    return Err(unsupported(
                        "Recursive and forward bytecode calls are unsupported",
                    ));
                }
            }
            _ => {}
        }
    }

    #[derive(Clone)]
    struct State {
        depth: usize,
        initialized: Vec<bool>,
    }

    let mut initially_defined = vec![false; slots];
    initially_defined
        .iter_mut()
        .take(params)
        .for_each(|flag| *flag = true);
    let mut states: Vec<Option<State>> = vec![None; code.len()];
    states[0] = Some(State {
        depth: 0,
        initialized: initially_defined,
    });
    let mut pending = VecDeque::from([0usize]);
    let mut reaches_halt = false;

    while let Some(pc) = pending.pop_front() {
        let mut state = states[pc].clone().expect("queued state exists");
        let instruction = &code[pc];
        let mut successors = Vec::with_capacity(2);
        match instruction {
            Instruction::Push(_)
            | Instruction::PushBool(_)
            | Instruction::PushText(_)
            | Instruction::PushNull => state.depth += 1,
            Instruction::Load(slot) => {
                if !state.initialized.get(*slot).copied().unwrap_or(false) {
                    return Err(unsupported(
                        "Load references a slot not initialized on every path",
                    ));
                }
                state.depth += 1;
            }
            Instruction::Define(slot) => {
                let Some(initialized) = state.initialized.get_mut(*slot) else {
                    return Err(unsupported("Define references an invalid slot"));
                };
                // A declaration in a loop may execute on multiple iterations.
                // Compiler lexical scopes prevent duplicates in the same block.
                *initialized = true;
                state.depth = state
                    .depth
                    .checked_sub(1)
                    .ok_or_else(|| unsupported("Bytecode stack underflow at declaration"))?;
            }
            Instruction::Store(slot) => {
                if !state.initialized.get(*slot).copied().unwrap_or(false) {
                    return Err(unsupported(
                        "Store references a slot not initialized on every path",
                    ));
                }
                state.depth = state
                    .depth
                    .checked_sub(1)
                    .ok_or_else(|| unsupported("Bytecode stack underflow at assignment"))?;
            }
            Instruction::Add
            | Instruction::Sub
            | Instruction::Mul
            | Instruction::Eq
            | Instruction::Ne
            | Instruction::Lt
            | Instruction::Le
            | Instruction::Gt
            | Instruction::Ge => {
                state.depth = state
                    .depth
                    .checked_sub(1)
                    .filter(|depth| *depth > 0)
                    .ok_or_else(|| unsupported("Binary bytecode stack underflow"))?;
            }
            Instruction::Neg | Instruction::Not => {
                if state.depth == 0 {
                    return Err(unsupported("Unary bytecode stack underflow"));
                }
            }
            Instruction::Drop => {
                state.depth = state
                    .depth
                    .checked_sub(1)
                    .ok_or_else(|| unsupported("Bytecode drop stack underflow"))?;
            }
            Instruction::Print(count) => {
                state.depth = state
                    .depth
                    .checked_sub(*count)
                    .ok_or_else(|| unsupported("Bytecode print stack underflow"))?;
            }
            Instruction::Call { argc, .. } => {
                state.depth = state.depth.checked_sub(*argc).ok_or_else(|| {
                    unsupported("Bytecode function-call argument stack underflow")
                })? + 1;
            }
            Instruction::Return => {
                if state.depth != 1 {
                    return Err(unsupported(
                        "Bytecode Return requires exactly one result value",
                    ));
                }
                reaches_halt = true;
                continue;
            }
            Instruction::Jump(target) => successors.push(*target),
            Instruction::RangeCheck {
                current,
                stop,
                step,
                exit,
            } => {
                for slot in [current, stop, step] {
                    if !state.initialized.get(*slot).copied().unwrap_or(false) {
                        return Err(unsupported("Range check references an uninitialized slot"));
                    }
                }
                successors.push(*exit);
            }
            Instruction::RangeAdvance {
                current,
                step,
                exit,
            } => {
                for slot in [current, step] {
                    if !state.initialized.get(*slot).copied().unwrap_or(false) {
                        return Err(unsupported(
                            "Range advance references an uninitialized slot",
                        ));
                    }
                }
                successors.push(*exit);
            }
            Instruction::JumpIfFalse(target) => {
                state.depth = state
                    .depth
                    .checked_sub(1)
                    .ok_or_else(|| unsupported("Conditional jump stack underflow"))?;
                successors.push(*target);
            }
            Instruction::Halt => {
                if state.depth != 0 {
                    return Err(unsupported("Bytecode Halt requires an empty stack"));
                }
                reaches_halt = true;
                continue;
            }
        }
        if state.depth > MAX_STACK {
            return Err(diagnostic(4005, "Bytecode operand stack budget exceeded"));
        }
        if !matches!(instruction, Instruction::Jump(_)) {
            successors.push(pc + 1);
        }
        for target in successors {
            if target >= code.len() {
                return Err(unsupported("Bytecode control flow falls beyond final Halt"));
            }
            match states[target].as_mut() {
                None => {
                    states[target] = Some(state.clone());
                    pending.push_back(target);
                }
                Some(previous) => {
                    if previous.depth != state.depth {
                        return Err(unsupported(
                            "Bytecode stack heights disagree at control-flow merge",
                        ));
                    }
                    let mut changed = false;
                    for (old, incoming) in previous.initialized.iter_mut().zip(&state.initialized) {
                        if *old && !incoming {
                            *old = false;
                            changed = true;
                        }
                    }
                    if changed {
                        pending.push_back(target);
                    }
                }
            }
        }
    }
    if !reaches_halt {
        return Err(unsupported(
            "Bytecode has no reachable terminal instruction",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Int(i128),
    Bool(bool),
    Text(String),
    Null,
}

impl Value {
    fn matches_type(&self, expected: ValueType) -> bool {
        matches!(
            (self, expected),
            (Self::Int(_), ValueType::Int)
                | (Self::Bool(_), ValueType::Bool)
                | (Self::Text(_), ValueType::Str)
        )
    }

    fn display(&self) -> String {
        match self {
            Value::Int(value) => value.to_string(),
            Value::Bool(value) => value.to_string(),
            Value::Text(value) => value.clone(),
            Value::Null => "null".to_owned(),
        }
    }
}

struct CallFrame {
    pc: usize,
    function_id: Option<usize>,
    variables: Vec<Option<Value>>,
    stack: Vec<Value>,
}

/// Execute verified instructions with an explicit PC and step budget.
/// Bytecode can jump backwards, but never bypass its software instruction
/// budget. This is not an OS-enforced sandbox for untrusted source.
pub fn execute(program: &Bytecode) -> Result<String, Diagnostic> {
    verify(program)?;
    let mut variables = vec![None::<Value>; program.slots];
    let mut stack = Vec::<Value>::new();
    let mut output = String::new();
    let mut pc = 0usize;
    let mut steps = 0usize;
    let mut current_function: Option<usize> = None;
    let mut frames: Vec<CallFrame> = Vec::new();
    loop {
        steps += 1;
        if steps > MAX_INSTRUCTIONS {
            return Err(diagnostic(4005, "Bytecode VM instruction budget exhausted"));
        }
        let code = match current_function {
            Some(id) => &program.functions[id].code,
            None => &program.code,
        };
        let instruction = &code[pc];
        match instruction {
            Instruction::Push(value) => stack.push(Value::Int(*value)),
            Instruction::PushBool(value) => stack.push(Value::Bool(*value)),
            Instruction::PushText(value) => stack.push(Value::Text(value.clone())),
            Instruction::PushNull => stack.push(Value::Null),
            Instruction::Load(slot) => {
                let value = variables[*slot]
                    .clone()
                    .ok_or_else(|| unsupported("Uninitialized bytecode slot at runtime"))?;
                stack.push(value);
            }
            Instruction::Define(slot) | Instruction::Store(slot) => {
                variables[*slot] = stack.pop();
            }
            Instruction::Add | Instruction::Sub | Instruction::Mul => {
                let right = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                let left = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                match (left, right, instruction) {
                    (Value::Text(a), Value::Text(b), Instruction::Add) => {
                        if a.len().saturating_add(b.len()) > MAX_OUTPUT {
                            return Err(diagnostic(
                                4005,
                                "Bytecode text concatenation exceeds 1 MiB",
                            ));
                        }
                        stack.push(Value::Text(a + &b));
                    }
                    (Value::Int(a), Value::Int(b), op) => {
                        let result = match op {
                            Instruction::Add => a.checked_add(b),
                            Instruction::Sub => a.checked_sub(b),
                            _ => a.checked_mul(b),
                        }
                        .ok_or_else(|| diagnostic(4003, "Bytecode arithmetic overflow"))?;
                        stack.push(Value::Int(result));
                    }
                    _ => {
                        return Err(unsupported(
                            "Arithmetic accepts integers, or two strings for '+'",
                        ))
                    }
                }
            }
            Instruction::Neg => {
                let Some(Value::Int(value)) = stack.pop() else {
                    return Err(unsupported("Unary minus requires an integer"));
                };
                stack.push(Value::Int(value.checked_neg().ok_or_else(|| {
                    diagnostic(4003, "Bytecode unary negation overflow")
                })?));
            }
            Instruction::Not => {
                let Some(Value::Bool(value)) = stack.pop() else {
                    return Err(unsupported("Logical not requires a boolean"));
                };
                stack.push(Value::Bool(!value));
            }
            Instruction::Eq | Instruction::Ne => {
                let right = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                let left = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                let equal = left == right;
                stack.push(Value::Bool(if matches!(instruction, Instruction::Eq) {
                    equal
                } else {
                    !equal
                }));
            }
            Instruction::Lt | Instruction::Le | Instruction::Gt | Instruction::Ge => {
                let right = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                let left = stack
                    .pop()
                    .ok_or_else(|| unsupported("VM stack underflow"))?;
                let (Value::Int(left), Value::Int(right)) = (left, right) else {
                    return Err(unsupported("Ordered comparisons require integer values"));
                };
                let value = match instruction {
                    Instruction::Lt => left < right,
                    Instruction::Le => left <= right,
                    Instruction::Gt => left > right,
                    _ => left >= right,
                };
                stack.push(Value::Bool(value));
            }
            Instruction::RangeCheck {
                current,
                stop,
                step,
                exit,
            } => {
                let (Some(Value::Int(current)), Some(Value::Int(stop)), Some(Value::Int(step))) =
                    (&variables[*current], &variables[*stop], &variables[*step])
                else {
                    return Err(unsupported("range arguments must be integers"));
                };
                if *step == 0 {
                    return Err(diagnostic(4006, "range step cannot be zero"));
                }
                let in_range = if *step > 0 {
                    current < stop
                } else {
                    current > stop
                };
                if !in_range {
                    pc = *exit;
                    continue;
                }
            }
            Instruction::RangeAdvance {
                current,
                step,
                exit,
            } => {
                let (Some(Value::Int(value)), Some(Value::Int(step))) =
                    (&variables[*current], &variables[*step])
                else {
                    return Err(unsupported("range current value and step must be integers"));
                };
                if let Some(next) = value.checked_add(*step) {
                    variables[*current] = Some(Value::Int(next));
                } else {
                    // The reference evaluator terminates rather than wrapping
                    // when the final iteration crosses the i128 boundary.
                    pc = *exit;
                    continue;
                }
            }
            Instruction::Jump(target) => {
                pc = *target;
                continue;
            }
            Instruction::JumpIfFalse(target) => {
                let Some(Value::Bool(condition)) = stack.pop() else {
                    return Err(unsupported("While and if conditions must be boolean"));
                };
                if !condition {
                    pc = *target;
                    continue;
                }
            }
            Instruction::Drop => {
                stack.pop();
            }
            Instruction::Print(count) => {
                let start = stack.len() - *count;
                let rendered = stack[start..]
                    .iter()
                    .map(Value::display)
                    .collect::<Vec<_>>()
                    .join(" ")
                    + "\n";
                if output.len().saturating_add(rendered.len()) > MAX_OUTPUT {
                    return Err(diagnostic(4005, "Bytecode output budget exceeded"));
                }
                output.push_str(&rendered);
                stack.truncate(start);
            }
            Instruction::Call { function, argc } => {
                if frames.len() >= 32 {
                    return Err(diagnostic(4005, "Bytecode function call depth exceeded"));
                }
                let target = &program.functions[*function];
                let start = stack.len() - *argc;
                // Validate before consuming the caller's operands or pushing
                // a new frame. No implicit conversions or truthiness.
                for (index, (argument, annotation)) in stack[start..]
                    .iter()
                    .zip(&target.parameter_types)
                    .enumerate()
                {
                    if annotation.is_some_and(|expected| !argument.matches_type(expected)) {
                        return Err(unsupported(format!(
                            "Function {:?} argument {} violates its bytecode type annotation",
                            target.name,
                            index + 1
                        )));
                    }
                }
                let arguments = stack.split_off(start);
                frames.push(CallFrame {
                    pc: pc + 1,
                    function_id: current_function,
                    variables,
                    stack,
                });
                variables = vec![None; target.slots];
                for (slot, value) in arguments.into_iter().enumerate() {
                    variables[slot] = Some(value);
                }
                stack = Vec::new();
                current_function = Some(*function);
                pc = 0;
                continue;
            }
            Instruction::Return => {
                let value = stack
                    .pop()
                    .ok_or_else(|| unsupported("Empty function return stack"))?;
                let function_id =
                    current_function.ok_or_else(|| unsupported("Return without function"))?;
                let function = &program.functions[function_id];
                if function
                    .return_type
                    .is_some_and(|expected| !value.matches_type(expected))
                {
                    return Err(unsupported(format!(
                        "Function {:?} returned a value incompatible with its bytecode return annotation",
                        function.name
                    )));
                }
                let saved = frames
                    .pop()
                    .ok_or_else(|| unsupported("Return without caller"))?;
                variables = saved.variables;
                stack = saved.stack;
                stack.push(value);
                current_function = saved.function_id;
                pc = saved.pc;
                continue;
            }
            Instruction::Halt => return Ok(output),
        }
        pc += 1;
    }
}

pub fn execute_source(source: &str, language: &str) -> Result<String, Diagnostic> {
    execute(&compile_source(source, language)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytecode_matches_interpreter_on_ai_and_slovak() {
        for (language, source) in [
            (
                "ai",
                "let a = 4\nlet b = 5\nprint(a + b)\na = a * 2\nprint(a)",
            ),
            (
                "sk",
                "nech a = 4\nnech b = 5\nvypis(a + b)\na = a * 2\nvypis(a)",
            ),
        ] {
            let output = execute_source(source, language).unwrap();
            assert_eq!(output, "9\n8\n");
            assert_eq!(output, crate::evaluate_source(source, language).unwrap());
        }
    }

    #[test]
    fn verifier_rejects_invalid_stack_and_slots() {
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![Instruction::Add, Instruction::Halt],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 1,
            functions: Vec::new(),
            code: vec![Instruction::Load(0), Instruction::Drop, Instruction::Halt],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![Instruction::Halt, Instruction::Push(3)],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
    }

    #[test]
    fn typed_function_signatures_enforce_integer_and_boolean_values() {
        let valid = "fn identity(flag: bool): bool { return flag }\nfn twice(n: int): int { return n * 2 }\nprint(identity(true))\nprint(twice(5))";
        let compiled = compile_source(valid, "ai").unwrap();
        assert_eq!(
            compiled.functions[0].parameter_types,
            vec![Some(ValueType::Bool)]
        );
        assert_eq!(compiled.functions[0].return_type, Some(ValueType::Bool));
        assert_eq!(
            compiled.functions[1].parameter_types,
            vec![Some(ValueType::Int)]
        );
        assert_eq!(execute(&compiled).unwrap(), "true\n10\n");
        assert_eq!(
            execute(&compiled).unwrap(),
            crate::evaluate_source(valid, "ai").unwrap()
        );
        let untyped = "fn identity(flag) { return flag }\nprint(identity(true))";
        assert_eq!(
            compile_source(untyped, "ai").unwrap().functions[0].parameter_types,
            vec![None]
        );
    }

    #[test]
    fn bytecode_typed_calls_reject_invalid_arguments_and_returns() {
        for source in [
            "fn f(flag: bool): bool { return flag }\nprint(f(4))",
            "fn f(n: int): bool { return n }\nprint(f(4))",
            "fn f(n: int): bool { print(n) }\nprint(f(4))",
        ] {
            assert_eq!(execute_source(source, "ai").unwrap_err().code, 4004);
            assert_eq!(crate::evaluate_source(source, "ai").unwrap_err().code, 4004);
        }
        for source in [
            "fn f(n: float): int { return 1 }",
        ] {
            assert_eq!(compile_source(source, "ai").unwrap_err().code, 4004);
        }
    }

    #[test]
    fn strings_and_typed_string_functions_match_rust_ast_evaluator() {
        for (profile, source) in [
            ("ai", "let a = \"Život\"\nlet b = a + \" ☸\"\nprint(a, b, b == \"Život ☸\", a != b)\nfn identity(s: str): str { return s }\nprint(identity(b))"),
            ("sk", "nech a = \"Život\"\nnech b = a + \" ☸\"\nvypis(a, b, b == \"Život ☸\", a != b)\nfunkcia identity(s: text): text { vrat s }\nvypis(identity(b))"),
            ("en", "let a = \"Život\"\nlet b = a + \" ☸\"\nprint(a, b, b == \"Život ☸\", a != b)\nfunction identity(s: str): str { return s }\nprint(identity(b))"),
        ] {
            let actual = execute_source(source, profile).unwrap();
            assert_eq!(actual, crate::evaluate_source(source, profile).unwrap());
        }
    }

    #[test]
    fn string_operations_have_bounded_runtime_behavior() {
        let source = "let text = \"x\"\nfor i in range(22) { text = text + text }\nprint(text)";
        assert_eq!(execute_source(source, "ai").unwrap_err().code, 4005);
        assert_eq!(
            execute_source("print(\"a\" - \"b\")", "ai")
                .unwrap_err()
                .code,
            4004
        );
    }

    #[test]
    fn signed_range_loops_match_the_reference_evaluator() {
        for (profile, source) in [
            ("ai", include_str!("../examples/bytecode_for_ai.nbd")),
            ("sk", include_str!("../examples/bytecode_for_sk.nbd")),
            ("en", include_str!("../examples/bytecode_for_en.nbd")),
            ("ai", include_str!("../examples/bytecode_for_nested_ai.nbd")),
            ("ai", include_str!("../examples/bytecode_for_shadow_ai.nbd")),
            (
                "ai",
                include_str!("../examples/bytecode_for_overflow_ai.nbd"),
            ),
        ] {
            let vm = execute_source(source, profile).unwrap();
            let reference = crate::evaluate_source(source, profile).unwrap();
            assert_eq!(vm, reference, "{profile}: {source}");
        }
    }

    #[test]
    fn range_step_zero_has_the_reference_error_number() {
        let source = include_str!("../examples/bytecode_for_bad_step_ai.nbd");
        assert_eq!(execute_source(source, "ai").unwrap_err().code, 4006);
        assert_eq!(crate::evaluate_source(source, "ai").unwrap_err().code, 4006);
    }

    #[test]
    fn malicious_range_slots_and_jump_targets_fail_verification() {
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 3,
            functions: Vec::new(),
            code: vec![
                Instruction::RangeCheck {
                    current: 0,
                    stop: 1,
                    step: 2,
                    exit: 1,
                },
                Instruction::Halt,
            ],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![
                Instruction::RangeAdvance {
                    current: 0,
                    step: 0,
                    exit: 4,
                },
                Instruction::Halt,
            ],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
    }

    #[test]
    fn integer_overflow_is_reported_not_wrapped() {
        let program = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![
                Instruction::Push(i128::MAX),
                Instruction::Push(1),
                Instruction::Add,
                Instruction::Drop,
                Instruction::Halt,
            ],
        };
        assert_eq!(execute(&program).unwrap_err().code, 4003);
    }

    #[test]
    fn localized_unicode_names_compile() {
        let source = "nech číslo = 12\nvypis(číslo - 5)";
        assert_eq!(execute_source(source, "sk").unwrap(), "7\n");
    }

    #[test]
    fn if_while_break_continue_match_existing_runtime() {
        let source = "let i = 0
let total = 0
while i < 5 {
  i = i + 1
  if i == 2 { continue }
  if i == 5 { break }
  total = total + i
}
print(total)
if total >= 8 { print(true) } else { print(false) }";
        let bytecode_output = execute_source(source, "ai").unwrap();
        assert_eq!(bytecode_output, "8\ntrue\n");
        assert_eq!(
            bytecode_output,
            crate::evaluate_source(source, "ai").unwrap()
        );
    }

    #[test]
    fn branch_and_loop_scopes_can_shadow_variables() {
        let source = "let x = 4
if true { let x = 9 print(x) }
print(x)
while x < 7 {
  let temporary = x + 1
  x = temporary
  print(x)
}";
        let bytecode_output = execute_source(source, "ai").unwrap();
        assert_eq!(bytecode_output, "9\n4\n5\n6\n7\n");
        assert_eq!(
            bytecode_output,
            crate::evaluate_source(source, "ai").unwrap()
        );
    }

    #[test]
    fn malformed_jumps_and_uninitialized_branches_are_rejected() {
        let outside = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![Instruction::Jump(9), Instruction::Halt],
        };
        assert_eq!(verify(&outside).unwrap_err().code, 4004);
        let uninitialized = Bytecode {
            version: BYTECODE_VERSION,
            slots: 1,
            functions: Vec::new(),
            code: vec![
                Instruction::PushBool(false),
                Instruction::JumpIfFalse(4),
                Instruction::Push(1),
                Instruction::Define(0),
                Instruction::Load(0),
                Instruction::Drop,
                Instruction::Halt,
            ],
        };
        assert_eq!(verify(&uninitialized).unwrap_err().code, 4004);
        let mismatched = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![
                Instruction::PushBool(true),
                Instruction::JumpIfFalse(3),
                Instruction::Push(5),
                Instruction::Halt,
            ],
        };
        assert_eq!(verify(&mismatched).unwrap_err().code, 4004);
    }

    #[test]
    fn infinite_empty_while_stops_at_step_budget() {
        let error = execute_source("while true { }", "ai").unwrap_err();
        assert_eq!(error.code, 4005);
    }

    #[test]
    fn non_boolean_conditions_fail_without_truthiness() {
        let error = execute_source("if 1 { print(2) }", "ai").unwrap_err();
        assert_eq!(error.code, 4004);
    }

    #[test]
    fn named_functions_match_tree_evaluator_for_ai_sk_en() {
        for (profile, source) in [
            ("ai", include_str!("../examples/bytecode_functions_ai.nbd")),
            ("sk", include_str!("../examples/bytecode_functions_sk.nbd")),
            ("en", include_str!("../examples/bytecode_functions_en.nbd")),
            (
                "ai",
                include_str!("../examples/bytecode_function_control_ai.nbd"),
            ),
            (
                "ai",
                include_str!("../examples/bytecode_function_null_ai.nbd"),
            ),
        ] {
            let compiled = compile_source(source, profile).unwrap();
            assert!(!compiled.functions.is_empty());
            let output = execute(&compiled).unwrap();
            assert_eq!(output, crate::evaluate_source(source, profile).unwrap());
        }
    }

    #[test]
    fn wrong_arity_and_recursion_are_numbered() {
        let arity = include_str!("../examples/bytecode_function_bad_arity_ai.nbd");
        assert_eq!(compile_source(arity, "ai").unwrap_err().code, 4007);
        let recursive = include_str!("../examples/bytecode_function_recursive_ai.nbd");
        assert_eq!(compile_source(recursive, "ai").unwrap_err().code, 4005);
    }

    #[test]
    fn globals_and_closures_are_not_silently_supported() {
        let globals = include_str!("../examples/bytecode_function_globals_ai.nbd");
        assert_eq!(compile_source(globals, "ai").unwrap_err().code, 4001);
        let nested = "fn outer(x) { fn inner(y) { return y + 1 } return x }";
        assert_eq!(compile_source(nested, "ai").unwrap_err().code, 4004);
    }

    #[test]
    fn verifier_rejects_invalid_function_calls_and_returns() {
        let function = CompiledFunction {
            name: "identity".to_owned(),
            arity: 1,
            slots: 1,
            parameter_types: vec![Some(ValueType::Bool)],
            return_type: Some(ValueType::Bool),
            code: vec![Instruction::Load(0), Instruction::Return],
        };
        let make = |code| Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            code,
            functions: vec![function.clone()],
        };
        let missing_arg = make(vec![
            Instruction::Call {
                function: 0,
                argc: 1,
            },
            Instruction::Drop,
            Instruction::Halt,
        ]);
        assert_eq!(verify(&missing_arg).unwrap_err().code, 4004);
        let mut wrong_metadata = make(vec![Instruction::Halt]);
        wrong_metadata.functions[0].parameter_types.clear();
        assert_eq!(verify(&wrong_metadata).unwrap_err().code, 4004);
        let bad_id = make(vec![
            Instruction::Call {
                function: 1,
                argc: 0,
            },
            Instruction::Drop,
            Instruction::Halt,
        ]);
        assert_eq!(verify(&bad_id).unwrap_err().code, 4004);
        let bad_return = make(vec![Instruction::Return, Instruction::Halt]);
        assert_eq!(verify(&bad_return).unwrap_err().code, 4004);
        let cyclic_function = CompiledFunction {
            code: vec![
                Instruction::Load(0),
                Instruction::Call {
                    function: 0,
                    argc: 1,
                },
                Instruction::Return,
            ],
            ..function
        };
        let cyclic = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            code: vec![Instruction::Halt],
            functions: vec![cyclic_function],
        };
        assert_eq!(verify(&cyclic).unwrap_err().code, 4004);
    }

    #[test]
    fn verifier_rejects_output_stack_underflow() {
        let program = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            functions: Vec::new(),
            code: vec![Instruction::Print(1), Instruction::Halt],
        };
        assert!(verify(&program).is_err());
    }
}
