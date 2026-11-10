# Nobody — Development Roadmap

**Last reviewed:** November 9, 2026  
**Language:** English (the canonical roadmap language).  
**Status:** Experimental. Checked boxes describe *implemented artifacts*, not proof that all features are production-ready.

Nobody is an independent, MIT-licensed programming language project with an eventual **self-hosted Nobody compiler**. Rust is the temporary implementation language for the compiler and runtime; Python is retained only as a legacy comparison baseline until replacements pass parity tests. The project is not an official OpenAI product.

## How to read this roadmap

- **[x] Implemented:** a specific capability or file has been committed or reproduced in a local prototype, as described. A checkmark does **not** imply full Rust parity, production reliability or successful GitHub CI.
- **[ ] Partial:** implementation has begun but the acceptance criterion remains unmet.
- **[ ] Planned:** not implemented or independently verified.
- Historical prototypes, local test totals and CI results must remain distinct. A newly created Rust module without a verified build is **not considered completed** at the component level.
- Each new diagnostic category must receive a stable **Nxxxx** number, a precise cause, a concrete remedy, documentation and regression tests. Numbers must never be reassigned.

## Non-negotiable requirements

- [x] Adopt **one syntax-language profile per source file**, with separate human-readable and machine-readable `ai` profiles in the experimental Python reference.
- [x] Preserve native writing systems, including Arabic/Urdu/Hebrew/Persian RTL *logical source order* and original non-Latin scripts in the experimental implementation.
- [x] Record 249 human language/script variants in the planning catalogue, **not** as implemented syntax.
- [x] Provide 39 experimental human syntax dictionaries plus one `ai` canonical profile in the Python reference; natural-language accuracy remains unreviewed.
- [x] Keep Nobody-to-Nobody syntax translation conceptually distinct from compiling to machine code.
- [x] License the project under MIT and clearly attribute ChatGPT by OpenAI as an assistance tool, with no endorsement implied.
- [ ] Guarantee that all profiles and all semantics work end-to-end in the Rust compiler.
- [ ] Publish a stable, versioned language specification and compatibility policy.

## Phase 0 — Foundation, project hygiene and migration

- [x] Publish repository `siposboris/NOBODY` with source, examples, MIT license, AI disclosure, feature matrix, changelog and release guidance.
- [x] Preserve the earlier Python bootstrap/interpreter prototypes and Nobody-authored bootstrap source in local development archives.
- [x] Add a historical experimental Python test suite and separate GitHub frontend test workflow.
- [x] Document [Rust-first migration acceptance gates](RUST_MIGRATION.md), including Python deletion only *after* verified behavioral equivalence.
- [x] Create a dependency-free Rust crate at [`rust/`](../rust/) and its [Rust CI workflow](../.github/workflows/rust.yml).
- [x] Independently verify Rust compilation, formatting, Clippy and CI. **Verified** on GitHub Actions [run #42501364426](https://github.com/siposboris/NOBODY/actions/runs/42501364426), commit `e1437613639c5428babc0e6d406795215762fce4`: complete Rust gate passed, 109 unit tests plus integration tests, strict Clippy, formatting and CLI smoke. The local authoring container still lacks Cargo.
- [ ] Import **all** tested legacy bootstrap sources and corresponding fixtures to GitHub without accidentally discarding work.
- [ ] Establish release tags, checksums, CI-attested builds and reproducible acceptance tests. **Verified input locking:** Cargo-generated `rust/Cargo.lock` is committed, all Rust build/test/Clippy/smoke commands use `--locked`, and [GitHub CI #42504352399](https://github.com/siposboris/NOBODY/actions/runs/42504352399) passed. Release tags, signed/checksummed distribution artifacts, CI attestations and reproducible binary output remain open.
- [ ] Remove Python from the regular execution path **only after** Rust replacements are fully tested.

## Phase 1 — Lexer, syntax, parser, AST and translation

**Full profile execution regression (2026-11-10; verified [PR #48](https://github.com/siposboris/NOBODY/pull/48), [Rust CI #42828013644](https://github.com/siposboris/NOBODY/actions/runs/42828013644)):** A Rust integration test constructs a typed, nontrivial Nobody program from each of the 40 registered syntax dictionaries: declarations, localized primitive types, `for` loops, `if`/`else`, named typed functions, Unicode strings and output. It verifies canonical IR equality, Rust AST evaluator output, conservative static checker acceptance and, on Linux x86-64 CI, actual standalone ELF execution with byte-identical stdout for *every* profile. Mixed-profile keywords are rejected. This is a machine-generated structural/runtime regression test, not 40 native-speaker linguistic reviews or full grammar parity.



- [x] Implement a native UTF-8 token scanner *prototype* in Rust for names, literals, symbols and comments, with byte spans and native-script letters; expose `nobody-rs lex` and lexer-only `check`.
- [x] Assign numbered Rust lexer diagnostics N2001–N2005 for string, case, combining-mark, unsupported-glyph and bidi-control errors.
- [ ] Complete the Rust lexer against Unicode normalization, combining characters, RTL, confusables, token boundaries and reference regression fixtures. **NFC progress (2026-11-08):** real Unicode NFC normalization validation in identifiers now rejects non-NFC names with N2008; UTF-8 source bytes, literal contents and comments are preserved. [Rust CI #42503823825](https://github.com/siposboris/NOBODY/actions/runs/42503823825) passed. This adds a pinned external Rust crate, and a committed Cargo.lock / full UTS #39 coverage are still outstanding. The full item stays open. **2026-11-08 N2007 update:** targeted mixed Latin/Greek/Cyrillic identifier guard rejects spoofable cross-script spellings without changing strings/comments or Japanese mixed scripts; four additional tests passed with the existing six Unicode tests in [Rust CI #42503288105](https://github.com/siposboris/NOBODY/actions/runs/42503288105). This is not general UTS #39 confusable detection or Unicode normalization; the item remains open. **Verified progress:** six Unicode/RTL lexer regression tests cover Arabic/Hebrew/Persian/Urdu logical token order, byte spans, Unicode scalar columns, composed identifier marks, orphan marks, and bidi controls. [Rust CI #42502868233](https://github.com/siposboris/NOBODY/actions/runs/42502868233) passed. Full normalization, confusable detection, exhaustive reference fixtures and native-speaker checks remain open. **Progress (2026-11-08):** added strict decimal numeric token validation with stable N2006 and three authored regression tests. Full Unicode/RTL and compiled tests still pending; item stays open. **Regression update (2026-11-08):** eight additional UTF-8/RTL/NFC/script-spoofing integration tests were merged in [PR #1](https://github.com/siposboris/NOBODY/pull/1), and [Rust CI #42505923514](https://github.com/siposboris/NOBODY/actions/runs/42505923514) passed locked builds, formatting, test and Clippy checks. The narrow guards do **not** establish full UTS #39, Unicode security completeness, or native-speaker review.
- [x] Port the **source keyword tables** for all 39 human syntax profiles and a redesigned `ai` profile into Rust, including original Unicode spellings, direct profile selection and preliminary native-script parser tests.
- [ ] Verify all 40 Rust profiles pass compiled lexer/parser tests, native-speaker review, Unicode normalization checks, reserved-word collisions and parity with historical behavior. **Verified subset:** representative conditional/function/loop/typed-declaration fixture parses and lowers to equal IR across all 40 profiles; all 1,600 ordered profile translations preserve that IR in [Rust CI #42501821609](https://github.com/siposboris/NOBODY/actions/runs/42501821609). Native-speaker review, full grammar and Unicode/normalization parity remain open.
- [x] Commit a **limited** Rust recursive-descent/precedence AST parser for `sk`, `en`, `ai`: declarations, optional primitive annotations, reassignment, expression operators, literals and call syntax. Its source exists, but a passing Rust build is **not yet verified**.
- [x] Extend the **Rust AST parser source** with `if/elif/else`, `while`, `for ... in`, named function declarations, `return`, `break`, `continue` and nested brace blocks for `sk/en/ai`. Rust source and tests committed; successful compilation remains unverified.
- [ ] Expand to complete language grammar, all syntax profiles, static semantic checks, proper spans and runtime behavior. **The current parser is not full frontend parity and does not execute programs.**
- [x] Implement a limited Python-reference AST parser for declarations, assignments, expressions, functions, loops, calls and blocks, with experimental 0.10–0.21 extensions.
- [x] Implement JSON `nobody.ast.v1` export/import validation and Nobody-source re-emission for a subset in the Python reference.
- [x] Introduce a versioned **canonical Rust IR v1 source representation** (declarations, expressions, branching, loops, functions) with conservative normalized built-ins and structural cross-profile comparison; implementation and tests committed, Rust build unverified.
- [ ] Port complete typed AST/IR semantics, JSON AST validator, deterministic source emitter and true semantic round-trip validation to Rust. The new IR v1 is in-memory only. **Partial verified progress (2026-11-09):** opt-in Rust `ast-json-v2` export now carries canonical typed function parameters and return annotations across `ai`, `sk`, `en`; v1 stays unchanged. A bounded Rust `nobody.ast.v2` structural importer, N5001 validation and deterministic JSON normalization have been verified in [PR #36](https://github.com/siposboris/NOBODY/pull/36), including [Rust CI #42686022615](https://github.com/siposboris/NOBODY/actions/runs/42686022615); an import-to-Rust-parser-AST bridge with lexer/context checks and non-executing structural comparison is now verified in [PR #37](https://github.com/siposboris/NOBODY/pull/37) ([Rust CI #42687593990](https://github.com/siposboris/NOBODY/actions/runs/42687593990)); partial deterministic Nobody source emission with mandatory Rust parser round-trip is now verified in [PR #38](https://github.com/siposboris/NOBODY/pull/38) ([Rust CI #42695169916](https://github.com/siposboris/NOBODY/actions/runs/42695169916)); executable JSON import, full semantic round-trip and complete parity remain open. [Rust CI #42668682647](https://github.com/siposboris/NOBODY/actions/runs/42668682647) passed.
- [x] Implement a prototype Nobody-to-Nobody translation tool in the Python reference that preserves strings and comments.
- [x] Implement preliminary Rust **Nobody-to-Nobody** keyword translation using token spans, preserving strings and comments, with roundtrip unit tests across selected native-script profiles.
- [x] Add a Rust **N3011 collision guard**: reject non-keyword source identifiers that coincide with reserved words in the destination profile; include line/column and concrete rename instructions, plus unit/integration tests.
- [ ] Extend Rust translation to become fully syntax-aware for lexical scope, automatic hygienic renaming, keyword ambiguities, Unicode normalization and verified semantic equivalence across all 40×40 profile pairs.
- [ ] Finish complete grammar: declarations, types, assignments, functions, lexical scope, collections, structured data, imports and modules.
- [ ] Build formatter, useful source diagnostics and parser error recovery.

## Phase 2 — Semantic analysis, typing and error reporting

**Known-operand logic/ordering safety (2026-11-10; verified [PR #43](https://github.com/siposboris/NOBODY/pull/43), [Rust CI #42766343202](https://github.com/siposboris/NOBODY/actions/runs/42766343202)):** Rust's conservative checker rejects provably invalid text ordering and non-boolean `and`/`or`/`not` with existing N1002, while preserving dynamic unknown expressions for runtime evaluation. Text `==`/`!=` and text concatenation remain accepted. Tests exercise nested expressions and the `ai`, `sk`, and `en` profiles. This is not sound whole-program type inference and closes no broad criterion.



- [x] Prototype lexical scopes, captured closures and basic runtime type annotations in Python.
- [x] Prototype static primitive type checks, lexical-scope diagnostics, conditional path joins and known-function arity checks in Python.
- [x] Prototype conservative function return-type inference, including unknown results when branches or dynamic calls cannot be proved, in Python.
- [x] Introduce an **append-only numbered error catalogue** in the Python frontend and [documentation with causes and repairs](ERROR_CODES.md).
- [ ] Port scopes, closures, type analysis, return inference and all applicable **Nxxxx** diagnostics to Rust.
- [x] Add precise **N1007** diagnostics for unknown explicit primitive type annotations in Rust parser and IR type checker, with six regression tests and preserved Slovak/AI primitive keywords. Verified by [Rust CI #42508306873](https://github.com/siposboris/NOBODY/actions/runs/42508306873) and merged in [PR #2](https://github.com/siposboris/NOBODY/pull/2). This does not implement user-defined types or complete static typing.
- [x] Add conservative **N1008** to Rust static checking for syntactically certain zero literal divisors (`/`, `//`, `%`), including signed decimal literals and exponents. Eight focused integration tests added and full [Rust CI #42508607904](https://github.com/siposboris/NOBODY/actions/runs/42508607904) passed; merged in [PR #3](https://github.com/siposboris/NOBODY/pull/3). Computed or runtime-dependent zero remains unproved and unchecked.
- [x] Extend N1008 to checked exact integer expression evaluation (`+`, `-`, `*`, unary signs, nested parentheses), detecting `10 / (5 - 5)` without unsafe floating-point approximation. Six new integration tests, full [Rust CI #42509584371](https://github.com/siposboris/NOBODY/actions/runs/42509584371) green; merged via [PR #4](https://github.com/siposboris/NOBODY/pull/4). Dynamic, floating-point, overflowing and non-modeled arithmetic remain deferred.
- [ ] Implement strict, inferred and gradual typing modes with explicit boundaries and stable semantics.
- [x] Add a Rust **AST contextual validation pass** rejecting return outside a function (N3008) and break/continue outside a loop in the current function (N3009), including nested-function boundaries. Implemented in source with ten authored unit tests; successful Rust compilation is **not yet verified**.
- [x] Commit Rust AST **control-flow context validation** (return only inside functions; break/continue only inside a loop in the current function), with new documented N3008/N3009 diagnostics and native Rust CLI integration tests. Source exists; cargo compilation and CI success remain **unverified**.
- [ ] Implement function parameter/return annotations, type inference for dynamic values, control-flow narrowing and union/optional/result/error types.
- [ ] Implement generics, algebraic data types, enums, pattern matching, traits and interfaces.
- [ ] Implement deep immutability, safe aliasing, borrow/move semantics and fully specified mutability.
- [ ] Develop semantic linter, source mappings, high-quality diagnostics and editor quick fixes.
- [ ] Eliminate legacy fallback `N9999` by assigning precise numbered categories with regression tests.

## Phase 3 — Memory, safety and resource management

- [ ] Implement a real garbage-collected memory profile with measured pauses and memory use.
- [ ] Implement ownership/borrowing, moves and lifetime rules, with formal safety invariants.
- [ ] Implement arena/region, reference-counted and deliberately unsafe manual profiles.
- [ ] Specify safe interoperability and FFI boundaries between memory profiles.
- [ ] Implement allocation budgets, recursion budgets, CPU timeouts and deterministic cancellation, distinguishing software counters from OS-enforced limits.
- [ ] Add fuzzing, sanitizers, race/leak detection and separately verified memory safety claims.

**Phase 4 control-flow extension (2026-11-08; verified):** The independent Rust bytecode VM now compiles `if/elif/else`, bounded `while` loops, `break` and `continue` using actual conditional/unconditional jump opcodes. Boolean literals, comparisons and `not` are supported for control flow; bytecode variable lookup follows lexical block scopes with fresh slots for shadowed declarations. A worklist verifier checks stack heights and definite slot initialization across all reachable branches and loop back-edges before execution. The VM enforces a finite instruction-step limit even for empty infinite loops. Direct CLI parity tests cover `ai`, English and Slovak programs against the existing Rust AST evaluator. All four CI workflows passed, including [Rust CI #42526595586](https://github.com/siposboris/NOBODY/actions/runs/42526595586); merged in [PR #24](https://github.com/siposboris/NOBODY/pull/24). This does **not** implement `for range`, functions, closures, a portable bytecode file format or native machine-code generation. The nine broad Phase 4 checklist criteria remain open until fully met.

**Phase 4 signed-range iteration (2026-11-09, verified):** The native, independent Rust bytecode compiler and VM now support `for variable in range(stop)`, `range(start, stop)`, and `range(start, stop, step)`, with positive/negative nonzero steps. The compiler evaluates arguments once in source order, isolates loop-iterator variables from outer scopes and gives each nested loop separate `break`/`continue` targets. `RangeCheck` and `RangeAdvance` opcodes have verifier rules for initialized slots, branch targets and stack consistency; runtime handles zero step (N4006), signed i128 overflow without wrapping, and finite instruction budgets. Exact stdout is compared with the existing Rust evaluator across `ai`, `sk`, `en` and nested/shadowed/boundary fixtures. **This is an in-memory prototype, not a complete compiler, portable serialized bytecode, native executable or self-hosting.** All original broad Phase 4 requirements remain incomplete. All four checks passed in [Rust CI #42559765856](https://github.com/siposboris/NOBODY/actions/runs/42559765856), including the added signed-range VM parity tests; merged via [PR #25](https://github.com/siposboris/NOBODY/pull/25).

**Phase 4 bytecode functions (2026-11-09; verified):** The native Rust bytecode compiler now emits separately verified function units and `Call`/`Return` instructions for named functions declared before use, with positional parameters, return expressions, default `null` return, and chained calls to previously declared functions. Each call gets its own local variable and operand stacks; the VM shares the existing finite instruction/output budgets across the entire program. The verifier checks parameter initialization, stack height at returns, function IDs, argument counts and an acyclic backward-only call graph. **Initial scope is deliberately limited to functions that use their own parameters and local variables, not reads/writes of global variables, closures, nested definitions, recursion, first-class functions or forward declarations.** Differential CLI tests compare `ai`, `sk` and `en` outputs against the existing Rust tree evaluator. This is still **in-memory experimental bytecode**, not portable bytecode, machine code or self-hosting. No broad Phase 4 checklist item is marked complete. All four workflows passed in [Rust CI #42577817470](https://github.com/siposboris/NOBODY/actions/runs/42577817470), including Rust unit/integration tests, strict Clippy and bytecode/AST function parity; merged via [PR #26](https://github.com/siposboris/NOBODY/pull/26).

**Portable binary bytecode v1 slice (2026-11-09; verified):** The Rust compiler can now write a deterministic, platform-neutral experimental `.nbc` binary with versioned header, fixed-width little-endian opcodes, function metadata, size limits and accidental-corruption checksum. The separate `nobody verify-bytecode FILE.nbc` and `nobody run-compiled FILE.nbc` commands decode, verify, then optionally execute without the original `.nbd`, syntax profile or Python. `nobody compile-bytecode PROFILE source.nbd output.nbc` refuses to overwrite existing files. Native integration tests demonstrate source-file deletion, cross-profile identical artifact bytes, corrupted/truncated file rejection and persistent VM runtime parity. Full file format details are documented in [Bytecode Format](BYTECODE_FORMAT.md). **Experimental only:** no cross-version stable ABI, cryptographic signatures, OS sandbox, native machine-code backend or self-hosting. The original broad Phase 4 bytecode/VM checkbox remains open while those acceptance requirements are incomplete. All four workflows passed, including [Rust CI #42599391089](https://github.com/siposboris/NOBODY/actions/runs/42599391089); this slice was merged in [PR #27](https://github.com/siposboris/NOBODY/pull/27).

**Native Linux x86-64 executable milestone (2026-11-09; verified):** The experimental Rust backend now lowers a **strict signed-i64 subset** of Nobody directly into Intel-syntax x86-64 assembly and uses GNU `as` + `ld` to build an executable Linux ELF64 binary. The output has its own Linux write/exit syscalls; it embeds **no Python, C or Rust runtime, and no Nobody bytecode interpreter**. First support: typed-by-inference numeric/boolean expressions, lexical variables, checked integer arithmetic, comparisons, `if/elif/else`, `while`, `break`/`continue` and `print(one integer)`. Unsupported features such as `for/range`, named functions, strings and i128 literals are explicitly rejected; this is **not a whole-language native compiler**. New direct-execution tests check ELF headers, cross-profile interpreter parity, non-overwrite, source independence, overflow and bounded loops. See [Native Linux x86-64](NATIVE_X86_64.md). All four workflows passed, including [Rust CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375); verified native execution and ELF inspection in [PR #28](https://github.com/siposboris/NOBODY/pull/28). The limited native-executable checklist criterion is now met; full-language compatibility and additional targets remain separate open goals.

**Explicit localized function signatures (2026-11-09; verified):** New backward-compatible source syntax `fn choose(flag: bool, a: int): int { ... }` and corresponding localized forms, e.g. `funkcia choose(flag: logicke, a: cele): cele { ... }`, record optional positional parameter annotations and return type in the shared canonical Rust IR. The evaluator checks declared argument and return types at runtime; the native x86-64 ELF backend uses explicit `int`/`bool` annotations to override conservative inference, checks typed calls and returns, and rejects unsupported `str`/`float` signatures. Existing untyped functions continue to run. Experimental `nobody.ast.v1` JSON export and bytecode VM **explicitly reject** typed signatures until their schemas and execution guards are extended; they never silently discard annotation metadata. The partial Rust type checker still does not prove whole-program signature soundness. Actual ELF/evaluator parity fixtures cover AI, English and Slovak; the broad Phase 4 checklist and percentage are unchanged. All four workflows passed, including [Rust CI #42660895804](https://github.com/siposboris/NOBODY/actions/runs/42660895804) with actual native ELF differential tests; merged in [PR #33](https://github.com/siposboris/NOBODY/pull/33). This is a partial implementation, not a completed typed-IR or self-hosting roadmap criterion.

## Phase 4 — Compiler pipeline, runtime and self-hosting

**Portable bytecode UTF-8 strings (2026-11-10; verified [PR #47](https://github.com/siposboris/NOBODY/pull/47) (final integrated CI required)):** The Python-free Rust bytecode VM now executes string literals, mutable text values, `+` joins, content equality, and typed `str` function calls and returns. Files carrying text are encoded with bounded, verified `NOBBC001` v3 text payload and `str` type tag; v1/v2 decoding remains supported, and integer/bool-only new files remain v2. Tests run stored `.nbc` v3 from disk and compare output against Rust AST interpretation in `ai`/`sk`/`en`, rejecting corrupted payloads. This is not a stable ABI or full self-hosting.



**Unicode text length (2026-11-10; CI pending):** The independent Rust AST evaluator and Linux x86-64 ELF compiler implement `len(text)` / localized `dlzka(text)`, returning a Unicode scalar count (not bytes or grapheme clusters) for literals, variables, concatenated strings and native function results. Real ELF-to-interpreter differential fixtures cover `ai`/`sk`/`en`, empty strings, non-ASCII symbols and invalid argument rejection. This is a narrow usable feature, not full Unicode string indexing or compiler parity.



**Native typed string function ABI (2026-11-10; verified [PR #44](https://github.com/siposboris/NOBODY/pull/44), [Rust CI #42766777167](https://github.com/siposboris/NOBODY/actions/runs/42766777167)):** Linux x86-64 generated programs now pass explicit UTF-8 `str` function parameters as address/length pairs and return `str` via `rax`/`rdx`. Earlier-declared functions mix `int`, `bool`, `str` positional arguments, support nested string-returning calls and concatenation, and check type mismatches at compilation. Cross-profile `ai`/`sk`/`en` fixtures compare real standalone ELF output to the separate Rust evaluator. This is an internal and incomplete ABI without recursion, closures, float, unannotated text inference or stable cross-platform compatibility. No broad roadmap checkbox closed.



**Native ELF text operations (2026-11-10, verified [PR #41](https://github.com/siposboris/NOBODY/pull/41)):** The restricted Linux x86-64 compiler now emits real bytewise UTF-8 string equality/inequality and runtime concatenation for string literals and text variables, including nested expressions and loops. A bounded 1 MiB lifetime arena provides storage without libc; overflow exits with status 7. New compiled ELF-vs-Rust-interpreter parity tests exercise `ai`, `sk`, `en`, Unicode, empty text, reassignment, conditionals, loop/function locality, and assembler-looking user text. All four workflows passed, including [Rust CI #42754693569](https://github.com/siposboris/NOBODY/actions/runs/42754693569). This extends the existing native compiler prototype, not a whole-language or self-hosting milestone. Text arguments/returns, full dynamic memory, Python removal, and self-hosting remain open.



**New genuine native ELF capability (2026-11-10; verified [PR #40](https://github.com/siposboris/NOBODY/pull/40)):** Native x86-64 now handles UTF-8 text variable declarations, reassignment, copying, shadowing and function-local storage by tracking read-only string pointers plus byte lengths in separate stack slots. `print` reads those values at runtime; independent `ai`, `sk` and `en` fixtures compare the actual standalone ELF output with the Rust evaluator. Text concatenation/comparison, native text parameter/return ABI, full collections and self-hosting remain open. All four checks passed including [Rust CI #42753533095](https://github.com/siposboris/NOBODY/actions/runs/42753533095). This expands an existing partial native compiler milestone, not a newly completed broad roadmap checkbox.



**New native compiler capability (2026-11-09; verified, [PR #39](https://github.com/siposboris/NOBODY/pull/39)):** The x86-64 ELF backend now supports `print()`, direct UTF-8 text literal output and multiple mixed integer/boolean/text-literal arguments via real Linux `write` syscalls, including prints inside loops. String escapes are decoded using the Rust evaluator's existing logic and emitted as numeric read-only bytes (not raw assembly text). Real ELF fixtures in `ai`, `sk`, `en` compare stdout byte-for-byte with the interpreter, not merely AST/IR equality. This provides concrete new executable behavior; dynamic string variables, text operations, string-valued function results, fully compatible native compilation and Python removal remain open. All four workflows passed, including [Rust CI #42697321428](https://github.com/siposboris/NOBODY/actions/runs/42697321428). No extra broad roadmap checkbox is claimed.



**Typed bytecode VM and portable `.nbc` v2 (2026-11-09; verified):** The Python-free bytecode VM now preserves `int`/`bool` function parameter and return annotations, checks argument values at every call, and checks return values before resuming the caller. The portable `NOBBC001` file format writes version 2 with per-function type tags and continues to decode legacy untyped v1 files. Invalid tags, wrong argument types, mismatched return types and malformed metadata fail explicitly. Typed `ai`, Slovak and English programs have differential, persisted execution and cross-profile byte-identity tests. `str`/`float` bytecode signatures and typed AST JSON v1 remain unsupported; no additional Phase 4 checkbox or self-hosting claim is made. All four workflows passed, including [Rust CI #42666833278](https://github.com/siposboris/NOBODY/actions/runs/42666833278), merged via [PR #34](https://github.com/siposboris/NOBODY/pull/34). Overall checklist remains 45/112 (40%); typed bytecode does not constitute complete Phase 4 or self-hosting.

**Native x86-64 boolean function parameter inference (2026-11-09; verified):** Earlier-declared native functions can now accept conservative `bool` parameter types alongside signed i64. The compiler infers a fixed signature from direct boolean or integer usage (`if`/`while`, `not`, `and`/`or`, numeric operations, literal equalities and calls to earlier known functions), preserves lexical shadowing, and rejects type conflicts or wrong arguments before producing an ELF. Ambiguous parameters default to i64; full flow-sensitive inference, generic specialization, type annotations and full typed IR are not yet available. Actual compiled Linux ELF outputs are compared with the Rust interpreter across `ai`, Slovak and English, including boolean arguments in nested loops and unsupported cases. This broadens the **already checked restricted native ELF item** and does **not** add a new roadmap checkbox or mean self-hosting is achieved. This conservative type-inference slice passed all four GitHub checks, including [Rust CI #42654797725](https://github.com/siposboris/NOBODY/actions/runs/42654797725), and merged via [PR #32](https://github.com/siposboris/NOBODY/pull/32).

**Native x86-64 lazy boolean and typed result slice (2026-11-09; verified):** The standalone Linux ELF backend now compiles short-circuit `and`/`or` for boolean operands, with a real conditional jump around the unevaluated RHS. It prints a single integer or boolean (`true`/`false`) and allows earlier-declared i64-argument functions to explicitly return either i64 or bool, with consistent return types enforced across branches. Independent AST-vs-native ELF tests for `ai`, English and Slovak include skipped overflow, skipped side-effect calls, boolean predicates and type-mismatch rejection. This expands the existing checked restricted ELF capability; **no extra roadmap checkbox is marked complete**. Boolean function parameters, dynamic values, full language type checking, native optimizations, other CPU targets and self-hosting remain incomplete. All four workflows passed, including [Rust CI #42652961905](https://github.com/siposboris/NOBODY/actions/runs/42652961905) with actual ELF parity tests; merged in [PR #31](https://github.com/siposboris/NOBODY/pull/31). Roadmap checklist remains **45/112 (40%)**, without double-counting the already completed restricted native ELF criterion.

**Native x86-64 function call/return subset (2026-11-09; verified):** The already verified standalone ELF compiler now emits independent function entrypoints, stack-based parameter passing, local callee frames and explicit signed i64 returns. Previously declared top-level functions can call one another, including as expression operands and inside control flow/loops, using a private internal ABI. Functions must provably return i64 along every path; global captures, closures, recursion, forward references, implicit null returns and noninteger parameters/results deliberately fail closed. Actual compiled ELF files are compared with the Rust AST interpreter in `ai`, Slovak and English, with argument order, nested calls and unsupported cases tested. **This expands the already checked restricted native ELF milestone, without adding a new roadmap checkbox; complete native function semantics, full i128 parity and self-hosting remain open.** The function-call ELF subset and its failure cases passed all four CI workflows, including [Rust CI #42650743553](https://github.com/siposboris/NOBODY/actions/runs/42650743553); merged in [PR #30](https://github.com/siposboris/NOBODY/pull/30).

**Native x86-64 signed range iteration (2026-11-09; verified):** The already verified standalone Linux ELF compiler has been extended beyond `while` to `for variable in range(stop/start,stop/start,stop,step)` over signed i64 integers, with lexical iteration variables, bound evaluation once, positive/negative step, nested loops, break/continue and checked iterator advance. Native machine code exits with status 6 for zero step, and terminates overflowing iterator steps without wrapping. Cross-language `ai`/Slovak/English native executable output, nested loops, scope and numeric boundary fixtures are tested against the independent Rust AST evaluator. GNU `as`/`ld` remain build-time prerequisites; generated ELF programs require neither Python nor the Nobody VM. **This expands the previously checked restricted native ELF item; it does not add a second roadmap checkbox, enable functions, make 64-bit integers equivalent to full i128, or complete self-hosting.** All four CI workflows passed, including [Rust CI #42641964768](https://github.com/siposboris/NOBODY/actions/runs/42641964768) with actual ELF differential tests; merged in [PR #29](https://github.com/siposboris/NOBODY/pull/29). Overall checklist remains **45/112 = 40%**.

**Phase 4 foundation (2026-11-08; verified prototype):** The first independently compiled **in-memory integer bytecode prototype** now exists in `rust/src/bytecode.rs`: Nobody source → canonical IR → instruction sequence → static stack/slot verifier → deterministic stack VM. Native CLI commands `nobody bytecode PROFILE FILE` and `nobody run-bytecode PROFILE FILE` execute separately from the AST evaluator, never invoking Python. It supports untyped declarations/assignments, integer literals/arithmetic and `print` and rejects unsupported constructs, including loops/functions, with N4004. Resource limits, overflow diagnostics and malformed-bytecode rejection have tests; three profiles are compared against the existing Rust AST evaluator. All four CI workflows passed, including format, Rust tests, strict Clippy and VM parity in [Rust CI #42524201892](https://github.com/siposboris/NOBODY/actions/runs/42524201892); [PR #23](https://github.com/siposboris/NOBODY/pull/23) merged. **This is not a portable serialized bytecode format, complete VM, full compiler, native x86-64 executable, or self-hosting.** At the November 8 foundation snapshot, all nine broad Phase 4 checklist items were still open; the first native ELF criterion was subsequently met on November 9. The path forward is typed and structured bytecode, branch/loop control-flow verification, serialization, then real native output.


- [ ] Implement a complete Rust parser and typed intermediate representation with reusable passes.
- [ ] Build a portable Nobody bytecode format, verifier and deterministic VM in Rust.
- [ ] Build compiler optimizations, debug information and a platform-neutral backend interface.
- [x] Produce a working **Linux x86-64 native executable** with a tested linker/toolchain. **Verified restricted i64 subset** using Rust IR → GNU x86-64 assembly → `as`/`ld` → standalone ELF64 with direct Linux syscalls. [Rust CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375) executed and compared the generated ELF programs. **This checkbox is for the first working native executable, not complete language compilation or multi-platform support.**
- [ ] Add Linux ARM64, Windows, macOS and WebAssembly targets with test suites.
- [ ] Remove Python from the normal compile/run workflow after Rust parity and stable integration tests.
- [ ] Rewrite compiler/runtime components in Nobody after the language can express them.
- [ ] Demonstrate self-hosting with reproducible stage 1 → stage 2 → stage 3 compiler builds and matching outputs.
- [ ] Distribute documented, verified binaries and checksums. The first native ELF prototype is verified, but no release binaries, signed artifacts or self-hosting milestone have been delivered.

## Phase 5 — Optimization and performance

- [ ] Add debug, fast-build, balanced, maximum-speed, minimum-size and low-energy execution profiles.
- [ ] Implement constant folding, dead-code elimination, inlining, SIMD/vectorization and profile-guided optimization.
- [ ] Implement architecture-specific code generation with portable fallback.
- [ ] Support safe C interoperability and explicitly versioned library ABI.
- [ ] Publish reproducible benchmarks against C, Rust, Go, Python and Julia where appropriate.

## Phase 6 — Concurrency and distributed systems

- [ ] Add structured async/await, cancellation and task lifetimes.
- [ ] Add threads, light tasks, channels and typed message passing.
- [ ] Add isolated actors, supervisor trees and recoverable failures.
- [ ] Specify race prevention, deterministic scheduling options and synchronization behavior.
- [ ] Add distributed processes, network serialization and authenticated service communication.

## Phase 7 — Scientific computing, GPU and AI tooling

- [ ] Add dense arrays, tensors, linear algebra and numerical dispatch policies.
- [ ] Add GPU kernels with explicit CPU fallback and measured portability.
- [ ] Define floating-point and reproducibility profiles.
- [ ] Implement AI/ML integration with clear hardware, resource and security limitations.
- [x] Prototype an `ai` **keyword profile** plus a versioned JSON AST interchange format in the Python reference.
- [x] Replace the prefix-based `ai_` dialect in the **Rust frontend** with familiar `let`, `fn`, `if`, `return`, `true`, `false`, `null` and other conventional keywords. Implement token-aware conversion of old AI examples and stable diagnostic N3010.
- [ ] Port full canonical AST JSON exchange to Rust and independently benchmark syntax familiarity against LLM coding accuracy and token costs. Do not claim that `ai` is objectively superior without evidence.
- [ ] Provide safe developer-agent APIs without claiming that source syntax grants AI autonomy or consciousness.

## Phase 8 — Advanced language design and formal verification

- [ ] Add hygienic macros, compile-time evaluation and controlled metaprogramming.
- [ ] Add effect checking and explicit capability restrictions.
- [ ] Experiment with design-by-contract, refinements and dependent types.
- [ ] Support machine-checkable proofs only where the soundness boundaries are explicit.
- [ ] Introduce safe query/collection DSLs and composable language extensions.

## Phase 9 — Tooling and developer experience

**Read-only formatter verification (2026-11-10, verified [PR #42](https://github.com/siposboris/NOBODY/pull/42)):** The Rust `format-check PROFILE FILE.nbd` detects noncanonical source formatting with N5002 and exit code 1, without touching the original. Tests cover `ai`, `sk`, `en` and formatter idempotence. Original comments and whitespace are not preserved by the AST emitter; full source-preserving formatting and editor integration remain open. All four workflows passed including [Rust CI #42765224570](https://github.com/siposboris/NOBODY/actions/runs/42765224570). No broad checklist item is closed.



- [ ] Build a configurator where each feature is labeled implemented, experimental or planned, without fake switches.
- [ ] Implement module-level profile negotiation and consistency checks.
- [ ] Build an LSP server, diagnostics, completion, refactoring, formatter and debugger.
- [ ] Build a reproducible package/build manager with dependency provenance and offline mode.
- [ ] Implement a versioned standard library: collections, JSON, files, processes, networking, time and cryptography interfaces.
- [ ] Add release CI, supply-chain scanning, signed/checksummed builds and reproducible Linux packaging.
- [ ] Publish tutorials and verified examples in human syntax profiles, with RTL-aware editor support.
- [ ] Add optional incremental compilation, hot reload and editor syntax-mode switching.

## Phase 10 — Production readiness

- [ ] Publish complete compatibility/specification tests and results for every claimed platform.
- [ ] Perform independent safety/security audits and benchmark memory, performance and reliability.
- [ ] Establish contributor governance, API stability promises and public deprecation policy.
- [ ] Release 1.0 **only** after the language, runtime, compiler and tooling satisfy written acceptance criteria.

## Configuration dimensions (not implemented simply because listed)

Syntax language, syntax complexity, abstraction level, strict/inferred/gradual typing, automatic/ownership/arena/reference-counted/manual memory, safety strictness, interpreted/bytecode/JIT/native/WASM execution, CPU/GPU distribution, sync/async/threads/actors concurrency, speed/size/energy/build-time optimization, platform/CPU target, diagnostics, errors, immutability, arithmetic behavior, IO permissions, proof level, dependencies, reproducibility and quotas. A selectable configuration requires an implemented behavioral difference and reproducible tests.

## Verified milestone ledger

| Milestone | What actually exists | Verification boundary |
|---|---|---|
| 0.1–0.9 | Experimental local Python translator/interpreter, Nobody bootstrap sources and archives | Historical, not all imported into GitHub |
| 0.10–0.15 | Python-reference AST parser, Unicode profiles, RTL metadata, experimental AST runtime and `ai` syntax | Local tests reported; no native compiler |
| 0.16–0.17 | Python-reference AST JSON import/emitter and lexical closures | Subset only; still Python |
| 0.18–0.21 | Python-reference type checker, branch joins, numbered errors and return-type inference | Static analysis remains conservative |
| 0.22 | Rust crate, partial lexer and CLI, GitHub Rust workflow | Source committed; **local Rust build unverified** |
| 0.23 source snapshot | Small Rust AST parser, SK/EN/AI fixtures, numbered N3006/N3007 errors | **Uncompiled/unverified locally; CI outcome pending** |
| 0.24 source snapshot | Rust AST grammar extended with conditionals, loops, function declarations, returns and nested blocks; new unit tests and smoke coverage | **Build and execution unverified** |
| 0.25 source snapshot | Rust AST contextual validator, new numbered errors N3008/N3009, CLI `validate`, test cases | **Unverified Rust build/CI** |
| 0.25 source snapshot | Rust contextual validator, N3008/N3009 errors, native `validate` CLI and authored integration tests | Rust compilation/CI **unverified** |
| 0.26 source snapshot | All 39 human keyword tables in Rust; redesigned familiar `ai` syntax, old-prefix migration, 40-profile source translator and test fixtures | **Rust build and parity checks unverified** |
| 0.27 source snapshot | Rust translator rejects target keyword collisions (N3011), five new integration tests, precise repair docs | Rust build / CI **not yet verified** |
| 0.28 source snapshot | Canonical IR v1 and CLI `ir`/`compare-ir`, eight unit tests and four integration tests | **Build and CI unverified; structure comparison is not semantics proof** |
| 0.29 source snapshot | Sequential builtin-shadowing in Rust IR, recursive function symbol handling and three authored regression tests | Rust build and tests **not verified** |
| 0.30 source snapshot | Conservative Rust IR type checking, N1001 mismatch diagnostic, CLI `typecheck`, eight unit and two integration tests, CI smoke | **Build/tests unverified; unknown types deferred** |
| 0.31 source snapshot | Rust lexical scope checks N1003/N1004, duplicate parameter detection, six unit tests and exact repair docs | **Rust build/tests unverified** |
| 0.32 source snapshot | N1005 known function arity checking, recursive call tests, CLI regression fixtures and CI smoke | **Build and tests unverified** |
| 0.33 source snapshot | Canonical profile test-key fix, arity invalidation on reassignment, two tests, non-mutating CI formatting gate | **Cargo build / CI still unverified** |
| 0.34 source snapshot | Reproducible `scripts/verify-rust.sh`, toolchain preflight, GitHub Actions manual dispatch and unified gate | **Execution blocked locally: Cargo unavailable; GitHub CI pass unverified** |
| 0.35 source snapshot | Conservative N1002 arithmetic type mismatch detection and six tests | **Rust build/tests unverified** |
| 0.36 source snapshot | Localized primitive type normalization in canonical IR; 40-profile typed parity test and Slovak checker regressions | **Build/test execution unverified** |
| 0.37 source snapshot | N1006 unary operand checks, five unit tests and CI path trigger correction | **Rust build/test execution unverified** |
| 0.38 source snapshot | Control-flow-sensitive known-function arity invalidation and lexical shadowing regressions | **Build/test execution unverified** |
| Next | Verified Rust CI, cross-profile Unicode/identifier parity and independent runtime | Incomplete |

- [x] Correct Rust IR symbol lowering so declarations shadow builtins only after their initializer, and function names remain visible for recursive references. Three regression tests authored; compilation remains unverified.

- [x] Add a conservative Rust static type checker over canonical IR for typed declarations, assignments, primitive expressions and nested blocks; expose `typecheck`, document N1001 and author regression tests. **Source committed, compilation unverified.**
- [ ] Add statement-level source spans, accurate locations in Rust semantic diagnostics, control-flow type joins, symbol tables and sounder type inference.

- [x] Implement conservative Rust lexical scope diagnostics for undeclared assignments (N1003), duplicate declarations and repeated function parameters (N1004); allow nested block shadowing and enclosing-scope assignment. Six additional unit tests authored, **not executed**.
- [x] Add conservative Rust IR function-call arity checking (N1005) for known direct calls, recursive calls, lexical parameter shadowing and nested expressions; six unit and two CLI tests authored. **Build/test execution unverified.**
- [x] Repair Rust regression tests that used non-canonical capitalized profile keys (`True`, `None` instead of `true`, `none`); prevent stale N1005 arity assumptions after a function variable is reassigned. Add two regression tests. **Build/test execution unverified.**
- [x] Add a read-only, reproducible Rust verification script with toolchain preflight, formatting, all-target tests, strict Clippy and positive/negative CLI fixtures; expose manual GitHub Actions dispatch. **Actual Cargo/CI pass is not yet verified.**
- [x] Add conservative Rust N1002 primitive arithmetic compatibility checks for nested expressions and function-call arguments, with six unit tests and exact repair guidance. **Build/test execution unverified.**
- [x] Canonicalize primitive type annotations (`int`, `float`, `str`, `bool`) when lowering localized declarations into Rust IR v1; add cross-profile structural equivalence and Slovak typed declaration regression tests. **Rust execution still unverified.**
- [x] Add conservative Rust N1006 numeric unary operand checks, five unit tests, actionable diagnostic documentation; ensure verifier-script edits trigger Rust GitHub Actions. **Build/test execution unverified.**
- [x] Conservatively invalidate known Rust function arity after potential reassignment in conditional branches and loops, without leaking local shadowing or iterator bindings; add four regression tests. **Build/test execution unverified.**
## Current acceptance-gate evidence (2026-11-08)

**Historical status before the first successful CI run: NOT VERIFIED.** The authoring environment has no Cargo, rustc or rustfmt and cannot access GitHub through its local shell. The GitHub connector used for source edits does not expose ordinary push-triggered Actions runs via its commit workflow lookup. A successful GitHub run is now independently confirmed: [#42501364426](https://github.com/siposboris/NOBODY/actions/runs/42501364426).

The existing Rust CI now runs `scripts/verify-rust.sh` first with an explicitly configured Rust toolchain and retains `rust-verification.log` as a downloadable artifact even if the gate fails. The gate was verified to pass on GitHub run #42501364426; future changes still require fresh CI. A targeted source review also corrected a preexisting arity-tracking error where lexical shadowing inside branches could discard an outer function signature; four new regression tests are authored but not yet executed.

**Progress accounting rule:** Do not add a completed checklist item, close an open checklist item, or claim a release milestone based solely on authoring tests or editing CI. The Phase 0 Rust verification item was closed after inspecting the successful run #42501364426 and job steps/logs. Totals are now **41 checked / 68 open**.

## Verified GitHub CI results (2026-11-08)

- **Reproducible Rust dependency graph:** [CI #42504352399](https://github.com/siposboris/NOBODY/actions/runs/42504352399) verified committed `rust/Cargo.lock`, Cargo `--locked` dependency resolution, formatting, tests, strict Clippy and CLI smoke. Python frontend [#42504352371](https://github.com/siposboris/NOBODY/actions/runs/42504352371) and lexer [#42504352340](https://github.com/siposboris/NOBODY/actions/runs/42504352340) also passed. This is not a release attestation nor bit-identical binary guarantee.

- **N2008 Unicode NFC canonicalization gate:** [Rust #42503823825](https://github.com/siposboris/NOBODY/actions/runs/42503823825) passed all Rust checks with pinned `unicode-normalization = "=0.1.24"`. Test coverage proves representative precomposed/decomposed names, unchanged strings/comments and translation safety; it does not certify all Unicode security concerns.

- **N2007 targeted identifier spoofing checks:** [Rust run #42503288105](https://github.com/siposboris/NOBODY/actions/runs/42503288105) passed all ten Unicode/RTL tests, full Rust suite, formatting, strict Clippy and CLI smoke. Python frontend [#42503288165](https://github.com/siposboris/NOBODY/actions/runs/42503288165) and lexer [#42503288178](https://github.com/siposboris/NOBODY/actions/runs/42503288178) passed. Unicode normalization and broad confusable coverage remain unimplemented.

- **Unicode/RTL regression gate:** [Rust CI #42502868233](https://github.com/siposboris/NOBODY/actions/runs/42502868233) passed six new tests in `rust/tests/unicode_lexing.rs`, along with format, compilation, all-target tests, Clippy and CLI smoke. Python frontend and lexer workflows also passed on the same commit. Scope is limited to named fixtures, not general Unicode correctness.

- **40×40 Rust profile matrix:** [run #42501821609](https://github.com/siposboris/NOBODY/actions/runs/42501821609) passed the two `profile_matrix.rs` tests on a representative function/condition/loop/typed-declaration fixture (1,600 ordered source→target pairs). **Structural IR comparison is not full semantics or natural-language quality certification.**

- **Rust frontend:** [run #42501364426](https://github.com/siposboris/NOBODY/actions/runs/42501364426) passed all jobs: `cargo fmt --check`, compilation, 109 library unit tests, integration tests, strict Clippy and CLI smoke. Source commit `e1437613639c5428babc0e6d406795215762fce4`.
- **Python reference frontend:** [run #42501583586](https://github.com/siposboris/NOBODY/actions/runs/42501583586) passed on Python 3.11, 3.12 and 3.13 after compatibility fixes. **Lexer:** [run #42501583602](https://github.com/siposboris/NOBODY/actions/runs/42501583602) passed.
- This does not prove full language correctness, full Rust/Python parity, platform portability or runtime implementation. Those roadmap items remain open.

## Latest work toward existing open items (2026-11-08)

- **Phase 1 lexer completeness (still open):** Reject malformed decimal literals such as `12abc`, `1__2` and `1e+` with N2006 and actionable repairs. Preserve decimal/exponent forms and `1-2` token separation. Three lexer unit tests and two CLI integration tests authored, with a malformed-number fixture.
- **Phase 0 Rust verification (still open):** The container has neither `cargo` nor `rustc`, and its network cannot resolve the official Rust toolchain host. No claim of a successful build, formatting gate, Clippy or GitHub Actions run is made. Do not remove the existing acceptance gate.
- **Accounting:** Existing roadmap totals stay unchanged. Fixes to partial work are not new completed milestones.

## Next engineering milestones — prioritized acceptance plan (updated 2026-11-08)

Roadmap checklist progress is **45/112 (40%)** as of this update. It counts marked checklist entries, including historical prototypes, and **must not** be treated as a percentage of a working language or production readiness. The README bar is generated from checkbox bullets by `scripts/update-roadmap-progress.py` and checked in CI. Counting is deliberately *unweighted*; do not claim that a checked lexer item is equivalent in engineering effort to a native backend or self-hosting.

### 0.39 target — small Rust evaluator foundation (partially implemented, CI verified)

1. Add a **separate, opt-in Rust AST/IR evaluator**, initially for integer/string/boolean/null literals, arithmetic, `let` bindings and `print`, with stdout capture for tests.
   - **Partial, verified:** [PR #6](https://github.com/siposboris/NOBODY/pull/6) merged basic Rust-only execution, opt-in `run`, simple untyped declarations and assignment, integer arithmetic, print and AI/Slovak fixture parity. Full runtime scope is still open. Keep `parse`, `validate`, `typecheck` and `check` as non-executing commands.
2. Define deterministic runtime-value and runtime-error types, including numbered Nxxxx diagnostics for division by zero, missing variables and unsupported operations. Avoid inventing source spans where IR does not contain them.
3. Add execution tests for a simple Slovak and `ai` program with identical observable results; cross-check supported fixtures against the legacy Python AST evaluator.
   - **Implemented within a narrow fixture:** AI and Slovak CLI stdout parity is tested. **Still missing:** an independently executed automated Rust-vs-Python reference comparison across shared fixtures and other profiles. Output must match while keyword syntax remains one language per file.
4. Before a new `run` CLI can claim support, verify guarded step/recursion budgets and document that these are **not** a security sandbox or hard memory/CPU caps.
5. **Gate:** `cargo fmt --check`, `cargo test --locked --all-targets`, strict Clippy, CLI smoke and Python-vs-Rust fixture comparison must pass in GitHub Actions.
   - **Verified gate subset:** [Rust GitHub Actions #42510492963](https://github.com/siposboris/NOBODY/actions/runs/42510492963) passed formatting, locked build/tests, Clippy, CLI evaluator smoke; Python frontend and lexer workflows also passed. **Full gate remains open** until differential parity and agreed safety boundaries are verified. Mark the evaluator partial until every claimed behavior has tests. Do **not** delete Python.

**Conditional execution slice (2026-11-08, verified):** Rust `run` implements `if` / `else if` / `else` blocks, integer comparisons, boolean short-circuit logic and lexical block shadowing. Eight additional integration tests and [Rust CI #42511789940](https://github.com/siposboris/NOBODY/actions/runs/42511789940) passed, and the change merged via [PR #8](https://github.com/siposboris/NOBODY/pull/8). This is a **partial runtime milestone**, not a completed Phase 4 checklist item. Loops, functions, full reference parity, and strong resource isolation remain open.

**While-loop execution slice (2026-11-08, verified):** The optional Rust `run` evaluator implements bounded `while` loops with boolean conditions, iteration-local lexical scopes and nested `break`/`continue`. Nine new integration tests cover AI/Slovak behavior, nested loop control and exhausted budgets; one legacy regression was updated for the newly supported feature. Full [Rust CI #42512435569](https://github.com/siposboris/NOBODY/actions/runs/42512435569) and all three other workflows passed, and changes merged in [PR #9](https://github.com/siposboris/NOBODY/pull/9). This remains partial runtime work; `for`, functions, native compilation, differential Python parity and secure resource isolation are not complete.

**Range-based for-loop execution slice (2026-11-08, verified):** The Rust `run` evaluator supports bounded `for` over 1–3-argument unshadowed `range`, positive and negative steps, lexical per-iteration variable scope, nested `break`/`continue`, and N4006 for zero step. Nine new integration tests cover these behaviors and iteration overflow/budget guards. All four GitHub workflows passed, including [Rust CI #42512795206](https://github.com/siposboris/NOBODY/actions/runs/42512795206), and the change merged in [PR #10](https://github.com/siposboris/NOBODY/pull/10). This is an incremental Phase 4 capability: generators, arbitrary iterables, functions, native compilation, full Python parity, and hard sandboxing remain open.

**Runtime syntax-profile matrix (2026-11-08, verified):** Two integration tests translate bounded `ai` execution fixtures (arithmetic/assignment/print and `while`/`if`) into all 40 profiles and compare their Rust `run` output. [Rust CI #42514669235](https://github.com/siposboris/NOBODY/actions/runs/42514669235) and all three other workflows passed; merged in [PR #12](https://github.com/siposboris/NOBODY/pull/12). This strengthens **narrow Rust runtime profile parity**, not complete semantics, native-speaker correctness or Rust-vs-Python differential equality. Full Phase 4 and 0.39 gates remain open.

### Next implementable milestone — functions and reference parity

The Rust `run` evaluator now has verified basic assignments, conditionals, bounded `while`, and `range`-based `for` loops. These are **partial** capabilities, not a complete runtime.

1. **Function values and calls:** implement named functions with explicit lexical environments, parameter arity, local scope and `return` control flow. Start with non-recursive integer/text functions; fail explicitly for unsupported calls.
2. **Function safety gates:** deterministic recursion/step budget, no unbounded native call stack, no shared caller-local leakage, and tests for shadowing, nested definitions, return behavior and arity.
3. **Differential verification:** run the same small compatible AI and Slovak fixtures under the Python reference and Rust evaluator; record expected stdout and error behavior in automated CI. Note semantic differences explicitly rather than silently accepting them.
4. **Preserve existing guarantees:** one syntax profile per file, checked integer math, numbered diagnostics, no implicit execution from `parse`/`typecheck`, no dependency on Python inside the Rust `run` implementation.
5. **Completion gate:** formatting, locked compilation/tests, strict Clippy, multilingual CLI smoke, and both Python workflows green. Do not mark full Phase 4 complete or remove Python on the strength of a single fixture.

**Named Rust functions (2026-11-08, verified):** The experimental `run` evaluator supports limited top-level named functions, parameter binding, `return` through conditional/loop control flow, runtime argument-count checking (N4007), function-local scopes and rejection of recursion. Calls read a snapshot of top-level variables; global writes in calls do not persist and caller-local variables cannot leak in. Ten new integration tests and updated regression coverage passed [Rust CI #42515102342](https://github.com/siposboris/NOBODY/actions/runs/42515102342) alongside frontend, lexer and roadmap checks; merged via [PR #13](https://github.com/siposboris/NOBODY/pull/13). Closures, nested function definitions, first-class functions, recursion, full reference parity and native compilation remain open. This is **not** complete Phase 4 or 0.39.

**Python → Rust differential execution gate (2026-11-08, verified narrow subset):** Six shared Slovak and English Nobody source fixtures are executed by the Python `ast-run` reference and the independently compiled Rust `run`; the differential step checks successful exit codes and identical stdout. All four GitHub workflows passed, including [Rust CI #42515578312](https://github.com/siposboris/NOBODY/actions/runs/42515578312); merged in [PR #14](https://github.com/siposboris/NOBODY/pull/14). The script checks both exit statuses and exact stdout, failing on disagreement. This is a **small verified-subset migration acceptance test**, not a claim of all-profile or all-feature parity. Preserve the Python implementation until the full compatibility matrix and end-to-end compiler features are independently verified. The longer-term goal remains a Nobody-written, independently self-hosted compiler; Rust is the intermediate execution/compiler implementation, not the final language.\n\n

**Compatibility discrepancy found by the first run:** the Slovak identifier `sucet` was accepted as a user-defined function name by Rust, but rejected by the Python reference because it is a reserved built-in keyword. The shared fixture now uses an unreserved identifier, and this discrepancy remains **open for Rust parser/profile alignment** (do not weaken the Python restriction just to make the tests green).

**Python-free user CLI (2026-11-08, verified):** A separate native Rust executable named `nobody` now shares the implemented `nobody-rs` command set. Its `run`, `languages` and `info` commands do not call Python. `languages` enumerates the 40 Rust-implemented syntax profiles (not the 249-entry planning catalogue); `info` identifies the Rust implementation and explicitly reports that native compilation/self-hosting are unavailable. `bin/nobody` invokes the compiled Rust executable or Cargo, with **no Python fallback**. The unchanged Python package remains as the reference test harness and retains legacy functionality not yet implemented in Rust (such as AST JSON and the full language catalogue). Native CLI and regression tests passed [Rust CI #42519156717](https://github.com/siposboris/NOBODY/actions/runs/42519156717), frontend, lexer and roadmap checks, and merged in [PR #16](https://github.com/siposboris/NOBODY/pull/16). A subsequent [PR #17](https://github.com/siposboris/NOBODY/pull/17) migrated profile direction/editor advice to Rust; [Rust CI #42519320129](https://github.com/siposboris/NOBODY/actions/runs/42519320129) and Python/lexer workflows passed. Do not remove Python before parity. This is a narrowly scoped reduction in Python dependence, not a complete language migration.

**Rust-only active-profile audit (2026-11-08, verified):** The native `nobody audit-languages` command checks the 40 Rust syntax dictionaries structurally without importing or executing Python. Checks include keyword arity, uniqueness, Unicode NFC, uppercase characters and forbidden bidi controls. All four checks passed including [Rust CI #42519710776](https://github.com/siposboris/NOBODY/actions/runs/42519710776); merged in [PR #19](https://github.com/siposboris/NOBODY/pull/19). The Python validator remains as a reference for its legacy profile catalogue, so this does not yet qualify any Python file for deletion. The separate migration inventory in [RUST_MIGRATION.md](RUST_MIGRATION.md) identifies seven partially ported workstreams and two major missing workstreams.\n\n### Migration stages toward Nobody-only self-hosting

**Stage A — Rust replaces Python for supported runtime behavior (ongoing):** Extend automated Python↔Rust differential fixtures from the six already verified examples to types, errors, scope/closure behavior, arithmetic edge cases, all relevant 40 profiles and compatible CLI commands. Require exact agreement for explicitly shared semantics and document intentional differences. Python remains a **test-only reference**, not a dependency of the Rust execution CLI.

**Stage B — Rust toolchain completeness (not done):** Complete parser, stable spec, source-span diagnostics, typed IR, modules, memory and resource-management design; implement and test bytecode/VM or native-code backend, dependency handling, packaging and reproducible releases. Only after rigorous parity and feature coverage should legacy Python runtime code be retired in a separate reviewed PR.

**Stage C — Nobody implementation (not started as a production toolchain):** Implement compiler/runtime modules in Nobody once the language can represent required data structures, IO, errors and memory behavior; use the proven Rust implementation as bootstrap compiler. Compare reproducible stage-1 (Rust-built) → stage-2 (Nobody-built) → stage-3 (Nobody-built) outputs and independently run the test corpus. Only then consider claiming a Nobody-only maintained toolchain.

**Safety constraint:** Do not delete the Python reference or claim self-hosting based on a small fixture suite. The top-level roadmap percentage remains based on the existing checklist items, not a measure of these stages.

**AST JSON v1 Rust export subset (2026-11-08, verified):** The native `nobody ast-json PROFILE FILE` command now exports the existing deterministic `nobody.ast.v1` JSON schema for **single-line source programs only**, with six English/Slovak Python-reference differential fixtures. All four CI workflows passed, including the exact-output AST JSON parity comparison in [Rust CI #42520969571](https://github.com/siposboris/NOBODY/actions/runs/42520969571), and the change merged in [PR #20](https://github.com/siposboris/NOBODY/pull/20). Rust AST nodes currently do **not** retain per-node source spans; this exporter intentionally refuses multiline source (N4004) instead of inventing line numbers. AST JSON import, validation, source re-emission, full expression grammar and multiline interchange **remain Python-only**. A green exact-output test would demonstrate only the documented single-line subset, not enough for removal of `nobody/ai_ir.py` or `nobody/ast_format.py`. The native CLI requires no Python; differential CI uses Python as the comparison baseline.

**Multiline Rust AST JSON source-line bridge (2026-11-08, verified narrow subset):** The native `nobody ast-json PROFILE FILE` exporter now derives actual AST node line numbers from the validated Rust syntax tree **and** the original lexer tokens. It supports a conservative multiline subset (declarations, calls, conditions, loops, functions) and returns N4004 when it cannot unambiguously associate nodes with source locations (for example, Python-incompatible `else if` or still-unmapped syntax). Four new multilingual multiline fixtures extend the exact Python ↔ Rust differential gate from six to ten. All four CI workflows passed, including the exact-output comparison in [Rust CI #42521825341](https://github.com/siposboris/NOBODY/actions/runs/42521825341); merged in [PR #21](https://github.com/siposboris/NOBODY/pull/21). This provides reliable line metadata for supported constructs, **not** full AST source spans or a spanned parser. JSON import, validation, source re-emission and the full grammar remain pending; `nobody/ai_ir.py` and `nobody/ast_format.py` stay as reference implementations. Python is never invoked by the Rust executable.

**Grouped-expression AST JSON line parity (2026-11-08, verified):** The Rust token-aligned source-line mapper now handles bounded, nested parentheses around complete expressions or their subexpressions by checking the validated AST shape and backtracking only token positions when grouping belongs to an inner operand. Three new Slovak/English multiline fixtures (grouped arithmetic, conditionals and call arguments) extend exact-output Python↔Rust AST JSON differential coverage from 10 to 13. All four CI workflows passed, including Rust formatting/build/tests, strict Clippy and exact AST JSON parity in [Rust CI #42522280632](https://github.com/siposboris/NOBODY/actions/runs/42522280632); merged via [PR #22](https://github.com/siposboris/NOBODY/pull/22). This remains a migration bridge, not a general source-span-bearing Rust AST or complete JSON import/export implementation.

### Following slices — only after the 0.39 gate

1. Port lexical scopes and closures with function calls and assignment; preserve typed writes, recursion safeguards and exact arity diagnostics across languages.
2. Extend flow-sensitive types (branch joins and loop writes) conservatively, with regression tests for false positive/false negative boundaries.
3. Add source-span-bearing AST/IR, semantic roundtrips and deterministic AST JSON interchange; keep unknown operations explicitly unsupported.
4. Extend the **implemented experimental** bytecode format, verifier, Rust VM and limited Linux x86-64 native backend to more language constructs; full compiler coverage and stable formats remain planned.
5. Compare reference and Rust behavior across the declared subset of all 40 experimental profiles, including RTL/Unicode fixtures, before considering a legacy Python component for removal.
6. After a mature Nobody runtime exists, attempt repeatable Nobody-written bootstrap stages 1 → 2 → 3. Native self-hosting is not claimed today.

## Progress accounting and refresh

Run `python3 scripts/update-roadmap-progress.py` after changing a checked or unchecked roadmap entry. CI runs `python3 scripts/update-roadmap-progress.py --check` to prevent README drift. The denominator may grow as requirements are split or newly discovered; percent changes are **not** a speed or quality metric. Build/test verification and release acceptance remain separate gates.

## Release acceptance criteria

Every release must contain source, tests, truthful supported-vs-planned feature status, exact build/run commands, documented known limitations, a complete numbered error catalogue, MIT license and AI assistance disclosure, a reproducible CI result, an updated English roadmap, checksums and a tagged version. Until these conditions are met, milestones are **development snapshots**, not audited production releases.
