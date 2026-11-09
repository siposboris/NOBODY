# Rust → Nobody migration contract

**Decision (2026-11-08):** Rust is the temporary production-compiler implementation language. The long-term goal is a Nobody compiler and runtime written in Nobody itself. Python is legacy reference material, not the target runtime.

## Hard rules

1. **New core implementation work goes to Rust**, not to newly written Python modules. Existing Python code is temporarily retained for comparison and to prevent data loss.
2. Port modules **by observable behavior**: preserve documented syntax (39 human profiles + `ai`), native scripts, logical Unicode order, Nxxxx error numbers, type semantics and source-to-source translation.
3. Each component must pass `cargo test`, an integration fixture suite and a Python-vs-Rust differential check for relevant legacy examples *before* the Python equivalent is deleted. Differential tests may temporarily invoke Python in CI, but shipping Rust binaries must not depend on it.
4. Mark a component **Rust-replaced** only when every feature it replaces is implemented, tests agree and the Rust CLI exposes an equivalent user-facing path. Otherwise mark **partial**.
5. Remove superseded Python files with a separate, reviewed Git commit **only after** the compatibility gate passes. Do not silently delete earlier user work or claim Python-free self-hosting prematurely.
6. Port Rust components into Nobody itself after the Nobody compiler/runtime can express them; perform reproducible stage-1 → stage-2 → stage-3 bootstrap tests before retiring Rust.

## Typed independent bytecode VM and .nbc v2 files (2026-11-09; verified)

The Rust bytecode VM now enforces explicitly annotated `int`/`bool` arguments and return values when invoking functions. Portable `.nbc` files write wire version 2, carrying exactly one byte per parameter type and one byte for the return type; the decoder keeps backwards compatibility with old untyped v1 files. Corrupt/unknown type tags and inconsistent signature lengths fail closed. Differential tests compare saved bytecode with the Rust evaluator for `ai`, `sk` and `en`. Untyped functions remain compatible; `float`/`str` bytecode signatures and typed AST JSON v1 exports remain unsupported. This is not full static typing, Python replacement or self-hosting. All four checks passed, including [Rust CI #42666833278](https://github.com/siposboris/NOBODY/actions/runs/42666833278); merged via [PR #34](https://github.com/siposboris/NOBODY/pull/34).

## Explicit localized function signatures in Rust source and IR (2026-11-09; verified)

The Rust parser now accepts optional primitive annotations in function signatures (`fn f(flag: bool, n: int): bool`) and normalizes localized primitive keywords (e.g. Slovak `logicke`, `cele`) into canonical fields on every Rust IR function node. The Rust evaluator enforces declared parameter/return types at runtime. The standalone native x86-64 compiler honors `int` and `bool` annotations and rejects conflicting inferred constraints or mismatched call/return types. The existing untyped function syntax remains valid.

**Deliberately partial:** no stable typed AST JSON schema yet; that output still rejects annotated functions rather than lose metadata. Typed `int`/`bool` bytecode runtime checks have been added separately, while both bytecode and native code still reject `float` or `str` function signatures. The partial Rust static checker does not guarantee global type soundness; Python legacy comparison and future self-hosting remain separate work. All four workflows passed, including [Rust CI #42660895804](https://github.com/siposboris/NOBODY/actions/runs/42660895804) with actual native ELF differential tests; merged in [PR #33](https://github.com/siposboris/NOBODY/pull/33). This is a partial implementation, not a completed typed-IR or self-hosting roadmap criterion.

## Native x86-64 boolean parameter inference (2026-11-09; verified)

The standalone Linux x86-64 machine-code backend now infers fixed i64/bool parameter signatures from direct typed contexts in previously declared functions. It checks typed call sites, lexical shadowing and contradictory constraints, rejecting unsupported ambiguous boolean signatures rather than silently converting values. Real ELF-vs-Rust evaluator tests run `ai`, `sk` and `en` functions with boolean parameters. This is not a general flow-sensitive type system, polymorphic specialization, a replacement for the Python frontend or self-hosting. This conservative type-inference slice passed all four GitHub checks, including [Rust CI #42654797725](https://github.com/siposboris/NOBODY/actions/runs/42654797725), and merged via [PR #32](https://github.com/siposboris/NOBODY/pull/32).

## Native x86-64 lazy boolean execution (2026-11-09; verified)

The independent Linux ELF backend now lowers typed boolean `and`/`or` into actual lazy jumps. It can print a single bool or integer and compile functions with signed-i64 arguments and **consistent explicit i64 or boolean return types**. Actual ELF-vs-Rust evaluator differential fixtures cover `ai`, `sk`, `en`, overflow skipped by short-circuit control flow, and side-effecting functions skipped on the RHS. Boolean parameters, full static typing, recursion and global captures remain unsupported. This is a partial native backend improvement, not proof of complete Python replacement or self-hosting. All four CI workflows passed, including [Rust CI #42652961905](https://github.com/siposboris/NOBODY/actions/runs/42652961905); merged in [PR #31](https://github.com/siposboris/NOBODY/pull/31).

## Native Linux x86-64 named-function subset (2026-11-09; verified)

The independent Rust AOT backend can compile restricted top-level named functions to standalone Linux ELF machine code using private stack-based parameter passing and isolated callee frames. It supports signed i64 parameters and explicit signed i64 returns, calls only to earlier declarations, lexical locals and existing if/while/for-range statements inside functions. Rust interpreter output is compared against actual ELF execution for ai/sk/en fixtures. The backend deliberately rejects recursion, forward references, global captures, implicit null returns and unsupported value types. The outputs do not depend on Python or the Nobody VM; this is not full language parity or self-hosting. The function-call ELF subset and its failure cases passed all four CI workflows, including [Rust CI #42650743553](https://github.com/siposboris/NOBODY/actions/runs/42650743553); merged in [PR #30](https://github.com/siposboris/NOBODY/pull/30).

## Native Linux signed `for range` milestone (2026-11-09; verified)

The native x86-64 AOT backend in Rust now emits direct machine-code ELF for bounded signed `for ... in range(...)` loops as well as the earlier arithmetic, branching and `while` slice. Argument expressions are evaluated once, loop variables are lexically isolated, and nested `break`/`continue` use loop-specific targets. A zero step terminates with status 6; checked iterator advance exits a range on signed i64 overflow. New actual-ELF parity tests compare `ai`, `sk` and `en` programs with the Rust evaluator. The output has no Python, libc or Nobody bytecode VM dependency, but compile-time GNU binutils are required. This is still a partial native backend, not Python module replacement or self-hosting. The implementation passed all four workflows including [Rust CI #42641964768](https://github.com/siposboris/NOBODY/actions/runs/42641964768) and was merged in [PR #29](https://github.com/siposboris/NOBODY/pull/29).

## Current verified migration direction (2026-11-08)

The compiled **`nobody` Rust executable** is the primary user-facing command for its documented subset, including `run`, `parse`, `translate`, `typecheck`, `info`, and `languages`. The new `direction PROFILE` command ports the old Python language-direction metadata for all 40 active profiles (four RTL, 36 LTR); its verification is tracked in GitHub Actions. Rust commands do **not** spawn Python. Python still runs in differential CI and provides older features (including AST JSON) not yet supported in Rust.

The older inventory below records original migration planning and is **historical**, not a current source-code status table. No complete Python module has yet passed every removal criterion. Do not delete Python reference files until their exact supported semantics and tests have been replaced; delete module by module after a dedicated review.

## Parenthesized AST JSON expressions (2026-11-08, verified)

The token-aligned Rust JSON source-line visitor now supports parenthesized/grouped expressions, including nested grouping and grouped left operands, without changing the immutable syntax AST. It uses a bounded (64 levels) source-token backtracking attempt when determining whether the grouping encloses a complete expression or only a subexpression. Three additional SK/EN differential fixtures compare the resulting JSON byte-for-byte against the Python reference. All 13 cases passed in [Rust CI #42522280632](https://github.com/siposboris/NOBODY/actions/runs/42522280632), with the frontend, lexer and roadmap workflows also green; merged via [PR #22](https://github.com/siposboris/NOBODY/pull/22). The Rust `run` implementation itself is unchanged. This is another subset improvement; true source spans, JSON import/validation/emission and full Python parity remain outstanding.

## Multiline AST JSON source-line bridge (2026-11-08, verified narrow subset)

The Rust `ast-json` exporter extends its exact legacy `nobody.ast.v1` serializer to a conservatively supported **multiline subset**, using original UTF-8 lexer token line positions to annotate the already-validated Rust AST. This source-line mapping is independent of Python at runtime. New exact-output fixtures cover Slovak and English calls, branching, loops, and functions across physical source lines. All four workflows passed in [Rust CI #42521825341](https://github.com/siposboris/NOBODY/actions/runs/42521825341); merged via [PR #21](https://github.com/siposboris/NOBODY/pull/21).

The Rust parser still does **not** retain a general source-span-bearing AST. To prevent false location claims, the mapping rejects unsupported constructions (including syntax shapes not yet mapped or incompatible with the Python reference) with N4004 rather than guessing; one-line export remains backward compatible. Full AST JSON deserialization, schema validation, source re-emission, indexed/list expressions, and full grammar parity remain open. Do not remove Python AST modules yet.

## AST JSON source migration (2026-11-08, verified single-line subset)

Rust now offers an **export-only** `nobody.ast.v1` bridge for **single-line** source files in `nobody ast-json PROFILE FILE`. It reproduces the existing Python schema for the parser's currently supported AST kinds without introducing Python into the compiled binary. Multiline source is explicitly rejected with N4004 because Rust parser AST nodes do not retain original node line positions; exporting fabricated line numbers would corrupt downstream tooling. Import, schema validation, output profile emission, full grammar, true source spans and large-input safeguards are still migration gates. Six differential fixtures compare byte-for-byte Python vs Rust output in CI. All six fixtures and the full Rust CI gate passed in [run #42520969571](https://github.com/siposboris/NOBODY/actions/runs/42520969571); merged via [PR #20](https://github.com/siposboris/NOBODY/pull/20). Do not delete `nobody/ai_ir.py` or `nobody/ast_format.py` yet.

## Phase 4 first real Linux x86-64 machine-code output (2026-11-09; verified)

A new Rust backend generates GNU Intel-syntax x86-64 assembly from a restricted, statically checked signed-i64 subset of canonical Nobody IR; `as` and `ld` produce a standalone Linux ELF64 executable using only kernel syscalls. The native binary does not embed the Nobody interpreter/VM or require Python/Rust/libc at runtime. Codegen supports numeric/boolean expressions, lexical variables, basic conditionals, while loops and single-integer output. It rejects any semantic construct outside the initial native subset, notably functions, for-range and rich data types. Native executable behavior is compared to the existing Rust evaluator in CLI integration tests and CI, including overflow/budget failure paths. This is an explicitly partial migration, not a feature-complete compiler or Nobody self-hosting. See [native backend details](NATIVE_X86_64.md). All four workflows passed, including the real ELF/linker/standalone execution parity tests in [Rust CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375); [PR #28](https://github.com/siposboris/NOBODY/pull/28) merged. This fulfills **only the first-working-native-Linux-binary criterion**, not full compiler feature parity or self-hosting.

## Phase 4 portable bytecode wire format (2026-11-09; verified)

The experimental Rust bytecode compiler has separate `compile-bytecode`, `verify-bytecode` and `run-compiled` native CLI paths. A deterministic, explicitly little-endian format saves compiled instructions, independently encoded function units and metadata; a strict bounded decoder rejects unsupported revisions, invalid opcodes, framing errors and accidental corruption, then calls the VM verifier on all units. Files can execute after the source is removed. No Python, repository working tree or source-language profile is needed at execution time. The checksum is not a cryptographic signature and the verifier is not a sandbox. See [the format specification](BYTECODE_FORMAT.md). Full platform targets, stable backwards compatibility, language features and self-hosting remain unfinished. All four workflows passed, including [Rust CI #42599391089](https://github.com/siposboris/NOBODY/actions/runs/42599391089); this slice was merged in [PR #27](https://github.com/siposboris/NOBODY/pull/27).

## Phase 4 isolated named function bytecode (2026-11-09; verified)

The independent Python-free Rust bytecode runtime now compiles named, top-level function definitions to verified function units. It supports ordered positional arguments, return values, implicit `null`, calls to earlier declared functions, control flow inside functions and calls from expressions. Independent VM call frames prevent caller/callee local-stack leaks. Call metadata and return-stack discipline are verified; recursive/forward calls and malformed arity fail explicitly. This subset intentionally does **not** capture or access globals (the Rust evaluator presently uses snapshots of global state in function calls), and nested functions, closures, true recursion and first-class functions remain unsupported. The Python reference stays intact. This increment is not a complete bytecode format, normal-language compiler or self-hosted toolchain. All four workflows passed in [Rust CI #42577817470](https://github.com/siposboris/NOBODY/actions/runs/42577817470), including Rust unit/integration tests, strict Clippy and bytecode/AST function parity; merged via [PR #26](https://github.com/siposboris/NOBODY/pull/26).

## Phase 4 signed for-range bytecode (2026-11-09, verified)

The independent Rust VM extends beyond `while` with one/two/three-argument canonical `range` iteration. The signed step is evaluated once; nested loops, `break`, `continue`, shadowed iterators, dynamic outer assignments and i128 limit termination are covered by explicit tests. Bytecode verifier checks all declared temporary slots on both range-control edges; invalid targets are rejected and zero steps raise N4006 at runtime. Native commands do not require Python. All four checks passed in [Rust CI #42559765856](https://github.com/siposboris/NOBODY/actions/runs/42559765856), including the added signed-range VM parity tests; merged via [PR #25](https://github.com/siposboris/NOBODY/pull/25). This is not yet arbitrary iterable support, closures, a stable serialized bytecode format or native machine code, and cannot qualify any Python reference module for deletion.

## Phase 4 control-flow bytecode (2026-11-08; verified)

The independent Python-free Rust bytecode path now has basic control-flow compilation: `if/elif/else`, `while`, `break` and `continue`, integer comparisons, boolean literals, `not`, and lexically scoped block declarations. A fixed-point control-flow verifier checks jump bounds, stack-height agreement and definite initialization at branch joins and while back-edges. The VM has explicit typed runtime checks and a bounded instruction count. Regression fixtures compare Rust VM output to the existing Rust AST evaluator for `ai`, `sk` and `en` syntax; this is **not** complete Python semantic parity. All four workflows passed, including [Rust CI #42526595586](https://github.com/siposboris/NOBODY/actions/runs/42526595586); the implementation merged in [PR #24](https://github.com/siposboris/NOBODY/pull/24). Functions, `for`, first-class values, portable serialization and native code generation still block completion of the original Phase 4 checklist items.

## Phase 4 Rust bytecode milestone — independent execution path (2026-11-08, verified)

A newly compiled `nobody bytecode` / `nobody run-bytecode` path produces and verifies **in-memory integer bytecode** from the Rust canonical IR and executes it with an independent stack VM. This begins actual compiler pipeline work rather than extending only the JSON exporter or reference language catalogue. Current semantics are intentionally restricted to integer variables, arithmetic and print; unsupported AST forms fail closed without switching to Python or the Rust tree evaluator. The verifier checks stack behavior, slot initialization and a final halt. Output is checked against the existing Rust evaluator in CLI/integration tests; parity against the *complete* Python implementation, serialized bytecode and native machine code remain future gates. All four workflows passed, including [Rust CI #42524201892](https://github.com/siposboris/NOBODY/actions/runs/42524201892), and [PR #23](https://github.com/siposboris/NOBODY/pull/23) merged. None of the original nine broad Phase 4 acceptance criteria have been fully satisfied yet.

## Updated component-level status — 2026-11-08

The following is a **qualitative inventory**, not a time or effort percentage. Across nine broad workstreams, seven have partial Rust implementations and two still lack their planned complete implementations. Even the seven partial modules are **not** full Python replacements. The native `nobody` CLI is independently executable without Python for its explicitly supported subset.

| Workstream | Rust status | What remains before replacing Python completely |
|---|---|---|
| Lexer / Unicode source safety | Partial | Full Python behavior and source-location parity across Unicode edge cases |
| Syntax language profiles | Partial (40 active profiles) | Full historical catalogue distinction, terminology review, collisions and compatibility |
| Parser / AST | Partial | Complete grammar, source spans and stable lossless tree representation |
| Source translation | Partial | All keyword collisions, semantic equivalence and round-trip correctness |
| Interpreter / scopes / functions | Partial | Closures, first-class functions, full numeric/built-in behavior and supported runtime semantics |
| Static typing | Partial | Flow, return-value, annotation and diagnostic parity |
| Error catalogue | Partial | Consistent complete error codes, position reporting and CLI interfaces |
| AST JSON interchange | Not yet complete in Rust | Versioned deterministic import, export, validation and source emission |
| Native compiler / backend | Not yet implemented | Bytecode or machine-code generation, reproducible build and deployment |

This inventory intentionally does **not** divide tasks into a numeric percentage: comparable line counts, implementation effort, and acceptance coverage are unavailable. The homepage's **39%** is a separate *unweighted roadmap checklist* metric and is **not** the Python-to-Rust migration percentage.

**Current added migration slice:** the `nobody audit-languages` command performs a Rust-only structural audit of the **40 active profiles**: keyword counts/duplicates, uppercase/control characters, NFC and forbidden bidi controls. This replaces a specific user-facing **subset** of Python `audit-languages`; the Python reference audits its own legacy vocabulary and remains in regression tests. It is not a linguistic/native-speaker audit or a reason to delete `nobody/profile_validator.py`.

**Next migration gates (in priority order):** (1) compare Python and Rust profile audit findings for the common 39 human-language profiles, allowing for the intentionally different `ai` dialect; (2) port deterministic AST JSON v1 and its round-trip tests; (3) implement closure semantics and built-ins with differential tests; (4) retire individual Python modules *only after* explicit replacement and regression gates. Then begin staged self-hosting of the compiler in Nobody. No Python module has been removed in this slice.

## Current migration inventory

| Component | Python reference | Rust replacement | Status |
|---|---|---|---|
| Unicode source tokenization | `nobody/lexer.py`, `nobody/bidi.py` | `rust/src/lib.rs` | **Partial Rust port**: byte offsets, strings/comments, native-script names, N2001–N2005. Not yet complete Unicode equivalence |
| Human and AI language profiles | `nobody/languages.py` | Planned Rust tables | Python remains reference |
| Parser and typed AST | `nobody/parser.py` | Planned Rust parser | Python remains reference |
| Source-to-source translation | `nobody/translator.py` | Planned Rust translator | Python remains reference |
| AST interpreter/scopes/closures | `nobody/ast_vm.py`, `nobody/scopes.py` | Planned Rust VM | Python remains reference |
| Static type analysis | `nobody/static_types.py`, `nobody/return_types.py` | Planned Rust checker | Python remains reference |
| Error-code catalogue | `nobody/errors.py` | N2001–N2005 in Rust lexer | Partial; all other numbered codes pending |
| AST JSON schema/emitter | `nobody/ai_ir.py`, `nobody/ast_format.py` | Planned Rust IR | Python remains reference |
| Bootstrap Nobody compiler | `compiler/bootstrap.nbd` and legacy bootstrap | Rust native compiler frontend/backend pending | Self-hosting not proved |

## Build current Rust prototype

Install a Rust toolchain on your own machine, then from the repository root:

```bash
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo run --manifest-path rust/Cargo.toml -- lex examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- check examples/hello_sk.nbd
```

**`check` currently checks lexical validity only**, not full parse or type correctness. The executable is an experimental lexer frontend, not a complete compiler.

The generation environment used for the first Rust commit did not contain `rustc` or `cargo`; the source was authored and published but **could not be compiled or tested locally**. A GitHub Actions workflow exists for remote verification; its outcome is not yet confirmed.

## Next steps

1. Establish successful GitHub Rust CI, fix compiler/linter errors without masking failures.
2. Port the language table and Unicode lexer with the missing combining-mark ranges and exact source spans; differential regression suite.
3. Implement Rust parser and AST, including the canonical `ai` profile.
4. Port numbered errors, source translator and static checker.
5. Port runtime with scopes and closures, then bytecode and native backend.
6. Remove Python files **individually** when their gate passes, and finally remove the Python CI/bootstrap requirements.
7. Rewrite core compiler in Nobody, demonstrate native self-compilation and reproducible outputs; then retire Rust if fully replaced.


## 0.23 incremental update

A small Rust AST parser is now authored in [`rust/src/syntax.rs`](../rust/src/syntax.rs) and exposed via `nobody-rs parse sk|en|ai FILE`. Parser coverage is intentionally restricted to variable declarations, optional primitive annotations, assignments, literals, basic operators and call syntax. Seven Rust unit tests and three CLI fixtures were added, but **none have been verified as passing in the authoring environment** (Rust toolchain absent). This is **partial migration** only. Python parser and other modules must remain until all existing reference behavior is replaced and differential checks pass.


## Rust 0.24 migration update

The Rust parser now has AST node definitions and source parsing for branching, loops, functions, returns and nested blocks for `sk`, `en` and `ai`. This extends the 0.23 subset but **still does not establish feature parity** with the existing Python parser, language tables, translator or runtime. No Python component is eligible for deletion yet.

The Rust build is not confirmed because the local toolchain is unavailable; GitHub Actions must be checked before accepting any further component as verified. New diagnostic categories still require stable Nxxxx numbers and specific documentation.


## 0.25 incremental migration update

The Rust compiler frontend now has a **standalone contextual AST validator**, capable of checking whether `return`, `break` and `continue` are legal within the current function/loop nesting. The validator adds N3008 and N3009 to the append-only error catalogue, plus CLI integration tests. This is still only a **partial** replacement of the Python reference parser and evaluator.

Rust compilation and CI must be demonstrated before marking the component tested. No historical Python file is eligible for removal yet.


## 0.26 — keyword and AI syntax migration

- Rust contains [40 keyword tables](../rust/src/profiles.rs): 39 experimental human languages in native scripts, plus redesigned canonical `ai`.
- The Rust `ai` syntax now uses ordinary programming keywords instead of `ai_` prefixes. The Python `ai` implementation still recognizes the **legacy prefixed** dialect as historical reference.
- The Rust `migrate-ai` command converts recognized `ai_` tokens safely, without modifying strings or comments. The original sample is preserved as `examples/hello_ai_legacy.nbd`; the current `examples/hello_ai.nbd` uses new syntax.
- A preliminary Rust token-aware Nobody-to-Nobody source translator now exists for all profiles. It is **not yet a complete syntax/semantic equivalence guarantee** when identifiers collide with keywords.
- Tests were authored for all 40 tables, native-script lexer/parser paths, AI syntax and round trips, but **successful compilation and full compatibility validation have not yet occurred**. Do not remove `nobody/languages.py` or any Python reference file until full feature parity and repeatable tests are verified.
- After a green Rust build, priorities remain: cross-profile Unicode and identifier conflict audit, full semantic AST interchange, source-span diagnostics, static types, scopes/closures and Rust runtime.

## 0.29 — temporal IR symbol visibility

The Rust IR lowerer now resolves built-in names using sequential declaration visibility: later declarations do not retroactively shadow earlier uses, and a declaration's initializer uses prior bindings. Recursive function references see the function's own name. Three new unit tests document these choices. Runtime scoping and closure behavior remain unspecified/unverified. No Python module is removed and Cargo/CI success is not claimed.
