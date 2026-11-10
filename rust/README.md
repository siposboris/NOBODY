# Nobody — Rust compiler frontend (experimental)

This is the Rust-first migration of the Nobody compiler frontend. Its goal is
eventually to replace the Python interpreter/compiler and later be rewritten in
Nobody itself. **Parsing does not mean running or native compilation.**

## Read-only Nobody formatter verification

`nobody-rs format-check PROFILE FILE.nbd` checks whether source matches the
canonical Rust `format-source` output, returning success on exact match or
diagnostic N5002 with a nonzero exit status on drift. It never modifies files.
It operates on one selected syntax profile at a time and is useful in CI.
The formatter emits AST-based source: original comments and whitespace are
not preserved, so review generated code before overwriting anything.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody-rs -- format-source sk rust/examples/native_text_ops_sk.nbd > clean.nbd
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody-rs -- format-check sk clean.nbd
```

## Conservative static boolean and text-operator diagnostics

Rust's `typecheck PROFILE FILE.nbd` rejects **certain** incompatible operations
before execution: text ordering with `<`/`>=`, mixed-type `and`/`or` and
non-boolean `not`, using N1002. It accepts text `+` and `==`/`!=` and
does not reject operands whose type is unknown. These checks use the existing
IR type inference and support `ai`, `sk`, and `en`. This is an incomplete
conservative checker, not proof of full type safety.

## Native typed UTF-8 string function call ABI (experimental, PR #44)

The Linux x86-64 native compiler now supports text arguments and text return values
with explicit `str` annotations, for example `fn identity(text: str): str { return text }`
and Slovak `funkcia identity(text: text): text { vrat text }`. Every text parameter
is passed as **two stack words** (UTF-8 address and byte length); the callee copies
them into its own local frame. Text results return in `rax`/`rdx`. Mixed `int`,
`bool` and `str` parameters, nested calls, text concatenation and returns on
both conditional paths are supported. Wrong argument/return types fail at
compile time, with real ELF-vs-Rust-evaluator parity tests for `ai`, `sk`, `en`.
The string ABI is internal to Nobody-generated functions, not a stable foreign
function ABI. Text parameter inference without annotations, recursion, closures
and cross-platform native targets remain unsupported.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_call_sk.nbd native-text-call
./native-text-call
```

## Native UTF-8 text operations (experimental)

The standalone ELF compiler now handles `let combined = left + right`, `combined == expected`, and `combined != other` for UTF-8 text variables. A fixed 1 MiB process-lifetime arena stores joined text without libc; allocation exhaustion terminates with exit status `7`. Equality compares all bytes, not addresses. Actual ELF/evaluator parity tests cover text within loops/functions and non-ASCII text across `ai`, `sk`, and `en`. String ordering, unannotated text parameter inference, garbage collection and unrestricted memory allocation remain out of scope.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_ops_sk.nbd native-text-ops
./native-text-ops
```

## Mutable UTF-8 text variables in real native ELF (experimental)

The x86-64 compiler now allocates **two stack words per local text binding** (read-only UTF-8 data pointer plus byte length). It supports declarations, copying, reassignment, lexical shadowing and printing stored text inside loops and function-local bodies. The actual ELF must produce identical output to the Rust interpreter for the `ai`, `sk` and `en` acceptance fixtures. This feature does **not** add heap allocation: source string contents live in the ELF read-only data segment, while variables hold mutable pointer/length pairs.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_variables_sk.nbd text-vars
./text-vars
```

Text concatenation (`+`) and bytewise equality/inequality (`==` / `!=`) now compile through a bounded 1 MiB arena and a native byte comparison routine; type changes and unannotated native text parameter inference remain unsupported. Rust evaluator and ELF may differ outside the documented subset.

## Real standalone UTF-8 text output in native ELF

The Rust x86-64 backend now compiles `print("Hello, Nobody!", 42, true)` to direct Linux syscall output in a standalone ELF. Unicode, escaped text, blank lines and mixed arguments are verified against the Rust evaluator in `ai`, `sk` and `en`. The compiler embeds sanitized numeric string bytes in read-only data. Native UTF-8 text variables can now be copied and reassigned, including in loops and function-local scopes. Native text concatenation and equality are now supported. Ordering and unannotated native text parameter inference remain unsupported.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_text_print_sk.nbd native-text
./native-text
```

## First real native Linux x86-64 backend (experimental)

The Rust CLI now supports **real machine-code compilation** for a small signed-i64 Nobody subset, separate from the bytecode VM:

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- compile-native sk rust/examples/native_flow_sk.nbd my-program
./my-program
```

The Rust compiler generates x86-64 assembly and uses GNU `as` + `ld` (Fedora package `binutils`); the resulting ELF executable runs using Linux syscalls, **without** the Nobody VM, Python, Rust runtime or libc. `native-asm PROFILE FILE.nbd` displays the generated assembly. Currently supported: i64 arithmetic and variables, boolean comparisons, if/else, while, signed `for ... in range(...)` (1–3 integer arguments), break/continue, and **UTF-8 literal/mixed-argument print**, lazy `and`/`or`, and boolean predicate return values. The native compiler evaluates range bounds once, supports nested loops and stops on iterator overflow; zero step exits with status 6. The backend also compiles **earlier-declared named functions with up to 32 integer parameters and explicit i64 or boolean return values on all paths**, including calls to earlier functions; separate stack frames isolate locals. It rejects recursion, forward calls, implicit null, captures, text/function arguments, conflicting parameter or return types, strings and full i128 semantics. Boolean parameters are supported only when their type can be inferred from direct typed use. See [native backend scope and security limits](../docs/NATIVE_X86_64.md); this is not a complete compiler or self-hosting.

## Deterministic Nobody source emission (experimental V2)

The Rust-only `ast_source` module can now emit readable Nobody code from an imported, validated `nobody.ast.v2` document. It supports the current Rust parser subset: declarations, primitive type annotations, expressions, functions, `if/elif/else`, `while`, `for`, and `break/continue` in **one selected syntax language per file**. Typed function signatures and localized keywords are preserved. Every emitted file is **parsed back in Rust and compared with the original in-memory syntax AST**; mismatches fail with N5001. Original comments, spacing and source line locations are not preserved, and the importer is not executed or compiled.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- ast-json-v2 sk rust/examples/native_typed_signatures_sk.nbd > program.json
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- emit-ast-v2 sk program.json > restored.nbd
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- compare-ast-v2 sk rust/examples/native_typed_signatures_sk.nbd program.json
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- format-source sk rust/examples/native_typed_signatures_sk.nbd
```

This is **syntax-tree parity**, not a proof of runtime or semantic equivalence for all 40 profiles, full lexer safety, or whole-program compiler correctness. The Python reference is still retained until all migration gates pass.

## Imported AST to Rust parser AST (experimental V2)

The new `lower_ast_json_v2(json, profile)` converts validated JSON into the native Rust `Program`/`Statement`/`Expr` tree without executing it. It verifies names and decimal literals through Rust's lexer, rejects profile-reserved keywords, retains canonical function annotations and applies the Rust control-flow context validator. `same_ast_json_v2(source, profile, json)` compares it with a freshly parsed Nobody source file; the new CLI command reports MATCH/DIFFERENT.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- ast-json-v2 sk rust/examples/native_typed_signatures_sk.nbd > /tmp/nobody-typed.json
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- compare-ast-v2 sk rust/examples/native_typed_signatures_sk.nbd /tmp/nobody-typed.json
```

This is an **unspanned structural AST comparison**, not verification of matching source positions or complete language semantics. Imported JSON still cannot be run or compiled to bytecode/native ELF. Its validated syntax subset can now be re-emitted as Nobody source; execution and full semantic parity remain unverified. It is not a sandbox or a proof of full Python parity.

## Typed AST JSON import and validation (experimental V2)

A bounded Rust-only importer now reads exported `nobody.ast.v2` JSON into a typed structural AST tree; it checks schema, child kinds, function signatures, Unicode string escapes, duplicate fields and size/depth/node limits. It rejects malformed documents with N5001 and **never evaluates imported code**. `normalize-ast-json-v2` deterministically reserializes a validated JSON tree; this is **JSON structure round-trip**, not Nobody source/semantic round-trip. No Python is used for these commands. Full AST-to-executable-tree import, full source re-emission, stable format and full Python parity are still open.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- ast-json-v2 ai rust/examples/native_typed_signatures_ai.nbd > program.json
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- verify-ast-json-v2 program.json
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- normalize-ast-json-v2 program.json
```

## Typed AST JSON export (experimental V2)

`ast-json-v2 PROFILE FILE.nbd` exports deterministic `nobody.ast.v2` JSON using the Rust parser alone. A function node's `value` is `[name, [parameter_names], [parameter_types], return_type]`, using canonical `int`/`bool`/`float`/`str` type strings or JSON `null` for an absent annotation. The positional parameter-type array always matches the parameter-name array. Multiline nodes retain source line numbers when the conservative location visitor can map them; unsupported shapes fail closed. The existing `ast-json` V1 command and its untyped format are unchanged. **V2 JSON import, normalization, non-executing Rust syntax lowering and partial source re-emission are available; a stable ABI and complete parity remain open.**

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- ast-json-v2 sk rust/examples/native_typed_signatures_sk.nbd
```

## Typed functions in bytecode VM and portable `.nbc` v2

The Rust bytecode VM now enforces annotated `int`/`bool` function parameters and returns at runtime. Both source execution and persisted binary execution support this subset, with a deterministic v2 file format and backward-compatible decoding of untyped v1 bytecode. Unsupported `float`/`str` function signatures fail explicitly. `nobody.ast.v1` JSON still rejects typed function signatures rather than losing them; the opt-in Rust `ast-json-v2` command exports these signatures without using Python.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- compile-bytecode sk rust/examples/native_typed_signatures_sk.nbd typed.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- verify-bytecode typed.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-compiled typed.nbc
```

See [the binary format specification](../docs/BYTECODE_FORMAT.md). The v2 wire format is experimental, not a stable ABI, native executable or OS sandbox.

## Explicit function signatures (experimental)

Function parameters and returns may now be annotated with localized primitive types, for example `fn identity(flag: bool): bool` or Slovak `funkcia identity(flag: logicke): logicke`. The parser carries normalized `int`/`bool`/`float`/`str` annotations into canonical Rust IR. The native Linux compiler currently supports explicit `int`, `bool` and `str` signatures, including argument and return mismatch rejection; the separate Rust evaluator checks annotations at runtime. Legacy untyped functions remain supported.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_typed_signatures_sk.nbd native-typed
./native-typed
```

**Compatibility boundary:** the bytecode VM now supports `int`/`bool` annotated signatures through `.nbc` v2; `nobody.ast.v1` JSON export deliberately rejects them; `nobody.ast.v2` export now preserves them, with Rust-only structural JSON import and syntax-AST lowering now available; executable import and source re-emission remain planned. Source-level `float` function signatures are not yet supported by native ELF; `str` works with explicit annotations; the Rust type checker is still partial.

## Native boolean function parameters (conservative subset)

Native x86-64 functions can now accept `bool` as well as signed i64 arguments. Nobody has no explicit parameter-type syntax, so the compiler uses direct typed contexts (`if flag`, `not flag`, `and`/`or`, numeric operators, calls to known functions) to infer a fixed signature. Shadowed names are handled lexically. Ambiguous parameters default to i64, type conflicts fail explicitly, and there is **no** implicit truthiness or generic specialization.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_bool_params_sk.nbd native-boolean-params
./native-boolean-params
```

Actual ELF differential tests cover `ai`, `sk` and `en`, nested loops/calls and type mismatch rejection. This does not yet support general flow-sensitive type inference, arbitrary types, recursion or self-hosting.

## Native lazy booleans and boolean-returning functions (experimental)

The Linux ELF backend now supports true lazy boolean `and`/`or` with compile-time type checking. The right operand is skipped when the left decides the result; differential ELF fixtures verify this with arithmetic-overflow and function-side-effect cases. Native `print` now accepts zero or more signed i64/boolean expressions or direct UTF-8 string literals, joining them with spaces and a newline. String-valued variables, concatenation and equality are now supported in a bounded native ELF subset. Functions accept i64 arguments and may explicitly return an i64 or bool, but all return paths must agree.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/native_bool_sk.nbd native-bool
./native-bool
```

This is still a restricted native subset: boolean parameters require conservative type inference; arbitrary text, closures, recursion, full i128 and cross-platform support remain unsupported. See [native backend limits](../docs/NATIVE_X86_64.md).

## Native Linux function call/return subset (experimental)

Standalone x86-64 ELF output now supports top-level named functions defined before their call sites, integer-only parameters and explicit integer/boolean returns, along with local statements, conditions and loops inside function bodies. This does **not** mean full function parity: no recursion, closures, global capture, implicit null returns or arbitrary value types. The compiler uses its own stack-based calling convention, and function invocation still consumes a shared software step budget. Verified via actual ELF execution and differential Rust evaluator fixtures for `ai`, `sk` and `en`.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- \
  compile-native sk rust/examples/bytecode_functions_sk.nbd native-function-demo
./native-function-demo
```

## Experimental portable `.nbc` files

The native Rust CLI now supports an explicit compiled-artifact workflow:

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- compile-bytecode ai rust/examples/bytecode_functions_ai.nbd program.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- verify-bytecode program.nbc
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-compiled program.nbc
```

`run-compiled` reads binary `.nbc` bytes directly and does **not** parse source or call Python. `compile-bytecode` does **not** overwrite existing output files. The experimental v1 little-endian format has deterministic encoding, a bounded validating decoder, checksum for accidental corruption and the VM's existing control-flow verifier. Equivalent supported `ai`, Slovak and English programs produce byte-identical binaries in tests. The schema, limitations and opcode table are in [Bytecode Format](../docs/BYTECODE_FORMAT.md). **No stable cross-version ABI, native executable, cryptographic verification or sandbox is implied.**

## Named functions in Rust bytecode (experimental)

The native bytecode compiler now emits separate, verifier-checked function instruction streams and `Call`/`Return` operations. Supported: top-level functions declared before use, positional parameters, explicit return expressions and implicit `null`, local variables, conditionals/loops in function bodies, and calls to earlier named functions. Each invocation has isolated local slots and an operand stack, with shared finite CPU-step/output counters. The verifier rejects invalid function IDs, argument counts, return stack shape and recursive/forward bytecode calls.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode sk rust/examples/bytecode_functions_sk.nbd
```

**Still missing:** global-variable access inside functions, closures, nested definitions, recursive/forward function references, first-class values, portable serialized bytecode and native machine-code output. Neither the bytecode VM nor its software limits is a security sandbox.

## Signed `for range` bytecode VM (experimental)

The independent native VM now supports `for variable in range(stop)`, `range(start, stop)`, and `range(start, stop, step)` for integer arguments with nonzero signed step. Arguments are evaluated once. Nested `for` and `while` loops maintain separate `break` and `continue` targets, loop variables use a lexical scope, zero step gives N4006, and checked i128 overflow terminates iteration without wrapping. Static verification validates slot initialization on range-control edges and jump target bounds. Unsupported iterables still fail explicitly; Python is not part of Rust execution.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode sk rust/examples/bytecode_for_sk.nbd
```

The bytecode format remains in memory only, incomplete and experimental. This does not provide native machine-code output, arbitrary iterables, function calls, full type coverage or a security sandbox.

## Bytecode VM control flow (experimental)

The separate Python-free bytecode runtime now handles `if/elif/else`, `while`, `break`, `continue`, comparisons, boolean literals and `not`. Branch bodies have lexically scoped variable slots; shadowing a name inside an `if` does not overwrite the outer declaration. A control-flow verifier checks every reachable edge, rejecting stack-height disagreement, uninitialized slots, invalid jumps and nonempty terminal stacks. Runtime software step budgets also stop empty infinite loops. The compiled VM and the pre-existing Rust AST evaluator are compared on `ai`, Slovak and English programs in CI.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode ai rust/examples/bytecode_control_ai.nbd
```

These capabilities remain **experimental**. The bytecode is in memory, not yet a stable portable format; arbitrary iterables, functions, short-circuit `and/or`, floats, strings, native output and complete self-hosting remain unimplemented. Resource counters do not form a security sandbox.

## Phase 4: first independent bytecode compiler + VM (experimental)

The Rust binary now exposes a **separate bytecode execution path** from the older tree-walking evaluator. This is the first concrete Phase 4 foundation, not an assertion that Nobody has a finished compiler or native backend.

```bash
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- bytecode ai rust/examples/eval_basic_ai.nbd
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run-bytecode sk rust/examples/eval_basic_sk.nbd
```

The compiler lowers the canonical Nobody IR into an **in-memory** versioned instruction vector, validates stack depth, slot initialization and termination, and executes with checked signed 128-bit arithmetic and bounded steps/output. Current support: **integer literals, `let`, assignment, `+`, `-`, `*`, unary sign, unshadowed `print`, and arithmetic expression statements**. The Python-free native CLI returns numbered errors for unsupported constructs. The AST evaluator remains independent to compare identical supported program results in unit and CLI tests across English, Slovak and `ai` profiles.

**Still missing:** portable bytecode file format, complete control flow including for-range, function calls, floats, text values, generalized types, runtime sandboxing, a linker and machine-code generation. Do not feed untrusted programs to the VM.

## Current source coverage

- **39 experimental human-language keyword profiles** plus a redesigned
  canonical **`ai`** syntax profile (40 total) in `src/profiles.rs`.
- UTF-8 lexical scanning with source byte offsets and common combining marks
  in `src/lib.rs`.
- AST parser supporting declarations, assignments, expressions, calls,
  functions, conditions, loops, returns and nested blocks in `src/syntax.rs`.
- Context validation for return/break/continue in `src/validate.rs`.
- A token-preserving converter for old `ai_` source in `src/ai_migration.rs`.
- Stable Nxxxx diagnostic categories for known source problems.

**Verified:** Rust compilation, tests, formatting, Clippy and 40×40 representative
IR profile matrix passed GitHub Actions. Unicode N2008 normalization checks
passed in [Rust CI #42503823825](https://github.com/siposboris/NOBODY/actions/runs/42503823825).
This is not proof of full frontend or runtime parity: native-speaker reviews,
Unicode confusables beyond the N2007 subset and full grammar remain open.
The Python reference remains until complete behavioral parity is demonstrated.

**Dependency change:** NFC validation uses the pinned `unicode-normalization = "=0.1.24"`
crate. The Rust frontend is no longer third-party-dependency-free, although it
still does not invoke Python or online services at runtime. A checked-in `Cargo.lock` now pins the entire resolved dependency graph, and
CI verifies it with `--locked`. This does not yet constitute a tagged,
reproducible release artifact or supply-chain audit.

## Rust evaluator for-range slice (experimental)

The opt-in `nobody-rs run PROFILE FILE` command now supports `for` loops over the **unshadowed** `range(...)` built-in with 1–3 integer arguments: `range(stop)`, `range(start, stop)` and `range(start, stop, step)` (both positive and negative steps). Arguments are evaluated once, and iteration variables live in a fresh lexical scope for each iteration. `break` and `continue` work inside nested conditions and loops. Invalid zero steps report **N4006**. Huge loops consume the existing N4005 step budget.

Other iterables, lists, arbitrary function calls and generators are **not** implemented. Integer steps use checked i128 arithmetic and stop cleanly at numeric limits; the runtime is not a sandbox. Python reference behavior is not yet fully parity-tested.

## Rust evaluator while-loop slice (experimental)

`nobody-rs run` now executes `while` loops with strict boolean conditions,
lexically scoped bodies, and `break`/`continue` (including when nested inside
`if` blocks). An empty infinite loop cannot run without a budget error: each
iteration consumes the same finite evaluator step budget. A nested `break`
terminates only the innermost loop. The language profile is chosen once per
source file, and identical AI/Slovak examples have regression tests.

These counters are **software checks, not hard limits**: untrusted Nobody
programs must not be executed. `for`, functions, recursion and full
Python-reference runtime parity are still unsupported.

## Rust evaluator conditional slice (experimental)

`nobody-rs run` now supports conditional blocks (`if`, `else if`, `else`), comparisons of integers (`<`, `>`, `<=`, `>=`), equality and inequality, boolean `not`, `and`, and `or`. Boolean `and`/`or` short-circuit without evaluating the skipped expression. Branch-local declarations are lexical: they do not leak outside the branch, while assignment can update a visible outer binding.

Conditions **currently require boolean values**. Unlike Python, using `if 1` fails with N4004 instead of applying implicit truthiness. Function calls (other than `print`), loops, and typed runtime declarations remain unsupported; no Python reference parity or sandbox guarantees are claimed. Each source file still uses exactly one selected syntax profile.

## 0.39 development slice — explicitly opt-in Rust evaluator (experimental)

The new `nobody-rs run PROFILE FILE` command executes a **small** Nobody IR subset directly in Rust, not Python. Implemented initially: `let` declarations without type annotations, assignment, integer `+`, `-`, `*`, floor division `//`, remainder `%`, simple text concatenation, boolean/null literals, and unshadowed `print`. The evaluator does not support functions, loops, conditionals, user-defined calls, floating-point execution or true division `/` with a nonzero divisor. Unsupported constructs fail with N4004 instead of silently generating wrong results. Runtime errors N4001–N4005 are documented in [the error catalogue](../docs/ERROR_CODES.md).

```bash
cargo run --locked --manifest-path rust/Cargo.toml -- run ai rust/examples/eval_basic_ai.nbd
cargo run --locked --manifest-path rust/Cargo.toml -- run sk rust/examples/eval_basic_sk.nbd
```

Both demonstration programs should print `9` and then `8` on separate lines. This is **not** the full 0.39 milestone: full typing, lexical closures, evaluator parity and runtime security remain open. Its step and output checks are software budgets, not hard OS resource isolation. **Never execute untrusted Nobody programs**. Existing `lex`, `check`, `parse`, `validate` and `typecheck` commands do not execute source code.

## Build and check

With the stable Rust toolchain installed:

```bash
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path rust/Cargo.toml -- list-languages
cargo run --manifest-path rust/Cargo.toml -- parse sk examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai examples/hello_ai.nbd
```

### Redefined `ai` syntax

```text
fn twice(x) {
    return x * 2
}

let n: int = 2
if n > 1 { print(twice(n)) }
```

The keywords resemble widely used programming languages rather than
inventing new `ai_` terms. `ai` is still an ordinary Nobody **syntax
profile**, with no special runtime or autonomous AI capability. The design
has **not** been benchmarked for LLM token efficiency or coding accuracy.
The canonical AI dictionary has `let`, `fn`, `if`, `else`, `while`,
`for`, `in`, `return`, `true`, `false`, `null`, `and`, `or`,
`not`, `print`, etc. The Rust parser also recognizes `else if` as an
alternative to `elif` in the AI profile.

### Convert legacy AI source safely

```bash
cargo run --manifest-path rust/Cargo.toml -- migrate-ai examples/hello_ai_legacy.nbd > new_ai.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai new_ai.nbd
```

The command prints the transformed program to **stdout**; it does not
overwrite the original. It changes **known keyword tokens only**, leaving
comments, strings and unrelated identifiers unchanged. Using a recognized
legacy keyword without conversion produces **N3010**, documented in
[`ERROR_CODES.md`](../docs/ERROR_CODES.md).

## Commands

| CLI command | Purpose |
|---|---|
| `lex FILE` | Print source tokens with line/column information |
| `check FILE` | **Lexer-only** checks |
| `parse PROFILE FILE` | Create an experimental AST, supports all 40 profile tables |
| `validate PROFILE FILE` | AST parse plus return/break/continue context checks |
| `list-languages` | Display implemented Rust keyword profile codes |
| `migrate-ai FILE` | Print converted legacy-prefixed AI source |

## Still outstanding

- Formal Unicode normalization and confusable detection, native-speaker reviews,
  full foreign-keyword collisions and source-span tracking for AST statements.
- Complete grammar, static typing, true function/runtime behavior,
  source-to-source translation, bytecode, native backend and self-hosting.
- Successful reproducible Rust compilation and CI, differential comparison with
  Python behavior and subsequent safe removal of Python modules.

[Canonical English roadmap](../docs/ROADMAP.md) ·
[AI syntax specification](../docs/AI_SYNTAX.md) ·
[Migration gates](../docs/RUST_MIGRATION.md).

## 0.27 — cross-language keyword collision protection

The Rust `translate FROM TO FILE` command now reports **N3011** if a legal source identifier would become a target-profile reserved keyword. For example, Slovak `nech fn = 3` cannot translate to AI `let fn = 3`. Rename `fn` in the source to a name not reserved in the destination profile, then retry. See [translation-safety specification](../docs/TRANSLATION_SAFETY.md) and [error catalogue](../docs/ERROR_CODES.md).

The check preserves strings/comments and reports source line/column, but full scope-aware renaming and semantic parity are still planned. The Rust build and test results are not yet independently verified.

## 0.30 — Rust static type checks

`cargo run --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/typecheck_good_ai.nbd` checks known primitive types and declarations without executing the program. `typecheck_bad_ai.nbd` intentionally fails with documented **N1001**. Unknown expressions are deferred; this is not a sound whole-program checker. The source is committed but Cargo tests/CI remain **unverified**. See [error catalogue](../docs/ERROR_CODES.md).

## 0.31 — lexical scope diagnostics

The Rust `typecheck` command now also reports **N1003** (assignment to undeclared name) and **N1004** (duplicate declaration or repeated function parameter). Nested blocks may shadow enclosing names. Source and six additional tests are committed; Rust compilation/CI results remain unverified. IR statement source spans, imports and full closure semantics are not yet implemented.

## 0.32 — known function arity

The Rust `typecheck` command also checks direct calls to lexically known functions for argument-count mismatch (**N1005**). Example: `fn add(a,b) { return a+b }` followed by `add(1)` is rejected with an actionable repair. Recursive calls are checked, but dynamic and unknown callees are deferred. Six unit and two CLI tests are authored; Cargo/CI success remains unverified.

## 0.33 — regression hardening

The Rust profile tests now use lowercase canonical semantic keys; the IR type checker invalidates previously known function arity when that variable is reassigned. Two regression tests were added. CI now runs `cargo fmt --all -- --check` without mutating files. **Formatting, compilation and tests have not been independently verified.**

## 0.34 — verification gate

Run `bash scripts/verify-rust.sh` from the repository root. The script checks toolchain prerequisites, runs `cargo fmt --check`, `cargo test --all-targets`, strict `cargo clippy`, and successful/failing CLI smoke cases without changing source files. The Rust GitHub Actions workflow also supports manual dispatch. **This is a verification harness, not proof that the crate compiles**: the authoring environment lacks Cargo and the CI result remains unverified.

## 0.35 — arithmetic type checks

The Rust `typecheck` command now also reports **N1002** for definitely incompatible primitive arithmetic, such as `let x = "text" - 1`. Six unit tests were authored. Unknown operand types remain deferred; source spans and complete runtime type semantics remain future work. **Compilation/CI unverified.**

## 0.36 — localized primitive types

IR lowering maps localized primitive annotations to canonical `int`, `float`, `str`, `bool` so equivalent typed declarations in different syntax profiles produce comparable IR. Regression tests cover Slovak annotations and translated typed declarations across 40 profiles. This is source-level progress; **Rust compilation and CI success remain unverified**.

## 0.37 — unary numeric operands

Rust `typecheck` now rejects known nonnumeric values with unary `+` and `-` as N1006. Unknown operands remain deferred. Five unit tests were authored; GitHub Actions triggers on verifier script edits. **Compilation and CI remain unverified.**

## 0.38 — control-flow arity tracking

Rust type checking now conservatively drops known function arities after potential reassignment in `if` and loop bodies. Local declarations and loop iterator shadowing do not replace outer signatures. Four regression tests were authored. **This is source-level work; successful Rust compilation and CI are not yet verified.**

## Locked Rust dependencies — verified

The repository commits `rust/Cargo.lock` produced by Cargo on GitHub Actions. The
verification script and workflow run `cargo metadata --locked`, `cargo test --locked`,
`cargo clippy --locked` and `cargo run --locked`. This prevents CI from
silently updating resolved dependency versions. [GitHub Actions run
#42504352399](https://github.com/siposboris/NOBODY/actions/runs/42504352399)
passed all Rust gates using the checked-in lockfile. The separate Python and
lexer workflows also passed on the same code commit. This is an input-locking
guarantee, not bit-for-bit reproducible native binary output.
