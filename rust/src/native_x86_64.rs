//! First real Linux x86-64 machine-code backend for a deliberately small Nobody
//! subset. Emits GNU assembler source, invokes binutils as+ld, and produces an
//! ELF64 binary with its own Linux sys_write/sys_exit routines.
//!
//! No Python, Nobody VM, libc, embedded interpreter or Rust runtime is linked
//! into the output. This is not a full language backend or an OS sandbox.
//!
//! Supported: signed i64 numbers, boolean conditions, checked + - *,
//! comparisons, lexical variables, if/elif/else, while, signed for-range,
//! break/continue, and print(one integer). Unsupported features fail before output.
use crate::ir::{IrExpr, IrStatement, Module, Name};
use crate::{lower_source, Diagnostic};
use std::collections::BTreeMap;
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

struct NativeCompiler {
    asm: String,
    scopes: Vec<BTreeMap<String, Binding>>,
    slots: usize,
    labels: usize,
    loops: Vec<LoopTargets>,
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
            IrExpr::Boolean(value) => {
                self.emit(if *value { "    mov eax, 1" } else { "    xor eax, eax" })?;
                Ok(Type::Bool)
            }
            IrExpr::Name(Name::User(name)) => {
                let binding = self.lookup(name).ok_or_else(|| issue(4001, format!("Unknown native variable {name:?}")))?;
                self.emit(&format!("    mov rax, QWORD PTR [rbp-{}]", (binding.slot + 1) * 8))?;
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
                let left = self.expression(lhs, depth + 1)?;
                self.emit("    push rax")?;
                let right = self.expression(rhs, depth + 1)?;
                self.emit("    pop rcx")?;
                let numeric = left == Type::Int && right == Type::Int;
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
                    "==" | "!=" if left == right => {
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
            _ => Err(unsupported("Native backend does not yet compile calls, strings, null, ranges or other complex values")),
        }
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
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (slot + 1) * 8))?;
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
                self.emit(&format!("    mov QWORD PTR [rbp-{}], rax", (binding.slot + 1) * 8))
            }
            IrStatement::Expression(IrExpr::Call { callee, args })
                if matches!(callee.as_ref(), IrExpr::Name(Name::Builtin("print"))) =>
            {
                if args.len() != 1 {
                    return Err(unsupported("Native output currently supports print(one integer) only"));
                }
                if self.expression(&args[0], 0)? != Type::Int {
                    return Err(unsupported("Native print currently supports integers only"));
                }
                self.emit("    mov rdi, rax")?;
                self.emit("    call nobody_print_i64")
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
            IrStatement::Break => {
                let target = self.loops.last().ok_or_else(|| unsupported("break outside native loop"))?.end;
                self.emit(&format!("    jmp .L{target}"))
            }
            IrStatement::Continue => {
                let target = self.loops.last().ok_or_else(|| unsupported("continue outside native loop"))?.start;
                self.emit(&format!("    jmp .L{target}"))
            }
            _ => Err(unsupported("Native x86-64 subset does not yet support functions, return, typed declarations or other statement forms")),
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
    compiler.emit(".Lnobody_budget:")?;
    compiler.emit("    mov edi, 5")?;
    compiler.emit("    mov eax, 60")?;
    compiler.emit("    syscall")?;
    compiler.emit("nobody_print_i64:")?;
    // The native output does not use libc or the bytecode interpreter.
    // Unsigned division handles i64::MIN after two's-complement negation.
    compiler.emit("    sub rsp, 64")?;
    compiler.emit("    lea rsi, [rsp+63]")?;
    compiler.emit("    mov BYTE PTR [rsi], 10")?;
    compiler.emit("    mov r10, 1")?;
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
            compile_to_assembly("print(\"text\")", "ai")
                .unwrap_err()
                .code,
            4004
        );
        assert_eq!(
            compile_to_assembly("print(true)", "ai").unwrap_err().code,
            4004
        );
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
            compile_to_assembly("fn f(x) { return x }", "ai")
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
    fn native_backend_does_not_interpolate_user_variable_names_into_assembly() {
        let asm =
            compile_to_assembly("let slovensky_nazov = 5\nprint(slovensky_nazov)", "ai").unwrap();
        assert!(!asm.contains("slovensky_nazov"));
        assert!(asm.contains("[rbp-8]"));
    }
}
