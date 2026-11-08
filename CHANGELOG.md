# Changelog

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
