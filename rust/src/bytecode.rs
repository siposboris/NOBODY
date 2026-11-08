//! Experimental Phase 4 bytecode compiler + verifier + deterministic stack VM.
//!
//! This is an IN-MEMORY, integer-only subset, not a portable file format,
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
    Load(usize),
    Define(usize),
    Store(usize),
    Add,
    Sub,
    Mul,
    Neg,
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
        repair: "Use the documented integer bytecode subset; verify the program before executing it.",
    }
}

fn unsupported(detail: impl Into<String>) -> Diagnostic {
    diagnostic(4004, detail)
}

struct Compiler {
    code: Vec<Instruction>,
    variables: BTreeMap<String, usize>,
}

impl Compiler {
    fn emit(&mut self, instruction: Instruction) -> Result<(), Diagnostic> {
        if self.code.len() >= MAX_INSTRUCTIONS {
            return Err(diagnostic(4005, "Bytecode instruction budget exceeded"));
        }
        self.code.push(instruction);
        Ok(())
    }

    fn expression(&mut self, expression: &IrExpr, depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_EXPR_DEPTH {
            return Err(diagnostic(4005, "Bytecode expression depth exceeded"));
        }
        match expression {
            IrExpr::Number(text) => {
                if !text.bytes().all(|ch| ch.is_ascii_digit() || ch == b'_') {
                    return Err(unsupported("Bytecode currently supports integer literals only"));
                }
                let value = text.replace('_', "").parse::<i128>().map_err(|_| {
                    diagnostic(4003, "Integer literal outside signed i128 range")
                })?;
                self.emit(Instruction::Push(value))
            }
            IrExpr::Name(Name::User(name)) => {
                let slot = *self.variables.get(name).ok_or_else(|| {
                    diagnostic(4001, format!("Unknown variable {name:?}"))
                })?;
                self.emit(Instruction::Load(slot))
            }
            IrExpr::Unary { op, value } => {
                self.expression(value, depth + 1)?;
                match op.as_str() {
                    "+" => Ok(()),
                    "-" => self.emit(Instruction::Neg),
                    _ => Err(unsupported(format!("Unsupported bytecode unary operator {op:?}"))),
                }
            }
            IrExpr::Binary { op, lhs, rhs } => {
                let instruction = match op.as_str() {
                    "+" => Instruction::Add,
                    "-" => Instruction::Sub,
                    "*" => Instruction::Mul,
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
            IrStatement::Let {
                name,
                type_name: None,
                expr,
            } => {
                if self.variables.contains_key(name) {
                    return Err(unsupported(format!("Duplicate declaration {name:?}")));
                }
                self.expression(expr, 0)?;
                let slot = self.variables.len();
                self.emit(Instruction::Define(slot))?;
                self.variables.insert(name.clone(), slot);
                Ok(())
            }
            IrStatement::Assign { name, expr } => {
                let slot = *self.variables.get(name).ok_or_else(|| {
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
            _ => Err(unsupported("Bytecode statements currently support declarations, assignments, arithmetic and print only")),
        }
    }
}

/// Compile an IR module to a bounded, in-memory integer bytecode prototype.
/// Always verify before executing; this is not a serialized, portable format.
pub fn compile_ir(module: &Module) -> Result<Bytecode, Diagnostic> {
    let mut compiler = Compiler {
        code: Vec::new(),
        variables: BTreeMap::new(),
    };
    for statement in &module.statements {
        compiler.statement(statement)?;
    }
    compiler.emit(Instruction::Halt)?;
    let bytecode = Bytecode {
        version: BYTECODE_VERSION,
        slots: compiler.variables.len(),
        code: compiler.code,
    };
    verify(&bytecode)?;
    Ok(bytecode)
}

pub fn compile_source(source: &str, language: &str) -> Result<Bytecode, Diagnostic> {
    compile_ir(&lower_source(source, language)?)
}

/// Reject stack underflow, undefined slots, invalid halts and oversized
/// programs before allowing the VM to execute a bytecode object.
pub fn verify(program: &Bytecode) -> Result<(), Diagnostic> {
    if program.version != BYTECODE_VERSION {
        return Err(unsupported("Unknown bytecode version"));
    }
    if program.code.is_empty() || program.code.len() > MAX_INSTRUCTIONS {
        return Err(unsupported("Bytecode instruction count outside verifier limits"));
    }
    if program.slots > MAX_INSTRUCTIONS {
        return Err(unsupported("Bytecode slot count outside verifier limits"));
    }
    let mut initialized = vec![false; program.slots];
    let mut stack_depth = 0usize;
    for (index, instruction) in program.code.iter().enumerate() {
        if matches!(instruction, Instruction::Halt) {
            if index + 1 != program.code.len() || stack_depth != 0 {
                return Err(unsupported("Halt must be last and leave an empty stack"));
            }
            return Ok(());
        }
        match instruction {
            Instruction::Push(_) => stack_depth += 1,
            Instruction::Load(slot) => {
                if !initialized.get(*slot).copied().unwrap_or(false) {
                    return Err(unsupported("Load references an undefined slot"));
                }
                stack_depth += 1;
            }
            Instruction::Define(slot) => {
                let Some(flag) = initialized.get_mut(*slot) else {
                    return Err(unsupported("Define references an invalid slot"));
                };
                if *flag {
                    return Err(unsupported("Bytecode slot redeclaration"));
                }
                *flag = true;
                stack_depth = stack_depth.checked_sub(1).ok_or_else(|| {
                    unsupported("Bytecode stack underflow at declaration")
                })?;
            }
            Instruction::Store(slot) => {
                if !initialized.get(*slot).copied().unwrap_or(false) {
                    return Err(unsupported("Store references an undefined slot"));
                }
                stack_depth = stack_depth.checked_sub(1).ok_or_else(|| {
                    unsupported("Bytecode stack underflow at assignment")
                })?;
            }
            Instruction::Add | Instruction::Sub | Instruction::Mul => {
                stack_depth = stack_depth.checked_sub(1).filter(|depth| *depth >= 1).ok_or_else(|| {
                    unsupported("Bytecode arithmetic stack underflow")
                })?;
            }
            Instruction::Neg => {
                if stack_depth == 0 {
                    return Err(unsupported("Bytecode unary stack underflow"));
                }
            }
            Instruction::Drop => {
                stack_depth = stack_depth.checked_sub(1).ok_or_else(|| {
                    unsupported("Bytecode drop stack underflow")
                })?;
            }
            Instruction::Print(count) => {
                stack_depth = stack_depth.checked_sub(*count).ok_or_else(|| {
                    unsupported("Bytecode print stack underflow")
                })?;
            }
            Instruction::Halt => unreachable!(),
        }
        if stack_depth > MAX_STACK {
            return Err(diagnostic(4005, "Bytecode operand stack budget exceeded"));
        }
    }
    Err(unsupported("Bytecode has no terminating Halt instruction"))
}

/// Execute only bytecode that passes the verifier. The step and output
/// limits are software counters, not an OS process sandbox.
pub fn execute(program: &Bytecode) -> Result<String, Diagnostic> {
    verify(program)?;
    let mut variables = vec![None::<i128>; program.slots];
    let mut stack = Vec::<i128>::new();
    let mut output = String::new();
    for instruction in &program.code {
        match instruction {
            Instruction::Push(value) => stack.push(*value),
            Instruction::Load(slot) => {
                let value = variables[*slot].ok_or_else(|| unsupported("Uninitialized bytecode slot"))?;
                stack.push(value);
            }
            Instruction::Define(slot) | Instruction::Store(slot) => {
                variables[*slot] = stack.pop();
            }
            Instruction::Add | Instruction::Sub | Instruction::Mul => {
                let right = stack.pop().ok_or_else(|| unsupported("VM stack underflow"))?;
                let left = stack.pop().ok_or_else(|| unsupported("VM stack underflow"))?;
                let result = match instruction {
                    Instruction::Add => left.checked_add(right),
                    Instruction::Sub => left.checked_sub(right),
                    _ => left.checked_mul(right),
                }.ok_or_else(|| diagnostic(4003, "Bytecode arithmetic overflow"))?;
                stack.push(result);
            }
            Instruction::Neg => {
                let value = stack.pop().ok_or_else(|| unsupported("VM stack underflow"))?;
                stack.push(value.checked_neg().ok_or_else(|| {
                    diagnostic(4003, "Bytecode unary negation overflow")
                })?);
            }
            Instruction::Drop => {
                stack.pop();
            }
            Instruction::Print(count) => {
                let start = stack.len() - *count;
                let rendered = stack[start..]
                    .iter()
                    .map(i128::to_string)
                    .collect::<Vec<_>>()
                    .join(" ") + "\n";
                if output.len().saturating_add(rendered.len()) > MAX_OUTPUT {
                    return Err(diagnostic(4005, "Bytecode output budget exceeded"));
                }
                output.push_str(&rendered);
                stack.truncate(start);
            }
            Instruction::Halt => return Ok(output),
        }
    }
    Err(unsupported("Missing VM halt"))
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
            ("ai", "let a = 4\nlet b = 5\nprint(a + b)\na = a * 2\nprint(a)"),
            ("sk", "nech a = 4\nnech b = 5\nvypis(a + b)\na = a * 2\nvypis(a)"),
        ] {
            let output = execute_source(source, language).unwrap();
            assert_eq!(output, "9\n8\n");
            assert_eq!(output, crate::evaluate_source(source, language).unwrap());
        }
    }

    #[test]
    fn verifier_rejects_invalid_stack_and_slots() {
        let bad = Bytecode { version: BYTECODE_VERSION, slots: 0, code: vec![Instruction::Add, Instruction::Halt] };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode { version: BYTECODE_VERSION, slots: 1, code: vec![Instruction::Load(0), Instruction::Drop, Instruction::Halt] };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
        let bad = Bytecode { version: BYTECODE_VERSION, slots: 0, code: vec![Instruction::Halt, Instruction::Push(3)] };
        assert_eq!(verify(&bad).unwrap_err().code, 4004);
    }

    #[test]
    fn unsupported_control_flow_is_explicitly_rejected() {
        let err = compile_source("if true { print(1) }", "ai").unwrap_err();
        assert_eq!(err.code, 4004);
    }

    #[test]
    fn integer_overflow_is_reported_not_wrapped() {
        let program = Bytecode {
            version: BYTECODE_VERSION,
            slots: 0,
            code: vec![Instruction::Push(i128::MAX), Instruction::Push(1), Instruction::Add, Instruction::Drop, Instruction::Halt],
        };
        assert_eq!(execute(&program).unwrap_err().code, 4003);
    }

    #[test]
    fn localized_unicode_names_compile() {
        let source = "nech číslo = 12\nvypis(číslo - 5)";
        assert_eq!(execute_source(source, "sk").unwrap(), "7\n");
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
