# Nobody — Development Roadmap

**Last reviewed:** November 8, 2026  
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
- [ ] Port complete typed AST/IR semantics, JSON AST validator, deterministic source emitter and true semantic round-trip validation to Rust. The new IR v1 is in-memory only.
- [x] Implement a prototype Nobody-to-Nobody translation tool in the Python reference that preserves strings and comments.
- [x] Implement preliminary Rust **Nobody-to-Nobody** keyword translation using token spans, preserving strings and comments, with roundtrip unit tests across selected native-script profiles.
- [x] Add a Rust **N3011 collision guard**: reject non-keyword source identifiers that coincide with reserved words in the destination profile; include line/column and concrete rename instructions, plus unit/integration tests.
- [ ] Extend Rust translation to become fully syntax-aware for lexical scope, automatic hygienic renaming, keyword ambiguities, Unicode normalization and verified semantic equivalence across all 40×40 profile pairs.
- [ ] Finish complete grammar: declarations, types, assignments, functions, lexical scope, collections, structured data, imports and modules.
- [ ] Build formatter, useful source diagnostics and parser error recovery.

## Phase 2 — Semantic analysis, typing and error reporting

- [x] Prototype lexical scopes, captured closures and basic runtime type annotations in Python.
- [x] Prototype static primitive type checks, lexical-scope diagnostics, conditional path joins and known-function arity checks in Python.
- [x] Prototype conservative function return-type inference, including unknown results when branches or dynamic calls cannot be proved, in Python.
- [x] Introduce an **append-only numbered error catalogue** in the Python frontend and [documentation with causes and repairs](ERROR_CODES.md).
- [ ] Port scopes, closures, type analysis, return inference and all applicable **Nxxxx** diagnostics to Rust.
- [x] Add precise **N1007** diagnostics for unknown explicit primitive type annotations in Rust parser and IR type checker, with six regression tests and preserved Slovak/AI primitive keywords. Verified by [Rust CI #42508306873](https://github.com/siposboris/NOBODY/actions/runs/42508306873) and merged in [PR #2](https://github.com/siposboris/NOBODY/pull/2). This does not implement user-defined types or complete static typing.
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

## Phase 4 — Compiler pipeline, runtime and self-hosting

- [ ] Implement a complete Rust parser and typed intermediate representation with reusable passes.
- [ ] Build a portable Nobody bytecode format, verifier and deterministic VM in Rust.
- [ ] Build compiler optimizations, debug information and a platform-neutral backend interface.
- [ ] Produce a working **Linux x86-64 native executable** with a tested linker/toolchain.
- [ ] Add Linux ARM64, Windows, macOS and WebAssembly targets with test suites.
- [ ] Remove Python from the normal compile/run workflow after Rust parity and stable integration tests.
- [ ] Rewrite compiler/runtime components in Nobody after the language can express them.
- [ ] Demonstrate self-hosting with reproducible stage 1 → stage 2 → stage 3 compiler builds and matching outputs.
- [ ] Distribute documented, verified binaries and checksums. **No native/self-hosting milestone has yet been met.**

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

## Immediate priorities (in dependency order)

1. **Completed initial verification gate:** Rust crate compiles; formatting, unit/integration tests, Clippy, and smoke pass on GitHub run #42501364426. Continue enforcing this gate on every future Rust change; do not infer full language parity.
2. Extend the now-compiled Rust parser beyond its currently tested subset and independently verify full frontend parity across profiles. **Parsing is not execution.**
3. **Compile and verify** the now-committed 40 Rust language tables, AI migration and source translation using the complete Rust test suite; then compare fixture outputs against the Python reference.
4. Port the canonical AST, source emitter, scopes and static analyzer; implement executable behavior only after Rust parses it.
5. Remove superseded Python modules in separate changes after parity is demonstrated.
6. Develop bytecode/native backends, then self-host the compiler in Nobody.

## Release acceptance criteria

Every release must contain source, tests, truthful supported-vs-planned feature status, exact build/run commands, documented known limitations, a complete numbered error catalogue, MIT license and AI assistance disclosure, a reproducible CI result, an updated English roadmap, checksums and a tagged version. Until these conditions are met, milestones are **development snapshots**, not audited production releases.
