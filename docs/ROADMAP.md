# Nobody — complete development roadmap

**Updated:** 2026-11-08. **Status:** planned directions, not guarantees or shipped capabilities.

Nobody aims to be a configurable multilingual programming language with a common semantic core. The design borrows useful ideas from Python, C, C++, Rust, Go, Erlang, Haskell, Lisp, Julia, Ada/SPARK, Lean, SQL, Nix and other languages. Their features cannot all be enabled simultaneously: every profile must specify consistent semantics.

## Non-negotiable language rules

1. **One human language of syntax per source file.** No mixed keyword languages inside one file. Language is configured explicitly; translation must preserve program meaning.
2. **Lowercase syntax in the selected language's native writing system.** Latin-based profiles generally keep ASCII-friendly spelling (Vietnamese uses lowercase native diacritics); other scripts use original glyphs instead of romanization. Strings and comments may contain arbitrary Unicode. See [22 syntax languages](LANGUAGES.md).
3. **Nobody source → Nobody source.** A source translator must not silently output Python. A future native backend may produce executable machine code, but its *compiler implementation* should ultimately be written in Nobody.
4. No pretend switches: implemented configurations must change real behavior; unsupported configurations must produce explicit errors.
5. Never claim self-hosting, sandboxing, hard RAM guarantees, or native compilation before independently repeatable tests prove them.
6. Open source, tested changes, documented limitations, changelog per release, transparent acknowledgement of ChatGPT by OpenAI.

## Milestones

### Phase 0 — Foundation (v0.1–v0.7 prototype history; partially implemented, not independently re-audited in GitHub)
- [x] Experimental Python-based interpreter and Nobody syntax prototypes exist in earlier local release bundles.
- [x] Basic multilingual keyword sets and Nobody → Nobody rewriting were prototyped.
- [x] Configuration catalog of 22 option groups was prototyped; many choices are unsupported.
- [x] Preliminary typed declarations and restricted immutable-only checks were prototyped.
- [ ] Import exact tested v0.7 sources, examples and test suite to this repository; rerun CI before marking a GitHub release.
- [ ] Audit correctness, security and the claimed number of tests.
- [ ] Adopt versioned language specification and stability policy.

### Phase 1 — Reliable frontend (next)
- [ ] Formal lexical grammar with ASCII lowercase constraints and robust Unicode string/comment handling.
- [ ] Per-file syntax language header/config; reject foreign-language keywords deterministically.
- [ ] Tokenizer, parser, source-span tracking, useful errors and recovery.
- [ ] Common AST / IR independent of syntax language.
- [ ] Formatters, syntax-aware translation, round-trip equivalence tests and deterministic output.
- [ ] Test naming collisions, strings, comments, escape sequences and ambiguous translated words.
- [ ] Stable grammar for variables, functions, scopes, loops, conditionals, structured data and modules.

### Phase 2 — Strong semantics and typing
- [ ] Lexical scoping, closures, module/import system, packages and namespaces.
- [ ] Strict, inferred and gradual typing modes with precisely defined boundaries.
- [ ] Algebraic data types, enums, pattern matching, type inference and generics.
- [ ] Interfaces/traits, parametric polymorphism and coherent extension methods.
- [ ] Nullable/optional types, result/error types, safe arithmetic and bounds checks.
- [ ] Immutability and mutability semantics including deep immutability and aliasing.
- [ ] Static analyzer, linter, detailed diagnostics and source mapping.

### Phase 3 — Memory and resource-control profiles
- [ ] Automatic GC profile.
- [ ] Ownership/borrowing, move semantics and lifetime safety profile.
- [ ] Arena/region allocation, reference-counting and explicit manual allocation profiles.
- [ ] Safe interoperability boundaries among memory modes.
- [ ] Resource limits: allocation budgets, CPU time, recursion limits and deterministic cancellation.
- [ ] Distinguish tracked allocation budgets from OS-enforced process RAM caps.
- [ ] Sanitizers, leak/race detectors, fuzzing and validated memory safety guarantees.

### Phase 4 — Compiler and self-hosting
- [ ] Nobody-written compiler frontend with reproducible bootstrapping.
- [ ] Compiler IR, optimizations, debug info and platform-independent backend interface.
- [ ] Portable interpreter and deterministic bytecode VM.
- [ ] Native Linux x86-64 backend and executable/linker toolchain.
- [ ] ARM64, Windows, macOS and WebAssembly targets.
- [ ] Self-compilation stage 1 → stage 2 → stage 3; compare reproducible outputs.
- [ ] Eliminate Python from normal end-user compile/run path only once verified.
- [ ] Publish compiler binaries and verify checksums and reproducibility.

### Phase 5 — Performance and runtime profiles
- [ ] Debug, fast-build, balanced, maximum-speed, minimum-size and low-energy profiles.
- [ ] Inlining, constant folding, dead code elimination, vectorization, SIMD and profiling.
- [ ] CPU architecture selection and portable fallback.
- [ ] Predictable FFI with C; interoperable libraries.
- [ ] Deterministic concurrency/error handling for supported profiles.
- [ ] Performance benchmarks against representative C, Rust, Go, Python and Julia tasks, with reproducible methods.

### Phase 6 — Concurrency and distributed execution
- [ ] Async/await, structured concurrency and cancellation.
- [ ] Threads, lightweight tasks, message passing and channels.
- [ ] Actor isolation, supervisor trees and fault tolerance.
- [ ] Data race prevention and scheduling policies.
- [ ] Distributed processes, serialization, networking and secure service communication.

### Phase 7 — GPU, scientific computing and AI
- [ ] Dense arrays, tensor operations, linear algebra and multiple-dispatch options.
- [ ] GPU kernel language and CPU fallback.
- [ ] Portable GPU strategy where possible, backend-specific accelerators where necessary.
- [ ] Scientific numerics, reproducibility modes and floating-point policies.
- [ ] AI/ML integration with transparent runtime and hardware constraints.

### Phase 8 — Advanced language features and verification
- [ ] Hygienic macros and metaprogramming, staged/compile-time evaluation.
- [ ] Controlled effects and effect handlers.
- [ ] Design by contract, refinement/dependent-type experiments.
- [ ] Optional formal verification and machine-checkable proofs.
- [ ] Declarative query/collection APIs; DSL construction with explicit safety limits.

### Phase 9 — Developer experience and ecosystem
- [ ] Configurator UI with *supported/experimental/planned* states for every choice.
- [ ] Project and per-module profile negotiation with compatibility checking.
- [ ] Language server (LSP), autocomplete, debugger, formatter, refactoring.
- [ ] Reproducible package/build manager, lockfiles, dependency provenance and offline mode.
- [ ] Standard library (files, processes, network, crypto interfaces, collections, time, JSON).
- [ ] Security review, dependency scanning, release CI and signed/checksummed artifacts.
- [ ] Stable documentation in supported human languages; tutorials and migration tools.
- [ ] Hot reload for supported targets, incremental builds and editor mode switching.

### Phase 10 — Production maturity
- [ ] Published compatibility and specification tests.
- [ ] Measured security, reliability, performance and memory characteristics.
- [ ] External audit and contributor governance.
- [ ] Version 1.0 only when core language/runtime/tooling meet stated acceptance criteria.

## Configuration dimensions (not all implemented)

Human syntax language; simple/advanced/expert syntax; high/hybrid/low abstraction; inferred/explicit/gradual/dependent typing; automatic/ownership/arena/refcount/manual memory; strict/balanced/expert safety; interpreted/bytecode/JIT/native/WASM execution; CPU/GPU/hybrid parallelism; sequential/async/threads/actors/distributed concurrency; speed/size/energy/build-time optimization; platform and CPU; diagnostics; errors; immutability; arithmetic semantics; IO permissions; proof level; dependency resolution; reproducibility; build mode; RAM/CPU quotas. Each option requires tests documenting the exact observed behavior.

## Release acceptance criteria

Every release must include: source code, tests and reported results; supported vs planned feature matrix; changes, limitations and known security issues; exact run/build instructions; license and AI disclosure; updated roadmap; tagged version and checksums for downloadable artifacts. No version numbers or dates are promised in advance.


### Progress note — 2026-11-08
- [x] Prototype lexer implemented locally: lowercase ASCII identifiers, strings/comments, line/column positions and errors; lexer source and tests committed to GitHub.
- [ ] Complete source import and live GitHub CI test of 0.9 pending.
- [ ] Full independent parser, proper typed AST and formal specification still pending.


### Progress note — experimental Nobody 0.10 (2026-11-08)
- [x] First independently written **subset AST parser** with explicit source-language selection, brace blocks, function declarations, conditions, loops, calls, lists and binary expression precedence. Parser, language aliases and regression tests committed to GitHub.
- [x] Added Github Actions frontend test configuration. Successful GitHub workflow execution not yet confirmed.
- [ ] Finish complete grammar: typed declarations, reassignments, indentation syntax, richer expressions, unambiguous tokens and structured error recovery.
- [ ] Import complete interpreter/compiler bootstrap and all previous tests to GitHub.
- [ ] Implement execution/code generation from the independent AST, replacing Python syntax translation.


### Multilingual roadmap update — 2026-11-08
- [x] Commit syntax dictionaries for **22 human languages**, including native Chinese, Japanese, Arabic, Urdu, Russian, Hindi, Bengali, Korean, Tamil and Thai glyphs and Vietnamese accented Latin spelling. [Full language list](LANGUAGES.md).
- [x] Commit Unicode-aware lexical scanning and a Nobody-to-Nobody keyword translator with regression tests.
- [ ] Obtain native-speaker and grammar reviews for all profiles; ensure no hidden romanization remains.
- [ ] Formalize script-aware identifier rules, normalization, keyword boundaries, Unicode security and bidirectional display for Arabic/Urdu.
- [x] Local Nobody-written bootstrap compiler tables expanded from 4 to 22 languages; native-script output round-tripped in the local Python bootstrap. Full bootstrap source is not yet published to this GitHub repository.\n- [ ] Add end-to-end parser/runtime tests, language-specific diagnostics and complete compiler/runtime source import.
- [ ] Add further languages only after profile tests, documentation and native-speaker review.

### Experimental 0.12 progress — RTL source safety (2026-11-08)
- [x] Add direction metadata for native Arabic and Urdu (`rtl`); preserve logical Unicode source order.
- [x] Lexically reject invisible Unicode bidi overrides and isolate formatting controls across source, including strings/comments.
- [x] Add RTL regression tests, standalone GitHub CI configuration and [RTL specification](RTL.md).
- [ ] Implement real direction-aware editor rendering, cursor behavior and mixed-direction punctuation handling. **Not shipped.**
- [ ] Review Unicode normalization/confusables and native speaker translations.
- [ ] Import the full runtime and Nobody-written bootstrap compiler to GitHub and verify complete CI.


## Milestone 0.13 — expanded linguistic catalogue and AST execution (2026-11-08)

**Reference scope:** 249 Google Translate language/script variants according to the publicly reported August 2026 list (https://en.wikipedia.org/wiki/Google_Translate#Supported_languages). **Do not claim that all 249 are implemented.**

- [x] Store [249 named catalogue entries](../nobody/data/google_translate_249.txt), including variants in different scripts.
- [x] Record experimental/unsupported status for every entry in [catalog.py](../nobody/catalog.py).
- [x] Add six experimental working syntax dictionaries: Hebrew (`he`), Greek (`el`), Ukrainian (`uk`), Persian (`fa`), Swahili (`sw`), Dutch (`nl`). Total **28 active experimental dictionaries; 221 planned entries**.
- [x] Add RTL direction metadata for Hebrew and Persian alongside Arabic and Urdu; source remains in logical order.
- [x] Add a [Python-hosted interpreter that executes Nobody AST nodes](../nobody/ast_vm.py) without generating Python source; restricted functions, lists, loops, branches and primitive typed declarations supported.
- [x] Add [GitHub CLI frontend](../nobody/__main__.py) and [regression tests](../tests/test_013_catalog_vm.py).
- [ ] Review each new syntax dictionary with native speakers, test Unicode normalization, identifier collisions and language-specific tokenization.
- [ ] Implement the remaining 221 listed variants **one at a time**, each with native script, a complete keyword set and regression tests; reject unavailable profiles explicitly.
- [ ] Complete AST grammar and independent execution semantics. Python still hosts the compiler frontend and AST evaluator.
- [ ] Import full bootstrap interpreter and Nobody-written compiler into GitHub, with verified CI and release tags.
- [ ] Build a native code backend; independently verify self-hosting. No milestone date is guaranteed.

- [x] Add [language-profile auditing](../nobody/profile_validator.py) with completeness, Unicode normalization and lowercase checks; 95 tests pass locally. Native linguistic quality still needs human review.


## Milestone Nobody 0.14 — expanded profiles and AST assignment

- [x] Six additional candidate syntax dictionaries: Bulgarian `bg`, Romanian `ro`, Swedish `sv`, Finnish `fi`, Armenian `hy`, Georgian `ka`. **34 experimental profiles**, **215 catalogue variants still unavailable**.
- [x] Update both the standalone Github translator and local Nobody-authored bootstrap keyword tables for all six; validate round-trip translations.
- [x] Support assignment to existing variables in the experimental AST parser and evaluator (`pocet = pocet + 1`) and reject assignment to undeclared variables.
- [x] Add regression tests and GitHub CI configuration; **104 local tests passed**, separate from any GitHub CI result.
- [ ] Check assignment against annotated type/mutability policies and add lexical block scopes.
- [ ] Add full bootstrap sources to GitHub; finish self-hosting independently of Python.
- [ ] Review translation quality with native speakers; resolve cross-language keyword/identifier collisions and Unicode security cases.
- [ ] Implement the remaining catalogue entries, one profile at a time with full test coverage.


## Nobody 0.15 (2026-11-08) — five human scripts + machine-friendly syntax + runtime typing

- [x] Add five provisional native-script syntax profiles: Nepali `ne`, Marathi `mr`, Gujarati `gu`, Kannada `kn`, Telugu `te`.
- [x] Add `ai` canonical syntax profile: keyword names prefixed `ai_`; same semantics as the other human syntaxes. [Specification](AI_SYNTAX.md).
- [x] Add a stable, versioned, deterministic AST JSON exporter `nobody.ast.v1` for coding tools.
- [x] Check primitive explicit type annotations on reassignment in the AST interpreter (initial declaration and later assignments).
- [x] Add six regression tests in version 0.15 (110 tests passing in the full **local** archive).
- [ ] Implement full native-speaker terminology review for all human profiles, correct collisions and Unicode normalization.
- [ ] Implement a schema validator and AST-to-Nobody source re-emitter with semantic roundtrip tests.
- [ ] Fix lexical scopes, closure captures, type inference and generics; ensure typed variables retain annotations through all scopes.
- [ ] Import previous complete Nobody-language bootstrap compiler and test it on GitHub Actions.
- [ ] Complete real Python-independent runtime and native compiler, with externally reproducible proof of self-hosting.

**Current scope:** 39 human syntax profiles + AI profile = **40** total; **249** human language/script entries in reference catalogue, **210** still without a Nobody syntax dictionary. The `ai` profile is not a Google Translate language.


## Nobody 0.16 — AST validation and source regeneration (2026-11-08)

- [x] Implement [AST v1 JSON schema validation and import](../nobody/ast_format.py) with explicit allowed node types and fields, structured child checks, depth/node limits, and version rejection.
- [x] Implement `emit_source(tree, language)`: generate valid **Nobody** source for the supported AST subset, in the selected language. No Python source is generated.
- [x] Add [regression tests](../tests/test_016_ast_format.py) for multilingual round-trips, deterministic output and malformed AST rejection.
- [x] Add `ast-validate` and `ast-emit` CLI commands to the GitHub frontend; update GitHub Actions test configuration.
- [ ] Design a complete typed AST specification and ensure cross-language semantic identity for built-ins and identifiers.
- [ ] Complete the entire expression/statement grammar, lexical scoping and closures, and a formally verified emitter.
- [ ] Validate GitHub CI results and import the rest of the historical Nobody bootstrap source. The implementation still requires Python.

**Local full-project test run:** 116 tests passed on 2026-11-08; this is not a GitHub Actions result. Full AST round-trips only apply to the currently supported syntax subset. The complete native compiler remains unimplemented.


## Nobody 0.17 — lexical scopes and closures (2026-11-08)

- [x] Introduce [lexical environments](../nobody/scopes.py) that distinguish local declarations from surrounding scope bindings.
- [x] Nested blocks can shadow an outer variable without overwriting it; an assignment resolves to the nearest previously declared lexical owner.
- [x] Functions capture their definition environment and can return callable closures that retain captured variables after outer function return.
- [x] Primitive type annotations remain enforced when a closure updates a captured binding.
- [x] A recursion call-depth guard and execution step budget are present in the experimental Python-hosted AST evaluator.
- [x] Add [0.17 regression tests](../tests/test_017_scopes.py) and CI test configuration.
- [ ] Formalize shadowing semantics across all supported profiles and namespace rules, including per-language identifier conflicts.
- [ ] Improve closures, nested function lifetime, block-local captures, and avoid Python recursion/RAM hazards; interpreter remains **not a sandbox**.
- [ ] Import all historical bootstrap sources and run full GitHub Actions verification before declaring a tagged public release.
- [ ] Develop independent typed IR, separate static type checking and eventually a Python-free native compiler.

**Verification:** Full local source tree passed 124 tests (116 prior + 8 new). GitHub Actions success not yet independently confirmed. The feature is an interpreter milestone, not a native compiler.


## Nobody 0.18 — conservative static primitive typing (2026-11-08)

- [x] Implement a separate [AST static type analyzer](../nobody/static_types.py) that parses a Nobody program but never executes it.
- [x] Identify **provable** primitive errors in annotated declarations/reassignments and unsupported arithmetic between known incompatible types.
- [x] Detect assignments to undeclared names and repeated declarations within the same lexical scope; respect distinct nested block scopes.
- [x] Support the `ai` and human-language syntax profiles through the same parser/semantic core.
- [x] Add [10 regression tests](../tests/test_018_static_types.py), a CLI `type-check` command and GitHub Actions test configuration.
- [ ] Implement branch-sensitive flow analysis, function signatures and full return-type inference, generics, unions, nullability and advanced type checks.
- [ ] Precisely distinguish declared vs inferred mutability and model closures with complete static effect analysis.
- [ ] Confirm GitHub CI passes on a complete source import. Python remains the implementation language for this phase; native/self-hosted compiler still pending.

**Local archive:** 134 tests passed (124 previous + 10 new). Successful analysis does **not** guarantee runtime type safety. Unknown/dynamic expression types are not assumed to be safe.
