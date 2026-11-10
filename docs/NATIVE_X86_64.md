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
- **`print(...)`** accepts zero or more positional arguments: signed integer or boolean expressions, **UTF-8 string literals and string-valued variables**. Values are separated by one space and terminated with a newline; `print()` outputs a blank line. Escapes `\\n`, `\\t`, `\\"`, `\\'` and `\\\\` follow the Rust evaluator. Numeric, boolean, text, separator and newline output uses Linux `write(1, ...)`, with literal bytes stored in `.rodata` as numeric `.byte` directives.
- **Top-level named functions defined before use**, accepting up to 32 positional signed i64 **or boolean** arguments (inferred conservatively from typed uses) and returning an explicit signed i64 **or boolean** value on every control-flow path. Chained calls to earlier functions, function-local integer/boolean variables, if/else, while and for-range inside functions, and expression-level calls are compiled directly to native x86-64 machine code. A maximum of 32 functions is supported.

Supported **native signed `for ... in range(stop)`**, `range(start, stop)` and `range(start, stop, step)` use one-time source-order integer argument evaluation. Iterator variables are lexically scoped and shadow same-named outer variables without destroying them; nested `for` and `while` loops have correct `break`/`continue` handling. Zero step exits with process status **6**, while range iterator overflow ends the range without wrapping. The i64-specific range boundary is narrower than the i128 bytecode/evaluator domain; out-of-i64 literals are rejected during compilation. Tests compare actual linked Linux ELF output to the Rust evaluator for `ai`, Slovak and English. 

Unsupported features are rejected at compile time: arbitrary iterables, recursive or forward calls, functions reading global bindings, closures and nested function definitions, implicit `null` returns, unannotated text function parameters, uninferrable boolean parameters, mixed integer/boolean return paths, floating-point numbers, string ordering and general dynamic string operations beyond concatenation and equality, lists, null literals, first-class functions, typed declarations and mutable variable type changes. Do not infer the complete language is native-compiled from this milestone.

## Native text arguments and return values (experimental; verified PR #44)

Functions declared before use may now accept **explicitly annotated `str` parameters** and return `str` values. Each string argument occupies two stack words (address, UTF-8 byte count) whereas `int` and `bool` occupy one; the native compiler reserves the corresponding local parameter slots, computes offsets for mixed argument layouts and reclaims the exact call-frame size. String results use `rax` and `rdx`, preserving pointer/length through nested calls and text concatenation. Wrong argument types and declared return-type mismatches reject compilation.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_call_sk.nbd native-text-call
./native-text-call
```

This is an internal ABI, **not** C interoperability, first-class functions or heap-based strings. Native recursion, closures, unknown/forward calls, implicit return `null`, and unannotated text parameter inference remain unsupported. Real ELF output is checked against the separate Rust evaluator across `ai`, Slovak and English. All four GitHub workflows passed including [Rust CI #42766777167](https://github.com/siposboris/NOBODY/actions/runs/42766777167) in [PR #44](https://github.com/siposboris/NOBODY/pull/44).

## Native text concatenation and content equality (experimental; PR #41, verified)

Nobody's native x86-64 backend now compiles `text = text + "!"`, `a == b`, and `a != b` for UTF-8 string values, not just text literals. Equality compares **actual UTF-8 bytes and lengths**, not memory addresses, and therefore works for distinct string allocations and empty texts. Concatenation allocates from a **fixed 1 MiB native text arena**, copies both byte slices in source order and produces an immutable (pointer, length) value. The resulting standalone ELF uses no libc or Rust runtime; a cumulative arena budget violation exits with **status 7** rather than allocating unchecked memory.

```bash
cargo run --locked --release --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_ops_sk.nbd native-text-ops
./native-text-ops
```

This is a bounded arena, **not** garbage collection or an unrestricted heap; repeated concatenation consumes space until process exit. Dynamic text ordering, unannotated text parameters, full Unicode collation, interpolation and general collection support remain unimplemented. Real ELF output is compared byte-for-byte with the Rust evaluator in `ai`, `sk`, `en`; all four GitHub workflows passed, including [Rust CI #42754693569](https://github.com/siposboris/NOBODY/actions/runs/42754693569) ([PR #41](https://github.com/siposboris/NOBODY/pull/41)).

## Native UTF-8 text bindings (experimental, PR #40)

The standalone ELF backend now supports `let text = "Hello"`, `let copy = text`, `text = "New"`, `print(text, copy)` and lexically shadowed text bindings in nested loops, conditionals and function-local bodies. A string expression provides **(UTF-8 data pointer, byte length)** using x86-64 registers; the compiler stores both values in two adjacent stack words. All source text remains immutable in ELF `.rodata`, so reassignment changes the pointer/length only. UTF-8 byte lengths (not character counts) are passed directly to Linux `write`; empty and Unicode text remain correct.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_variables_sk.nbd native-vars
./native-vars
```

**Unsupported:** string ordering, unannotated native string parameters, typed declarations, changing a variable's type and unrestricted heap-allocated strings. Operations that cannot be compiled correctly fail rather than silently performing pointer equality. New integration tests run actual ELF files and compare stdout byte-for-byte with the Rust evaluator for `ai`, `sk`, `en` (including loop, function, shadowing and copy/reassignment). All four checks passed, including [Rust CI #42753533095](https://github.com/siposboris/NOBODY/actions/runs/42753533095), merged through [PR #40](https://github.com/siposboris/NOBODY/pull/40).

## Native UTF-8 output and mixed arguments

The standalone ELF backend now compiles **direct text literals** without Python, Rust runtime, libc or Nobody VM inside the resulting program. The string decoder is shared with the Rust AST evaluator; strings are embedded in read-only data with numeric byte directives, so assembler characters or directives inside source strings cannot become machine-code instructions. The compiler handles `print()`, `print("Hello")`, and mixed values such as `print("step", n, true)`, including text within loops and functions.

```bash
cargo run --release --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_print_sk.nbd native-text
./native-text
```

**Limit:** string ordering and unannotated native string parameters are still rejected; text variables, concatenation and equality are now supported in the native subset. This is an output feature, not a complete native string type. The new ELF output is checked against the independent Rust evaluator for `ai`, `sk` and `en`, including Unicode, escaped literals, blank lines, mixed types and loops. All four workflows passed, including [Rust CI #42697321428](https://github.com/siposboris/NOBODY/actions/runs/42697321428) in [PR #39](https://github.com/siposboris/NOBODY/pull/39).

## Independent native function call frames

The native backend implements a deliberately small private **stack-based calling convention** between functions compiled in the same program. Arguments are evaluated in source order and pushed as signed i64 values; the callee saves its own frame pointer and copies parameters into independent local stack slots. The caller reclaims argument storage after return. All returns use the signed i64 result register. Functions have no access to calling scopes or global bindings, so captures and closures do not silently pretend to work. Only earlier declared functions can be called; direct or indirect recursion is unsupported. The compiler statically rejects missing-return paths and mismatched argument counts. An unexpected fallthrough exits status **7** as a defensive guard.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/bytecode_functions_sk.nbd native-functions
./native-functions
```

Expected output: `13` then `6` on separate lines. Generated ELF files execute without the Nobody bytecode VM, Python, Cargo, Rust or libc. GNU binutils remain build-time prerequisites. Native functions are an experimental subset, **not** an external ABI or a security sandbox.

## Lazy boolean machine code and typed results (experimental)

The native backend now emits true **short-circuit** `and`/`or`: it checks the left boolean and jumps around the entire right expression when the result is already determined. The skipped expression is never executed, even when it contains a function call with output side effects or overflowing arithmetic. Both operands still must be boolean-typed at compile time. Native `print` now accepts zero or more integer/boolean expressions and direct UTF-8 string literals, joining arguments with spaces and a final newline via Linux syscalls.

Functions may now have signed i64 **or inferred boolean parameters**, and explicitly return a **boolean or i64**. Every return statement in the same function must have the same type; unsupported/mixed returns fail compilation. The compiler tracks the known return type for calls to earlier-declared functions, so predicates can be composed in conditions. Parameter types are inferred only from direct, unambiguous uses such as `if flag`, `not flag`, boolean `and`/`or`, arithmetic, comparisons or arguments to earlier known functions. Unconstrained parameters default to i64; passing `bool` to such a parameter is rejected, and conflicting constraints fail compilation. This is deliberately **not** flow-sensitive whole-program type inference. These changes do not implement implicit null returns, recursion, fully generic functions, complete static typing, arbitrary values or a general purpose ABI.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_bool_sk.nbd native-boolean
./native-boolean
```

Expected output is `true`, `false`, `false`, `true`, `77` on separate lines.

## Explicit function signature annotations

Native Nobody now supports optional **source-level primitive function signatures** for the existing `int` and `bool` machine types:

```text
fn choose(flag: bool, a: int, b: int): int {
    if flag { return a } else { return b }
}
```

One source file still uses exactly one selected syntax profile. Slovak spelling is `funkcia choose(flag: logicke, a: cele, b: cele): cele`; the parser converts types to canonical names in Rust IR. Both parameter and return annotations are optional. The native backend verifies calls and return types against declared annotations, and verifies that any inferred constraints agree. Untyped functions still use conservative parameter inference. All four workflows passed, including [Rust CI #42660895804](https://github.com/siposboris/NOBODY/actions/runs/42660895804) with actual native ELF differential tests; merged in [PR #33](https://github.com/siposboris/NOBODY/pull/33). This is a partial implementation, not a completed typed-IR or self-hosting roadmap criterion.

Explicit `int` (signed 64-bit in native ELF), `bool` and `str` signatures compile to machine code. `str` parameters use a private two-word pointer/byte-length calling convention and string results return in `rax`/`rdx`. `float` remains rejected by this native subset. The separate Rust evaluator can validate supported runtime value types; typed function declarations are not yet supported by experimental bytecode or AST JSON v1 output because those paths would otherwise drop type metadata.

## Conservative boolean parameter inference

Nobody's existing function declaration syntax has no explicit parameter-type annotations. The native compiler can now infer parameters as `bool` from direct boolean contexts (`if`, `while`, `not`, `and`/`or`), boolean-literal equality or passing the parameter to an earlier compiled boolean-parameter function. Integer contexts constrain the parameter to signed i64. Lexically shadowing a parameter with a local variable does not reclassify the original parameter.

This intentionally accepts only a **conservative subset**. A parameter with no resolvable type constraints defaults to i64, and a call with an incompatible boolean argument fails compilation rather than implicitly converting values. Conflicting type constraints also fail. Inference through intermediate aliases, complex flow-sensitive assignments, annotations, generic specialization and full type-system soundness remain future work.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_bool_params_sk.nbd native-bool-params
./native-bool-params
```

The ELF still has no Python, libc, Rust or Nobody VM runtime dependency. This conservative type-inference slice passed all four GitHub checks, including [Rust CI #42654797725](https://github.com/siposboris/NOBODY/actions/runs/42654797725), and merged via [PR #32](https://github.com/siposboris/NOBODY/pull/32).

## Architecture and safety limits

- The Rust compiler directly produces x86-64 assembly and invokes **GNU `as --64`** and **`ld -m elf_x86_64`**; there is no C intermediary or embedded bytecode interpreter. No shell is used to launch toolchain commands.
- `_start` is the entry point. Generated code calls its own signed-integer printing routine and exits through Linux `sys_exit`; there is no dynamic linker or libc dependence.
- Local integer variables live in bounded stack slots. Source identifier strings never become assembler symbol names, avoiding direct source-to-assembly injection.
- Machine arithmetic traps signed overflow with process exit code **3**. A software instruction budget stops empty infinite loops with process exit code **5**. Success returns 0. These are **not** operating-system sandbox guarantees.
- Only Linux x86-64 hosts are supported by the current compiler mode; other targets are not claimed. GNU binutils must be installed when compiling; generated executables do not need Cargo or binutils to run.
- Compilation writes intermediate files in a newly created, private working directory adjacent to the output. After linking succeeds it atomically hard-links the binary into a previously nonexistent target and removes intermediates. Existing outputs are not overwritten.
- The backend currently uses native **i64**, whereas the Rust interpreter supports a larger integer domain. Source literals outside i64 and unsupported semantics are rejected, not truncated. Cross-target ABIs, debug info, general optimizations, full stack-depth proofs, portability, runtime limits enforced by the OS and self-hosting remain future tasks.

Native CI tests assemble/link actual binaries, inspect ELF headers, run them in a child process, compare supported program output to the existing Rust interpreter in `ai`/Slovak/English, test failure modes, and verify source independence. Native lazy booleans, boolean printing, typed i64/bool returns and skipped side effects passed [Rust CI #42652961905](https://github.com/siposboris/NOBODY/actions/runs/42652961905), merged in [PR #31](https://github.com/siposboris/NOBODY/pull/31). The function-call ELF subset and its failure cases passed all four CI workflows, including [Rust CI #42650743553](https://github.com/siposboris/NOBODY/actions/runs/42650743553); merged in [PR #30](https://github.com/siposboris/NOBODY/pull/30). Signed range loops and their edge cases passed [Rust CI #42641964768](https://github.com/siposboris/NOBODY/actions/runs/42641964768) after [PR #29](https://github.com/siposboris/NOBODY/pull/29) merged into main. Do not execute user-supplied untrusted Nobody source without independent OS isolation.
