# Nobody — complete development roadmap

**Updated:** 2026-11-08. **Status:** planned directions, not guarantees or shipped capabilities.

Nobody aims to be a configurable multilingual programming language with a common semantic core. The design borrows useful ideas from Python, C, C++, Rust, Go, Erlang, Haskell, Lisp, Julia, Ada/SPARK, Lean, SQL, Nix and other languages. Their features cannot all be enabled simultaneously: every profile must specify consistent semantics.

## Non-negotiable language rules

1. **One human language of syntax per source file.** No mixed keyword languages inside one file. Language is configured explicitly; translation must preserve program meaning.
2. **ASCII lowercase syntax and identifiers** (a-z, 0-9, underscore where grammatically valid). No diacritics or uppercase in keywords/built-in names/identifiers. Unicode is allowed inside strings and comments.
3. **Nobody source → Nobody source.** A source translator must not silently output Python. A future native backend may produce executable machine code, but its *compiler implementation* should ultimately be written in Nobody.
4. No pretend switches: implemented configurations must change real behavior; unsupported configurations must produce explicit errors.
5. Never claim self-hosting, sandboxing, hard RAM guarantees, or native compilation before independently repeatable tests prove them.
6. Open source, tested changes, documented limitations, changelog per release, transparent acknowledgement of ChatGPT by OpenAI.

## Milestones

### Phase 0 — Foundation (v0.1–v0.7 prototype history; partially implemented, not independently re-audited in GitHub)
- [x] Experimental Python-based interpreter and Nobody syntax prototypes exist in earlier local release bundles.
- [x] Basic SK/EN/CS/DE keyword sets and Nobody → Nobody rewriting were prototyped.
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
