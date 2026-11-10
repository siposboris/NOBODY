# Changelog

## Unreleased — Rust-native roadmap progress tool (2026-11-10; verified [PR #53](https://github.com/siposboris/NOBODY/pull/53))

- Added `nobody-roadmap` Rust checker and explicit `--write` generator matching the legacy Python checklist count, round-to-nearest percent and 20-cell display.
- Removed Python setup and invocation from the README roadmap GitHub CI job; only the Rust tool is required for routine progress verification.
- Kept the historical Python script as a comparison baseline until accepted by CI; this does not remove Python from every other migration task.


## Unreleased — cross-backend UTF-8 length parity for all 40 syntax profiles (2026-11-10; verified [PR #52](https://github.com/siposboris/NOBODY/pull/52))

- New fully generated cross-profile differential test checks canonical IR, static typecheck, AST evaluation, VM execution, deterministic on-disk `.nbc` v3 and decoded VM output in every registered syntax profile.
- Linux x86-64 CI also compiles and executes 40 standalone ELF programs, comparing Unicode scalar-length stdout against the independent Rust evaluator.
- Test scope is deliberately narrow; no claim of complete compiler equivalence, full Unicode segmentation or human review.


## Unreleased — Unicode string length in portable bytecode (2026-11-10; verified [PR #50](https://github.com/siposboris/NOBODY/pull/50))

- Added `TextLen` experimental v3 bytecode opcode 27, matching Rust AST and Linux ELF `len(text)` Unicode scalar semantics.
- Verified stack underflow, type and arity failures, and older-version opcode rejection; added stored-file `.nbc` vs AST/VM parity for `ai`, `sk`, `en`.
- Does not imply grapheme counting, safe execution of untrusted bytecode or complete runtime parity.


## Unreleased — localized length builtin static type contracts (2026-11-10; verified [PR #51](https://github.com/siposboris/NOBODY/pull/51))

- Static Rust checker infers `len(text)` result type `Int` and validates arity (N1005), known primitive operand types (N1002) and incompatible typed assignments (N1001).
- Unknown dynamic values remain accepted for runtime checking; tests cover `ai`, `sk`, `en`. No claim of complete type safety.


## Unreleased — portable UTF-8 string bytecode v3 (2026-11-10; verified [PR #47](https://github.com/siposboris/NOBODY/pull/47), [Rust CI #42829747456](https://github.com/siposboris/NOBODY/actions/runs/42829747456))

- Independent Rust bytecode compiler/VM supports `str` values, UTF-8 literals, concatenation, content equality and annotated `str` parameter/return checks.
- Experimental portable `NOBBC001` v3 adds bounded `PushText` opcode and type tag 3; v1/v2 decoding remains, and integer/bool-only files still serialize as v2.
- Disk-file CLI tests for `ai`/`sk`/`en` compare AST and in-memory VM execution with decoded v3 execution. Invalid UTF-8/corruption rejected; stable ABI, sandboxing and self-hosting remain open.


## Unreleased — all-40-profile runtime and real ELF differential regression (2026-11-10; verified [PR #48](https://github.com/siposboris/NOBODY/pull/48))

- Added generated nontrivial source programs for each of 40 syntax profiles and matching canonical IR, parser, static checker and Rust evaluator checks.
- Linux x86-64 acceptance suite compiles and runs all 40 variants as standalone native ELF, verifying stdout against the reference Rust AST interpreter.
- Negative tests reject a mixed syntax-profile keyword file. This is a machine-verified runtime subset, not a native-speaker review or proof of complete language parity.


## Unreleased — Unicode scalar text length in Rust evaluator and native ELF (2026-11-10; verified [PR #46](https://github.com/siposboris/NOBODY/pull/46))

- Implemented localized `len(text)` / Slovak `dlzka(text)`: counts Unicode scalar values, not UTF-8 bytes or grapheme clusters.
- Standalone ELF uses a native bounded byte-loop; output parity tests cover `ai`, `sk`, `en`, UTF-8, empty literals, text variables, and text function arguments/returns.
- Invalid arity or nontext argument produces numbered errors. This does not implement Unicode grapheme segmentation, FFI or full text indexing.


## Unreleased — native typed UTF-8 string function ABI (2026-11-10; verified [PR #44](https://github.com/siposboris/NOBODY/pull/44))

- Extended standalone Linux x86-64 native function calls for explicit `str` parameters and return values using two-word UTF-8 pointer/length arguments and `rax`/`rdx` return pairs.
- Mixed `str`, `int` and `bool` parameters, nested typed calls, text concatenation inside functions and conditional string returns.
- Real ELF-vs-Rust interpreter fixtures for `ai`/`sk`/`en`, with type-mismatch rejection. [Rust CI #42766777167](https://github.com/siposboris/NOBODY/actions/runs/42766777167) passed. ABI is internal/experimental; unannotated text parameter inference, recursion, closures and self-hosting remain open.


## Unreleased — read-only Rust formatter check (2026-11-10; verified [PR #42](https://github.com/siposboris/NOBODY/pull/42))

- Added `format-check PROFILE FILE.nbd`: non-mutating canonical formatting check with N5002 and nonzero exit status on drift, useful for CI.
- Added `ai`, `sk`, `en` CLI regression tests for success, failure and idempotence.
- All four CI workflows passed including [Rust CI #42765224570](https://github.com/siposboris/NOBODY/actions/runs/42765224570). This does not preserve original comments, implement LSP, or finish the whole formatter milestone.


## Unreleased — static text and boolean operator checks (2026-11-10; verified [PR #43](https://github.com/siposboris/NOBODY/pull/43))

- Rust `typecheck` now rejects statically known text ordering and non-boolean logical `and`, `or`, `not` (N1002).
- Unknown types remain deferred to runtime rather than creating false-positive diagnostics; Unicode text equality and concatenation still pass.
- All four workflows passed including [Rust CI #42766343202](https://github.com/siposboris/NOBODY/actions/runs/42766343202). Rust tests cover primary syntax profiles and nested expressions; full soundness and type inference remain open.


## Unreleased — native UTF-8 concatenation and equality (2026-11-10, verified [PR #41](https://github.com/siposboris/NOBODY/pull/41))

- Standalone Linux x86-64 ELF now performs `+`, `==` and `!=` on UTF-8 text literals and mutable text variables. Equality compares contents rather than pointers.
- Text joins allocate from a fixed 1 MiB process-lifetime arena without Python, libc, Rust or Nobody VM inside the executable. Arena exhaustion exits with status 7.
- New source fixtures and actual ELF-vs-Rust-evaluator parity tests for `ai`, `sk`, `en`, UTF-8, empty values, nested concatenation, mutable loops and function-local text.
- All four CI checks passed, including [Rust CI #42754693569](https://github.com/siposboris/NOBODY/actions/runs/42754693569). **Still open:** text arguments/returns, string ordering, GC, collections, all-platform native compiler, parity completion, Python removal and self-hosting.


## Unreleased — native UTF-8 text variables (2026-11-10; verified in [PR #40](https://github.com/siposboris/NOBODY/pull/40))

- Standalone ELF programs can now declare, copy, reassign and print UTF-8 text variables, including inside loops, conditional scopes and native functions.
- Native text bindings hold immutable read-only text data pointers and UTF-8 byte lengths in two independently allocated stack words; no runtime allocation or Python dependency.
- Actual compiled ELF output is compared with Rust interpreter output for `ai`, `sk`, `en`. Unsupported string concatenation, equality, text arguments/returns, typed declarations and value-type changes are rejected.
- All four CI checks passed including [Rust CI #42753533095](https://github.com/siposboris/NOBODY/actions/runs/42753533095). No broad roadmap checkbox is closed until complete acceptance criteria pass.


## Unreleased — real native UTF-8 text and mixed-argument output (2026-11-09)

- **Implemented and verified in [PR #39](https://github.com/siposboris/NOBODY/pull/39)**: standalone Linux x86-64 ELF now prints direct UTF-8 text literals, multiple numeric/boolean/text arguments separated by spaces, and empty `print()` calls, including inside loops.
- Text decoding shares the Rust evaluator's escape rules. Literal bytes are emitted as numeric directives in ELF read-only data to prevent source strings from becoming assembler instructions.
- New `native_text_print_ai.nbd`, `native_text_print_sk.nbd`, `native_text_print_en.nbd` fixtures are compiled into and **run as independent ELF processes**; their exact stdout must match the Rust evaluator. Existing standalone ELF regression suite still passes. [Rust CI #42697321428](https://github.com/siposboris/NOBODY/actions/runs/42697321428) passed, along with the three other workflows.
- **Still open:** dynamic string variables, string concatenation, text function parameters and results, recursion, whole-language native compilation, Python parity/removal and Nobody self-hosting. No extra broad roadmap checkbox claimed.


## Unreleased — first genuine native Linux x86-64 ELF compiler (2026-11-09)

- **Verified GitHub CI and merged [PR #28](https://github.com/siposboris/NOBODY/pull/28).** Rust now generates GNU Intel-syntax x86-64 assembly directly from a restricted Nobody IR subset; GNU `as` and `ld` link genuine standalone ELF64 machine code. **The resulting executable does not embed Python, Rust runtime, libc or the Nobody bytecode VM.**
- Added native CLI commands `compile-native PROFILE source.nbd output` and `native-asm PROFILE source.nbd`. The compiler refuses to overwrite existing outputs and uses a temporary private linking directory.
- Initial accepted subset: signed i64 integers, variables with fixed inferred types, checked `+`, `-`, `*`, boolean comparisons and `not`, `if/elif/else`, `while`, `break`, `continue`, and one-integer `print`.
- Added live Linux x86-64 ELF integration tests checking actual binary headers, direct process execution, exact stdout parity with the Rust evaluator across `ai`, `sk`, `en`, source-file independence, integer-overflow failure, finite empty-loop budgets, unsupported-feature rejection and no-overwrite semantics.
- All four workflows passed including [Rust CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375). Marked the **first-working-native-Linux-x86-64** roadmap criterion complete (**45/112, 40%**) without claiming a complete native backend.
- **Still missing:** native functions, `for range`, text, full i128 numbers, other architectures/operating systems, stable release binaries, an OS security sandbox and self-hosting. GNU binutils is required at compile time, not runtime. See [native backend guide](docs/NATIVE_X86_64.md).


## 0.8.0 — local experimental prototype (2026-11-08; not yet GitHub release)
- Strengthened language separation for one configured syntax language per source file, including short foreign keywords.
- Added source and target validation to the public translation API.
- Added five regression tests; 49 tests passed **locally**, not in GitHub CI.
- Complete prototype source import is pending. See [0.8 notes](docs/PROTOTYPE_0_8.md).

## Unreleased — GitHub documentation bootstrap (2026-11-08)
- Added complete staged roadmap, feature status table and MIT license.
- Added contributor/security guidance, release checklist and transparent AI disclosure.
- Initial import and verification of the prototype source code is still pending.

## Prototype history (local bundles; not verified GitHub Releases)
- **0.1** Basic interpreter and multilingual syntax experiments.
- **0.2** Nobody naming and prototype.
- **0.3** Configuration options and syntax language separation.
- **0.4** Experimental Nobody-written compiler core.
- **0.5** Nobody → Nobody source output.
- **0.6** Lowercase ASCII syntax.
- **0.7** Basic explicit typing and restricted immutable-only checking.

Do not confuse these historical notes with published and verified tags.


## Experimental 0.9 (local prototype; GitHub source import in progress)
- Added independent lexical scanner for lowercase ASCII syntax, strings, comments and source positions.
- Added lexer regression tests and integrated scanner validation into the local compiler frontend.
- 55 tests passed locally in the complete source tree; this is not a GitHub CI result.
- Scanner/parser is still incomplete and Python bootstrap remains.


## Experimental Nobody 0.10 — AST parser milestone (2026-11-08)
- Added independent recursive-descent AST parser (brace-based subset) and 10 regression tests.
- Supports basic declarations, functions, control flow, calls, lists and arithmetic operator precedence.
- Supports SK/EN/CS/DE syntax profiles with foreign keyword rejection.
- Added standalone GitHub frontend CI workflow (results not yet confirmed).
- Full local working tree passed 65 tests; GitHub does not yet have the entire interpreter/compiler source.
- Not a native compiler; typed declaration parsing and indentation grammar are not yet included.

## Experimental Nobody 0.11 — native script profiles (2026-11-08)
- Expanded syntax translation tables to **22 human languages** with native writing systems; no romanization for non-Latin scripts.
- Added Unicode-aware lexical scanning for letter/combining-mark sequences and native script keywords.
- Added Nobody-to-Nobody translation module, preserving string literals and comments.
- Added multilingual round-trip regression tests and GitHub Actions test configuration.
- Updated README, roadmap, parser and lexer docs and full [language catalog](docs/LANGUAGES.md).
- **80 local prototype tests passed** after adding five tests. The complete Python bootstrap remains an experimental prototype; The full local Nobody-written bootstrap compiler also round-tripped all 22 profiles (using the temporary Python bootstrap). GitHub CI outcome has not been verified.
- Native translations need native-speaker review; neither Unicode security nor full compiler integration is complete.

## Experimental Nobody 0.12 — RTL language safety (2026-11-08)
- Added Arabic/Urdu RTL display metadata with logical order guarantees.
- Added bidi-control validation to the Unicode lexical scanner, protecting against hidden source direction overrides.
- Added CLI `direction` in the complete local prototype and new regression tests.
- Local complete prototype: **86 tests passing**, including RTL scenarios; **no confirmed full GitHub CI result**.
- Added [RTL documentation](docs/RTL.md). External RTL editor integration remains a future milestone.


## Experimental Nobody 0.13 — language catalogue and AST execution (2026-11-08)
- Added 249-entry language/script-variant planning catalogue using a publicly reported August 2026 Google Translate list.
- Added six experimental profiles with original scripts: Hebrew, Greek, Ukrainian, Persian, Swahili and Dutch. 28 profiles can now be selected; other 221 catalogue variants remain unavailable.
- Extended RTL metadata to Hebrew and Persian.
- Implemented direct evaluation of a subset of Nobody AST, without generation of Python source. This interpreter still runs on Python.
- Added primitive annotated declarations and mismatch checks, a runnable GitHub frontend CLI and regression tests.
- **95 tests passed in the complete local 0.13 tree.** GitHub Actions status has not yet been verified.
- Full prior bootstrap importer, native backend, full language grammar and independent self-hosting are still planned.

- Added experimental profile validator and GitHub CLI `audit-languages`; 95 tests pass in local source archive. GitHub CI result remains unconfirmed.


## Experimental Nobody 0.14 (2026-11-08)
- Added six provisional syntax languages: Bulgarian, Romanian, Swedish, Finnish, Armenian, Georgian. 34 experimental total / 215 reference variants pending.
- Updated the local Nobody-written source-to-source bootstrap compiler's keyword tables for all 34 profiles.
- AST parser and evaluator now support assignment to previously declared variables, enabling mutable loop counters; unknown names fail.
- Added new tests and CI configuration. **104 tests passed locally** in the complete 0.14 archive, not confirmed on GitHub Actions.
- Known limitations: type constraints on reassignment, block-scoping, complete bootstrap source import, native compilation and native-speaker validation.


## Nobody 0.15 experimental development (2026-11-08)

- Added Nepali, Marathi, Gujarati, Kannada and Telugu provisional original-script syntax dictionaries.
- Added dedicated canonical `ai` syntax with explicit, stable `ai_` prefixes.
- Added `nobody.ast.v1` deterministic JSON AST output for machine-based developer tools.
- Added runtime enforcement of primitive explicitly declared types on reassignment, not only initial declaration.
- Updated local Nobody-written bootstrap compiler vocabulary to all 40 syntaxes (39 human + AI).
- Added six new regression tests: **110 passed in complete local development archive**, GitHub CI result unconfirmed.
- 249 human variants remain in reference catalogue; 210 human variants still have no usable Nobody syntax dictionary.
- Known limitations: parser grammar subset, Python-hosted interpreter, unverified native linguistic word choice, older bootstrap not yet mirrored in GitHub and no native self-hosting.


## Nobody 0.16 experimental milestone (2026-11-08)
- Added versioned AST JSON importer and validator with whitelisted nodes, child shape checking and explicit rejection of unsupported schemas.
- Added source emitter from AST to Nobody source for a subset of the grammar (selected SK/EN/AI/Arabic/Japanese/Nepali/Persian/German profiles tested locally).
- Added bounded-depth and malformed-input regression tests; 116 tests passed in the complete local 0.16 tree.
- Added `ast-validate` and `ast-emit` commands to the GitHub frontend and expanded frontend CI.
- Restrictions: Python bootstrap still required, no native executable backend, no formal proof of equivalence, incomplete grammar; GitHub CI success not yet confirmed.


## Nobody 0.17 experimental development (2026-11-08)

- Implemented nested lexical scope frames for variables; shadowing and assignment to outer variables are distinguished.
- User-defined functions now capture their lexical environment, enabling returned closures and recursive calls in the supported AST subset.
- Runtime annotations on captured variables remain enforced when they are reassigned in closures.
- Added bounded function call depth and eight regression tests.
- Local complete source tree: **124 tests passed**; GitHub CI not yet confirmed.
- Limitations: Python bootstrap and object memory semantics, incomplete grammar, no native machine code, no hard resource isolation or compiler self-hosting.


## Nobody 0.18 experimental milestone (2026-11-08)
- Added conservative AST static type checker to diagnose primitive type mismatches and invalid known-type operators **before execution**.
- Added undeclared assignment detection, lexical-scope awareness and diagnostic codes `E_TYPE`, `E_BINARY`, `E_UNDECLARED`, etc.
- Introduced `type-check` CLI and ten tests; the complete local 0.18 archive passed **134 tests**.
- Unknown function results, control-flow merges and generic types remain uncertain; static success is **not** a proof that runtime execution cannot fail.
- GitHub Actions outcome and complete historical bootstrap import have not yet been verified.


## Nobody 0.19 — branch-aware static typing (2026-11-08)
- Statics: conditional branches now use independent snapshots and conservative type joins; loops account for possibly zero iterations.
- Added `E_ARITY` diagnostics for direct calls to functions whose argument count is statically known.
- Added ten regression tests; **144 tests passed locally** across the complete development tree.
- GitHub source and CI workflow updated. Full GitHub test result not yet confirmed.
- Known limitations: Python dependency, incomplete static inference, cross-language keyword collisions, unimported historical bootstrap files and no native compiler.


## Nobody 0.20 — číslované a akčne vysvetlené chybové hlášky (2026-11-08)

- Pridali sme 24 stabilných N-kódov so samostatnou príčinou a postupom opravy a [dokumentáciu](docs/ERROR_CODES.md) s konkrétnymi príkladmi.
- Statické diagnostiky majú `.number` a `.detailed_message`, zachovávajú symbolické kódy kvôli spätnej kompatibilite.
- Nový CLI príkaz `errors [number]`; chyby argumentov majú `N9001`, bežné výnimky sú mapované na kategórie.
- Doplnených 12 testov; celá lokálna sada Nobody 0.20: **156/156 úspešných testov**. GitHub CI nezávisle neoverený.
- Obmedzenie: `N9999` stále funguje ako dočasná kategória pre neklasifikované staršie chyby, ktorým treba doplniť vlastné číslovanie.


## Rust-first architecture transition (2026-11-08)

- Began replacing Python as Nobody implementation language with a dependency-free Rust crate (initial source lexer, token positions and categorized N2001–N2005 error handling).
- Added a small Rust CLI `nobody-rs lex`/`check`; `check` validates **lexical syntax only**.
- Added separate [Rust GitHub CI](.github/workflows/rust.yml) and explicit [migration plan](docs/RUST_MIGRATION.md).
- **No Python files removed yet**: Python parser, translator, checker and interpreter have not passed parity gates against their future Rust replacements.
- Rust source authored and committed; local compilation unverified because `cargo`/`rustc` are not installed here. GitHub Rust workflow outcome unconfirmed.
- Long-term architecture: Rust bootstrap → native Nobody compiler → reproducible Nobody-in-Nobody self-hosting.


## Rust 0.23 development snapshot — English roadmap and partial AST (2026-11-08)

- Rewrote the **entire canonical roadmap in English** and reviewed each checkbox for claimed implemented status vs. pending verification.
- Added Rust typed AST types and a restricted recursive-descent/precedence parser for `sk`, `en`, and `ai` declarations, assignments, arithmetic/comparison and call expressions.
- Added seven Rust parser unit tests and separate SK/EN/AI source fixtures; Rust CLI now exposes `parse <lang> <file>`.
- Assigned N3006 and N3007 for unsupported constructs and Rust profile selection; documented exact causes and repairs.
- Updated GitHub Actions Rust workflow with parser smoke checks and added Rust-specific README.
- **Build/test not verified**: `cargo`/`rustc` unavailable in the current authoring environment. No Python modules removed; no native compilation, execution, or self-hosting claimed.


## Nobody Rust 0.24 — control-flow and function AST source (2026-11-08)

- Added Rust AST variants and parser logic for conditions, else-if/else branches, while/for loops, function declarations, returns, break/continue and nested brace-delimited blocks.
- Added Rust unit tests for SK/AI/EN grammar paths, missing closing braces (N3002), malformed signatures (N3003) and misplaced else (N3006).
- Extended GitHub Actions smoke tests to parse the existing `examples/hello_sk.nbd` and `examples/hello_ai.nbd`, as well as the smaller Rust fixtures.
- Updated the canonical English roadmap and error-code explanation to match the newly supported grammar.
- **Verification limitation:** the environment has neither Cargo nor rustc; compilation and GitHub Actions success are unconfirmed. This is **Rust parser source progress**, not a verified Rust compiler release.


## Nobody Rust 0.25 — contextual AST validation (2026-11-08)

- Added an independent Rust validation pass for function and loop-control context, without executing the program.
- Numbered new errors `N3008` (return outside function) and `N3009` (break/continue outside loop in current function); added precise causes, fixes and examples to the error catalogue.
- Added Rust `validate <sk|en|ai> FILE` CLI action, ten unit tests and four command-line integration tests with intentionally invalid Nobody fixtures.
- Improved diagnostics to avoid printing fabricated line 0 for AST nodes that do not yet carry source spans.
- Updated the English roadmap and CI smoke commands. **No Rust compilation or passing test run is claimed yet**, since toolchain/CI results are not confirmed.
- Python code has not been removed; parity gates still apply.


## Nobody Rust 0.26 — complete keyword-table port and familiar AI syntax (2026-11-08)

- Ported **all 39 experimental human syntax tables** and redesigned canonical `ai` profile into independent [Rust profile data](rust/src/profiles.rs), keeping original Unicode keyword spellings.
- Replaced old `ai_let`/`ai_if`/`ai_def` etc. in the **Rust `ai` dialect** with `let`/`if`/`fn`, plus `true`, `false`, `null`, `and`, `or`, `not` and support for `else if`.
- Added [token-preserving AI migration](rust/src/ai_migration.rs) and numbered **N3010** error with exact corrective command. Old programs retained separately.
- Added a preliminary [Rust Nobody-to-Nobody translator](rust/src/translator.rs) for all 40 profiles. It does **not** produce Python, and preserves strings/comments.
- Added full-profile keyword and native-script parsing tests, AI migration tests and CLI/CI smoke commands.
- Expanded Tamil combining-mark support in Rust UTF-8 lexer.
- **Verification:** The Rust crate could not be compiled in the authoring environment (`cargo`/`rustc` absent). No passing Rust test count is claimed. Full Unicode normalization, reserved-word semantics, interpreter/runtime and Python removal remain pending.

## Nobody Rust 0.27 — keyword collision safety (2026-11-08)

- Rust source translator now rejects non-keyword source identifiers that become reserved words in the target syntax, instead of emitting silently corrupted source.
- Introduced stable error **N3011**, containing the exact identifier, target profile, line, column and a direct rename repair.
- Added five Rust integration tests covering all 40 syntax profiles, lexical positions, false-positive avoidance for comments/strings and AI syntax translation.
- Added [translation-safety specification](docs/TRANSLATION_SAFETY.md) and updated the English roadmap.
- **Verification:** Cargo/rustc unavailable locally; Rust CI and actual test execution remain unconfirmed. Python reference retained pending parity.

## Rust 0.29 — IR declaration visibility (source snapshot, 2026-11-08)

- Fixed Rust IR lowering so a variable declaration does not shadow a builtin **before** its initializer or declaration position.
- Function names are visible inside their own bodies, including recursive references, while previous statements keep their earlier bindings.
- Added three Rust unit tests for before/after shadowing, initializer lookup and recursion.
- **Verification:** cargo/rustc unavailable in the authoring container; tests have not been executed. This remains a partial static IR, not a runtime or a semantic equivalence guarantee.

## Rust 0.30 — conservative static typing (source snapshot, 2026-11-08)

- Added `rust/src/typecheck.rs`: type checking for known primitive literals, typed declarations, subsequent assignments, numeric expressions, local branches and function bodies.
- Added `nobody-rs typecheck PROFILE FILE` CLI and documented N1001 exact mismatch diagnostics.
- Added eight unit tests, two CLI integration tests, good/bad AI fixtures and CI smoke steps.
- **Limitations:** unknown types deferred, no source spans in IR v1, no control-flow type joins or complete lexical symbol table. Cargo compilation and tests are **not verified**.

## Rust 0.31 — lexical scope diagnostics (source snapshot, 2026-11-08)

- Conservative IR checker rejects assignment to undeclared names with N1003 and duplicate declarations/repeated function parameters with N1004.
- Inner block shadowing and assignments to enclosing declared variables remain valid.
- Added six Rust unit tests; expanded numbered error documentation with exact fixes.
- **Cargo compilation and tests not yet verified**; IR source spans and full closure semantics remain future work.

## Rust 0.32 — known function arity validation (source snapshot, 2026-11-08)

- Rust IR type checker reports N1005 for calls to known functions with incorrect positional argument counts.
- Includes recursive calls, lexical parameter shadowing, nested expression traversal and deferred dynamic calls.
- Six new Rust unit tests, two CLI integration tests, good/bad fixtures and GitHub Actions smoke coverage authored.
- **Cargo build and CI remain unverified.** This does not implement full scope, closures, overloads or native runtime.

## Rust 0.33 — verification-focused regression repairs (source snapshot, 2026-11-08)

- Corrected AI profile tests to use the canonical lowercase semantic keys `true` and `none`.
- Fixed stale known-function arity metadata after assignment, preventing false N1005 diagnostics; added two regression tests.
- Changed Rust GitHub Actions formatting step to `cargo fmt --all -- --check` so CI detects formatting differences instead of silently modifying the checkout.
- **Rust compilation, formatting and test success are unverified.** No runtime or Python replacement is claimed.

## Rust 0.34 — reproducible verification gate (source snapshot, 2026-11-08)

- Added `scripts/verify-rust.sh`: non-mutating formatting check, all-target Cargo tests, strict Clippy, successful CLI examples and intentionally failing CLI examples.
- Added Rust toolchain preflight with explicit failure if Cargo, rustfmt or Clippy is unavailable.
- Enabled manual `workflow_dispatch` for the Rust GitHub Actions workflow and wired the unified verification script into CI.
- Shell preflight syntax was checked locally; the local environment has no Cargo, so **Rust build/tests/Clippy and GitHub CI remain unverified**. The roadmap acceptance gate remains open.

## Rust 0.35 — conservative arithmetic type validation (source snapshot, 2026-11-08)

- Added N1002 known-incompatible arithmetic operand detection in the Rust IR checker, including nested expressions, function-call arguments and return expressions.
- Added six unit tests and documented exact causes and fixes.
- Unknown types are deliberately deferred. **Cargo build, Clippy and CI results are not yet verified.**

## Rust 0.36 — localized primitive annotation normalization (source snapshot, 2026-11-08)

- Canonical IR lowering now normalizes localized primitive type names into `int`, `float`, `str`, `bool`, without rewriting original source text.
- Added structural IR parity checks for typed declarations across all 40 profiles and additional Slovak checker regression cases.
- Unknown annotation names are left untouched for future validation; complete semantic/type checking is not claimed.
- **Cargo build, test and CI execution remain unverified.**

## Rust 0.37 — numeric unary diagnostics and CI trigger fix (source snapshot, 2026-11-08)

- Added N1006 for definitely nonnumeric operands of unary plus/minus, including nested expressions.
- Added five unit tests and an actionable diagnostic entry.
- Rust workflow now triggers on changes to `scripts/verify-rust.sh`, as well as Rust sources and the workflow itself.
- **Cargo build, Clippy and GitHub CI results remain unverified.**

## Rust 0.38 — conservative control-flow function signature tracking (source snapshot, 2026-11-08)

- Known function argument counts are invalidated if a branch or loop can reassign the binding, preventing stale N1005 diagnostics after control flow.
- Lexical shadowing and loop iterator bindings are not propagated to enclosing scopes.
- Four new Rust regression tests cover branches, loops and shadowing.
- **Cargo build, formatting, Clippy and GitHub CI success remain unverified.**

## Unreleased Rust fixes — toward existing roadmap acceptance gates (2026-11-08)

- Corrected lexical-shadowing handling in static function arity analysis. Added four regression tests for nested branches, loops and function-body boundaries.
- Added strict decimal literal validation to the Rust lexer with N2006 and actionable documentation. Added three unit tests, a malformed-number fixture and two CLI integration tests.
- Improved GitHub Actions to preserve verification logs when the Rust gate fails.
- **No roadmap acceptance item was closed.** The authoring environment has no Rust toolchain and GitHub Actions success has not been independently verified; these are source snapshots, not a certified build.

## Unreleased — Unicode NFC identifier validation (2026-11-08)

- Added N2008: a Rust lexer diagnostic for identifiers that are not in Unicode NFC, without silently rewriting source bytes.
- Added regression coverage for decomposed Latin names, precomposed Unicode and noncomposing marks, preserved strings/comments, and translation rejection.
- Pinned `unicode-normalization` to `=0.1.24`. The core is no longer third-party-dependency-free; a committed `Cargo.lock` is still pending for reproducible dependency resolution.
- **Verified:** [Rust CI #42503823825](https://github.com/siposboris/NOBODY/actions/runs/42503823825) passed. The full Unicode security and profile verification roadmap gate remains open.

## Unreleased — locked Rust dependency graph (2026-11-08)

- Committed `rust/Cargo.lock` generated by Cargo in the real Rust GitHub Actions runner, with exact resolved versions and checksums for `unicode-normalization` and its `tinyvec` dependency.
- Made the Rust verifier and GitHub Actions use `--locked` for dependency resolution, tests, Clippy and CLI smoke.
- **Verified:** [Rust CI #42504352399](https://github.com/siposboris/NOBODY/actions/runs/42504352399) passed, as did the Python frontend and lexer workflows. This is not evidence of reproducible binary builds, distribution signing or release attestation.
- Existing release-acceptance roadmap item remains open: no new checklist checkbox was added.
