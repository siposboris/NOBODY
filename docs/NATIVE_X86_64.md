# Nobody native Linux x86-64 backend (experimental)

This is the first **real ELF64 machine-code output** from Nobody. The Rust compiler lowers a small subset of canonical Nobody IR directly into x86-64 GNU Intel-syntax assembly, runs `as` and `ld`, and emits a standalone ELF executable with Linux syscalls for output and exit. The resulting executable needs **no Python, Nobody bytecode VM, Rust runtime or libc**. This is not a complete compiler backend, sandbox, or self-hosting.

## Compile and run on Fedora

Install GNU binutils (`sudo dnf install binutils`) if `as`, `ld` and `readelf` are missing. Rust/Cargo are required to build the compiler, not to run the generated executable.

```bash
cd NOBODY
cargo run --release --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_flow_sk.nbd my-program
./my-program

# Optional: view the generated assembly without linking
cargo run --release --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  native-asm sk rust/examples/native_flow_sk.nbd

# Check the executable format
readelf -h my-program
```

Expected output for the example is `8` and `-42` on separate lines. `compile-native` never overwrites an existing output path, including symlinks. Compiler/assembler/linker errors are reported instead of silently switching to Python or the bytecode VM.

## Currently supported subset

- Exactly one source-language profile per file; lowercased Nobody keywords are normalized by Rust IR first.
- Signed **64-bit** decimal integer literals, strict boolean literals and lexical variables.
- Checked `+`, `-`, `*`, unary `+`/`-`, and integer comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`). Equality of booleans and logical `not` are supported.
- `let`/localized declaration, assignment with unchanged value type, `if`/`elif`/`else`, `while`, `for` over signed `range(...)`, `break`, `continue`.
- **`print(one integer)`** with decimal output and newline via `write(1, ...)`; signed negatives, zero and `i64::MAX` are tested.
- **Top-level named functions defined before use**, accepting up to 32 positional signed i64 integer arguments and returning an explicit i64 value on every control-flow path. Chained calls to earlier functions, function-local integer/boolean variables, if/else, while and for-range inside functions, and expression-level calls are compiled directly to native x86-64 machine code. A maximum of 32 functions is supported.

Supported **native signed `for ... in range(stop)`**, `range(start, stop)` and `range(start, stop, step)` use one-time source-order integer argument evaluation. Iterator variables are lexically scoped and shadow same-named outer variables without destroying them; nested `for` and `while` loops have correct `break`/`continue` handling. Zero step exits with process status **6**, while range iterator overflow ends the range without wrapping. The i64-specific range boundary is narrower than the i128 bytecode/evaluator domain; out-of-i64 literals are rejected during compilation. Tests compare actual linked Linux ELF output to the Rust evaluator for `ai`, Slovak and English. 

Unsupported features are rejected at compile time: arbitrary iterables, recursive or forward calls, functions reading global bindings, closures and nested function definitions, implicit `null` returns, boolean or text parameters/return values, floating-point numbers, arbitrary strings, lists, null literals, first-class functions, `and/or`, typed declarations, multiple-argument print, and mutable variable type changes. Do not infer the complete language is native-compiled from this milestone.

## Independent native function call frames

The native backend implements a deliberately small private **stack-based calling convention** between functions compiled in the same program. Arguments are evaluated in source order and pushed as signed i64 values; the callee saves its own frame pointer and copies parameters into independent local stack slots. The caller reclaims argument storage after return. All returns use the signed i64 result register. Functions have no access to calling scopes or global bindings, so captures and closures do not silently pretend to work. Only earlier declared functions can be called; direct or indirect recursion is unsupported. The compiler statically rejects missing-return paths and mismatched argument counts. An unexpected fallthrough exits status **7** as a defensive guard.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/bytecode_functions_sk.nbd native-functions
./native-functions
```

Expected output: `13` then `6` on separate lines. Generated ELF files execute without the Nobody bytecode VM, Python, Cargo, Rust or libc. GNU binutils remain build-time prerequisites. Native functions are an experimental subset, **not** an external ABI or a security sandbox.

## Architecture and safety limits

- The Rust compiler directly produces x86-64 assembly and invokes **GNU `as --64`** and **`ld -m elf_x86_64`**; there is no C intermediary or embedded bytecode interpreter. No shell is used to launch toolchain commands.
- `_start` is the entry point. Generated code calls its own signed-integer printing routine and exits through Linux `sys_exit`; there is no dynamic linker or libc dependence.
- Local integer variables live in bounded stack slots. Source identifier strings never become assembler symbol names, avoiding direct source-to-assembly injection.
- Machine arithmetic traps signed overflow with process exit code **3**. A software instruction budget stops empty infinite loops with process exit code **5**. Success returns 0. These are **not** operating-system sandbox guarantees.
- Only Linux x86-64 hosts are supported by the current compiler mode; other targets are not claimed. GNU binutils must be installed when compiling; generated executables do not need Cargo or binutils to run.
- Compilation writes intermediate files in a newly created, private working directory adjacent to the output. After linking succeeds it atomically hard-links the binary into a previously nonexistent target and removes intermediates. Existing outputs are not overwritten.
- The backend currently uses native **i64**, whereas the Rust interpreter supports a larger integer domain. Source literals outside i64 and unsupported semantics are rejected, not truncated. Cross-target ABIs, debug info, general optimizations, full stack-depth proofs, portability, runtime limits enforced by the OS and self-hosting remain future tasks.

Native CI tests assemble/link actual binaries, inspect ELF headers, run them in a child process, compare supported program output to the existing Rust interpreter in `ai`/Slovak/English, test failure modes, and verify source independence. The function-call ELF subset and its failure cases passed all four CI workflows, including [Rust CI #42650743553](https://github.com/siposboris/NOBODY/actions/runs/42650743553); merged in [PR #30](https://github.com/siposboris/NOBODY/pull/30). Signed range loops and their edge cases passed [Rust CI #42641964768](https://github.com/siposboris/NOBODY/actions/runs/42641964768) after [PR #29](https://github.com/siposboris/NOBODY/pull/29) merged into main. Do not execute user-supplied untrusted Nobody source without independent OS isolation.
