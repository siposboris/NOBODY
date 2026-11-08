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
- [ ] Independently verify Rust compilation, formatting, Clippy and CI; the development environment used for initial Rust authoring has no `cargo` or `rustc`.
- [ ] Import **all** tested legacy bootstrap sources and corresponding fixtures to GitHub without accidentally discarding work.
- [ ] Establish release tags, checksums, CI-attested builds and reproducible acceptance tests.
- [ ] Remove Python from the regular execution path **only after** Rust replacements are fully tested.

## Phase 1 — Lexer, syntax, parser, AST and translation

- [x] Implement a native UTF-8 token scanner *prototype* in Rust for names, literals, symbols and comments, with byte spans and native-script letters; expose `nobody-rs lex` and lexer-only `check`.
- [x] Assign numbered Rust lexer diagnostics N2001–N2005 for string, case, combining-mark, unsupported-glyph and bidi-control errors.
- [ ] Complete the Rust lexer against Unicode normalization, combining characters, RTL, confusables, token boundaries and reference regression fixtures.
- [ ] Port all 39 human syntax profiles and one `ai` profile into Rust, preserving original script, lowercase rules, conflict handling and deterministic language selection.
- [ ] Build a full Rust recursive-descent/Pratt parser with source spans, precedence, diagnostics and structured AST. **A small incremental Rust parser subset is the next development target; it is not full grammar support.**
- [x] Implement a limited Python-reference AST parser for declarations, assignments, expressions, functions, loops, calls and blocks, with experimental 0.10–0.21 extensions.
- [x] Implement JSON `nobody.ast.v1` export/import validation and Nobody-source re-emission for a subset in the Python reference.
- [ ] Port language-neutral typed AST/IR, JSON AST validator and deterministic source emitter into Rust, including round-trip equivalence checks.
- [x] Implement a prototype Nobody-to-Nobody translation tool in the Python reference that preserves strings and comments.
- [ ] Port source translation to Rust; test reserved-word collisions, escaped literals, filenames, identifiers and round trips across all supported syntax profiles.
- [ ] Finish complete grammar: declarations, types, assignments, functions, lexical scope, collections, structured data, imports and modules.
- [ ] Build formatter, useful source diagnostics and parser error recovery.

## Phase 2 — Semantic analysis, typing and error reporting

- [x] Prototype lexical scopes, captured closures and basic runtime type annotations in Python.
- [x] Prototype static primitive type checks, lexical-scope diagnostics, conditional path joins and known-function arity checks in Python.
- [x] Prototype conservative function return-type inference, including unknown results when branches or dynamic calls cannot be proved, in Python.
- [x] Introduce an **append-only numbered error catalogue** in the Python frontend and [documentation with causes and repairs](ERROR_CODES.md).
- [ ] Port scopes, closures, type analysis, return inference and all applicable **Nxxxx** diagnostics to Rust.
- [ ] Implement strict, inferred and gradual typing modes with explicit boundaries and stable semantics.
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
- [ ] Port and validate `ai` syntax + AST interchange in Rust; benchmark whether they actually help AI coding tools.
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
| Next | Verified Rust CI, incremental parser, syntax profile port, differential tests | Incomplete |

## Immediate priorities (in dependency order)

1. Compile the Rust crate and confirm GitHub CI; fix errors rather than weakening checks.
2. Add a **small, tested Rust AST parser** for explicitly selected `sk`, `en` and `ai` syntax, with numbered errors and explicit unsupported features. **Do not mistake this subset for completed frontend parity.**
3. Bring all language tables and Unicode lexer behavior into Rust; run automated fixture comparisons with the Python reference.
4. Port the canonical AST, source emitter, scopes and static analyzer; implement executable behavior only after Rust parses it.
5. Remove superseded Python modules in separate changes after parity is demonstrated.
6. Develop bytecode/native backends, then self-host the compiler in Nobody.

## Release acceptance criteria

Every release must contain source, tests, truthful supported-vs-planned feature status, exact build/run commands, documented known limitations, a complete numbered error catalogue, MIT license and AI assistance disclosure, a reproducible CI result, an updated English roadmap, checksums and a tagged version. Until these conditions are met, milestones are **development snapshots**, not audited production releases.
