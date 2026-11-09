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
    RangeCheck { current: usize, stop: usize, step: usize, exit: usize },
    RangeAdvance { current: usize, step: usize, exit: usize },
    Drop,
    Print(usize),
    Halt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub version: &'static str,
    pub slots: usize,
    pub code: Vec<Instruction>,
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
    };
    for statement in &module.statements {
        compiler.statement(statement)?;
    }
    compiler.emit(Instruction::Halt)?;
    let bytecode = Bytecode {
        version: BYTECODE_VERSION,
        slots: compiler.slots,
        code: compiler.code,
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
    use std::collections::VecDeque;

    if program.version != BYTECODE_VERSION {
        return Err(unsupported("Unknown bytecode version"));
    }
    if program.code.is_empty() || program.code.len() > MAX_INSTRUCTIONS {
        return Err(unsupported(
            "Bytecode instruction count outside verifier limits",
        ));
    }
    if program.slots > MAX_STACK {
        return Err(unsupported("Bytecode slot count outside verifier limits"));
    }
    let last = program.code.len() - 1;
    if !matches!(program.code[last], Instruction::Halt) {
        return Err(unsupported("Bytecode must end with Halt"));
    }
    for (pc, instruction) in program.code.iter().enumerate() {
        match instruction {
            Instruction::Halt if pc != last => {
                return Err(unsupported(
                    "Halt cannot appear before the last instruction",
                ));
            }
            Instruction::Jump(target) | Instruction::JumpIfFalse(target)
                if *target >= program.code.len() =>
            {
                return Err(unsupported(
                    "Bytecode jump destination is outside the program",
                ));
            }
            Instruction::RangeCheck { exit, .. } | Instruction::RangeAdvance { exit, .. }
                if *exit >= program.code.len() =>
            {
                return Err(unsupported("Bytecode range jump destination is outside the program"));
            }
            _ => {}
        }
    }

    #[derive(Clone)]
    struct State {
        depth: usize,
        initialized: Vec<bool>,
    }

    let mut states: Vec<Option<State>> = vec![None; program.code.len()];
    states[0] = Some(State {
        depth: 0,
        initialized: vec![false; program.slots],
    });
    let mut pending = VecDeque::from([0usize]);
    let mut reaches_halt = false;

    while let Some(pc) = pending.pop_front() {
        let mut state = states[pc].clone().expect("queued state exists");
        let instruction = &program.code[pc];
        let mut successors = Vec::with_capacity(2);
        match instruction {
            Instruction::Push(_) | Instruction::PushBool(_) => state.depth += 1,
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
            Instruction::Jump(target) => successors.push(*target),
            Instruction::RangeCheck { current, stop, step, exit } => {
                for slot in [current, stop, step] {
                    if !state.initialized.get(*slot).copied().unwrap_or(false) {
                        return Err(unsupported("Range check references an uninitialized slot"));
                    }
                }
                successors.push(*exit);
            }
            Instruction::RangeAdvance { current, step, exit } => {
                for slot in [current, step] {
                    if !state.initialized.get(*slot).copied().unwrap_or(false) {
                        return Err(unsupported("Range advance references an uninitialized slot"));
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
            if target >= program.code.len() {
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
        return Err(unsupported("Bytecode has no reachable Halt"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Int(i128),
    Bool(bool),
}

impl Value {
    fn display(&self) -> String {
        match self {
            Value::Int(value) => value.to_string(),
            Value::Bool(value) => value.to_string(),
        }
    }
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
    loop {
        steps += 1;
        if steps > MAX_INSTRUCTIONS {
            return Err(diagnostic(4005, "Bytecode VM instruction budget exhausted"));
        }
        let instruction = &program.code[pc];
        match instruction {
            Instruction::Push(value) => stack.push(Value::Int(*value)),
            Instruction::PushBool(value) => stack.push(Value::Bool(*value)),
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
                let (Value::Int(left), Value::Int(right)) = (left, right) else {
                    return Err(unsupported("Arithmetic requires integer values"));
                };
                let value = match instruction {
                    Instruction::Add => left.checked_add(right),
                    Instruction::Sub => left.checked_sub(right),
                    _ => left.checked_mul(right),
                }
                .ok_or_else(|| diagnostic(4003, "Bytecode arithmetic overflow"))?;
                stack.push(Value::Int(value));
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
            Instruction::RangeCheck { current, stop, step, exit } => {
                let (Some(Value::Int(current)), Some(Value::Int(stop)), Some(Value::Int(step))) =
                    (&variables[*current], &variables[*stop], &variables[*step])
                else {
                    return Err(unsupported("range arguments must be integers"));
                };
                if *step == 0 {
                    return Err(diagnostic(4006, "range step cannot be zero"));
                }
                let in_range = if *step > 0 { current < stop } else { current > stop };
                if !in_range {
                    pc = *exit;
                    continue;
                }
            }
            Instruction::RangeAdvance { current, step, exit } => {
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
            code: vec![Instruction::Add, Instruction::Halt],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 1,
            code: vec![Instruction::Load(0), Instruction::Drop, Instruction::Halt],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            code: vec![Instruction::Halt, Instruction::Push(3)],
        };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
    }

    #[test]
    fn unsupported_range_loop_is_explicitly_rejected() {
        let err = compile_source("for n in range(3) { print(n) }", "ai").unwrap_err();
        assert_eq!(err.code, 4004);
    }

    #[test]
    fn integer_overflow_is_reported_not_wrapped() {
        let program = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
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
            code: vec![Instruction::Jump(9), Instruction::Halt],
        };
        assert_eq!(verify(&outside).unwrap_err().code, 4004);
        let uninitialized = Bytecode {
            version: BYTECODE_VERSION,
            slots: 1,
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
    fn verifier_rejects_output_stack_underflow() {
        let program = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            code: vec![Instruction::Print(1), Instruction::Halt],
        };
        assert!(verify(&program).is_err());
    }
}
