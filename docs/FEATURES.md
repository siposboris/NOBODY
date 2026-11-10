# Nobody — feature status

**Prototype** means implemented in earlier local bundles but not yet re-audited on GitHub. **Planned** means not implemented. A configuration switch is *not* proof a feature works.

| Capability | Status | Notes |
|---|---|---|
| 39 human syntax profiles plus `ai` | Experimental | Rust frontend profile dictionaries; one syntax language per source file, native scripts retained. [List](LANGUAGES.md) |
| Lowercase native-script syntax | Experimental | ASCII-friendly Latin spelling except Vietnamese accented lowercase; non-Latin scripts supported. Strings/comments keep Unicode |
| Variables, conditions, loops, functions | Prototype | Limited core |
| Nobody → Nobody source translation | Prototype | .nbd output |
| Nobody-written compiler core | Prototype | Python bootstrap still required |
| Basic inferred/explicit type checks | Prototype | Limited coverage |
| Restricted immutable-only mode | Prototype | Limited constructs |
| Catalog of 22 option families | Prototype | Many unimplemented |
| Multi-file project compiler with explicit dependency manifests | Experimental |
| Localized source-level module imports in v2 projects | Experimental | 40 syntax profiles; mandatory manifest-direct import graph, explicit direct symbol visibility; global names remain unnamespaced |
 Rust-only bounded `nobody.project.v1` graph loader; per-file syntax profile, IR translation validation, portable bytecode and Linux x86-64 ELF builds. No source-level import syntax or module namespaces yet |
| Formal parser, AST/IR, modules, generics | Planned | Full grammar, language-level modules, generics and formal semantics remain incomplete |
| Rust-style ownership, GC and manual memory profiles | Planned | |
| Linux x86-64 native ELF machine-code backend | **Verified experimental subset** | Rust IR → x86-64 assembly → GNU `as`/`ld` → standalone executable. Supports signed i64/boolean arithmetic and named functions, variables, conditions, while and for-range, plus UTF-8 text literals and mixed arguments in `print`. Dynamic text variables still unsupported. [Details](NATIVE_X86_64.md); [CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375). Not a full compiler |\n| ARM64, Windows, macOS and WebAssembly native targets | Planned | No native machine-code backend verified for these targets |
| GPU/tensors, scientific math, SIMD | Planned | |
| Async, threads, actors, distribution | Planned | |
| Hygienic macros and formal proof modes | Planned | |
| LSP, debugger, package manager, reproducible builds | Planned | |
| Native self-hosted toolchain without Python | Planned | |
| Hard resource isolation and security sandbox | Planned | |

### Current confirmed native output (2026-11-09)

A [limited Rust x86-64 backend](NATIVE_X86_64.md) now creates **real Linux ELF64 executables**, not interpreter wrappers. GNU binutils produces machine code that executes independently of Python, Rust, libc or the Nobody bytecode VM. All four workflows passed in [Rust CI #42638140375](https://github.com/siposboris/NOBODY/actions/runs/42638140375). The native subset includes signed i64 numeric expressions, assignments, booleans and comparisons, lexical variables, if/else, while/break/continue and one-argument integer output. **No** native for-range, functions, text, complete language coverage, other platform targets, release distribution or self-hosting yet.

See [full roadmap](ROADMAP.md).


## Experimental 0.8 progress

A local prototype now adds strict short foreign-keyword checks and validates both ends of Nobody-to-Nobody source translation. 49 tests passed locally, but the complete code has not yet been imported here. See [0.8 notes](PROTOTYPE_0_8.md). This does not change the status of native compilation, runtime, memory safety or full self-hosting.


## Local 0.9 development
- **Implemented in local prototype:** independent lexer for ASCII identifiers, Unicode strings/comments, source location tracking, and lexical errors. The [lexer source](../nobody/lexer.py) and [unit tests](../tests/test_lexer.py) are published in GitHub.
- **Not yet complete in repository:** full 0.9 interpreter, integration files, complete tests and CI execution.
- **Still planned:** full AST parser, native compilation, hard memory profiles, GPU, parallel execution and complete self-hosting.


## Experimental 0.10 milestone (2026-11-08)
- **Now committed:** [AST parser](../nobody/parser.py) for supported brace-syntax subset, [parser regression tests](../tests/test_parser.py), [frontend CI](../.github/workflows/frontend-tests.yml).
- **Local tests:** 65 passed in the complete local 0.10 tree. **GitHub CI:** results not verified.
- **Missing:** typed declarations in parser, assignments, indentation blocks, full grammar, complete interpreter/compiler import, native compilation and Python-free self-hosting.


### Native-script translation implementation (experimental)

The GitHub repo now contains 22 syntax dictionaries, a Unicode-aware lexer, source-to-source translator and regression tests. Their precise keyword terminology still requires native-speaker review. See [LANGUAGES.md](LANGUAGES.md). GitHub CI run result is not yet verified.

## Experimental 0.12 RTL profile support

- **Implemented in GitHub frontend:** Arabic and Urdu `rtl` display metadata; logical order preservation, prevention of hidden Unicode bidi formatting controls during lexical scanning and [standalone tests](../tests/test_bidi.py).
- **Not implemented:** automatic RTL paragraph rendering or bidirectional cursor controls in external editors. [Details](RTL.md).
- **Local prototype:** 86 tests passed; this is not GitHub CI confirmation.


## Experimental Nobody 0.13 — implementation truth table

| Component | Actual state |
|---|---|
| Google Translate reference list | 249 named language/script variants; **catalogue only** |
| Working experimental syntax dictionaries | 28 (22 previous + Hebrew, Greek, Ukrainian, Persian, Swahili and Dutch) |
| Pending syntax dictionaries | 221 catalogue entries; deliberately unavailable |
| Original scripts | Used in active native-script profiles; no forced transliteration |
| RTL direction metadata | Arabic, Urdu, Hebrew, Persian; **external editor support is not automatic** |
| Source translator | Translates recognized Nobody keywords between active dictionaries, preserving literals |
| AST parser | Subset: expressions, function calls, branches, loops, annotated declarations |
| AST interpreter | Python-hosted; does **not generate Python source**, but still depends on Python |
| Native compiler and true self-hosting | Not implemented |
| Security / hard memory quotas | Not implemented; not a sandbox |
| Testing | 95 tests passed in full **local** 0.13 source tree; separate GitHub CI outcome unconfirmed |

[Source catalogue](../nobody/data/google_translate_249.txt) · [Implementation statuses](../nobody/catalog.py) · [Full roadmap](ROADMAP.md)


## Nobody 0.14 feature state

| Feature | State |
|---|---|
| Syntax dictionaries | **34 experimental** (including six new in 0.14); native-speaker review incomplete |
| Google Translate reference variants | **249 in catalogue**; **215 not implemented** as Nobody syntax |
| Nobody-to-Nobody source compiler | Local prototype with 34 keyword tables; complete bootstrap not yet mirrored to GitHub |
| AST parser/evaluator reassignment | Implemented for existing variables; unknown variables rejected |
| Enforcing initial primitive type annotations | Experimental; reassignment typing not yet enforced |
| Full static typing, native compilation, Python-independent runtime | Planned |
| Tests | **104 local tests passed** in complete 0.14 archive; GitHub CI unverified |


## 0.15 experimental development status

| Capability | Current state |
|---|---|
| Human syntax profiles | **39 experimental**; five more native scripts added |
| AI syntax profile | **1 experimental** `ai` profile with lowercase ASCII canonical words |
| 249 Google language/script reference variants | 39 mapped to a syntax profile; **210 unmapped** |
| AI-compatible AST JSON | Deterministic schema tag `nobody.ast.v1`; no schema validator/roundtrip loader yet |
| AST runtime type guard | Checks explicit primitive types on declarations **and reassignments** |
| Full static type system, memory safety and native self-hosting | Not implemented |
| Test results | 110 tests passed in full local 0.15 tree; GitHub CI not yet confirmed |

See [AI syntax](AI_SYNTAX.md), [language list](LANGUAGES.md), [roadmap](ROADMAP.md).


## Nobody 0.16 new capabilities

- **AST JSON import/validation:** implemented for `nobody.ast.v1` subset; reject malformed schemas and invalid AST shapes, impose depth/count limits.
- **AST-to-Nobody source emitter:** implemented for supported statements and expressions; chooses a configured syntax language and does not produce Python source. Round-trip coverage is limited to examples and supported grammar, not a formal equivalence guarantee.
- **CLI:** `ast-json`, `ast-validate`, `ast-emit`.
- **Local tests:** 116 passed in 0.16 complete local tree. GitHub Actions status not yet verified.
- **Still missing:** full lexer/parser spec, cross-language canonical semantic IR for builtins, lexical scoping, closures, native compilation and self-hosting.


## Nobody 0.17 — lexical environment status

- **Implemented in AST interpreter:** lexical block scopes, local shadowing, outer-scope assignment, first-class closure values, recursive calls, runtime primitive annotation checks for captured variables.
- **Bounded recursion:** configurable interpreter call depth plus step count; not a secure resource-isolation guarantee.
- **Local verification:** 124 tests passed in full 0.17 development tree. GitHub CI results are not independently confirmed.
- **Not implemented:** comprehensive module namespaces, lexical captures in native machine code, static ownership checking, independent Python-free runtime, deterministic hard memory limits.


## 0.18 — conservative static typing

| Capability | State |
|---|---|
| Separate AST type analysis without code execution | Implemented, restricted subset |
| Type-annotated declarations and reassignment checks | Provably mismatched primitive types are diagnosed |
| Lexical scope-aware duplicate/undeclared checks | Implemented for supported AST nodes |
| Warnings for dynamically unknown identifiers | Implemented; unknown does not imply type safety |
| Full function type inference, branch joins, generics and sound static typing | Not implemented |
| Native compiler, formal proofs or secure sandbox | Not implemented |

Full local source archive: 134 unit tests passed. GitHub CI status unconfirmed.


## Nobody 0.19 — static analysis enhancements

| Feature | Status |
|---|---|
| Independent alternative path type joining for `if/elif/else` | Experimental implementation |
| Loop type joins including zero iterations | Experimental implementation |
| Known named-function argument count checking | Experimental implementation; `E_ARITY` |
| Inference for dynamic function objects and return values | Planned |
| Complete source import, independent IR and native self-hosting | Not implemented |

144 tests passed in local complete 0.19 tree; GitHub Actions result not yet verified.


## Nobody 0.20 — diagnostiky

| Schopnosť | Stav |
|---|---|
| Stabilné číselné diagnostiky Nxxxx | 24 experimentálnych kategórií v katalógu |
| Detailná príčina, kontext a postup opravy | Implementované pre katalogizované prípady |
| Pôvodné symbolické kódy `E_*` | Zachované |
| Výpis `nobody errors 1001` | Implementovaný |
| Všetky historické chyby bez `N9999` | Zatiaľ nie |
| Lokalizované diagnostiky pre 39 ľudských profilov a `ai` | Plánované |

Dokumentácia: [ERROR_CODES.md](ERROR_CODES.md). Kompletný lokálny zdrojový balík: 156 úspešných testov; stav GitHub CI neoverený.


## Rust-first implementation status (2026-11-08)

| Part | Rust status | Python retirement |
|---|---|---|
| UTF-8 lexical scanner | Partial source port; **build not yet verified** | Not permitted yet |
| Original-script language tables | Not ported | No |
| AST parser, validator and source emitter | Not ported | No |
| Static types, closures and function semantics | Not ported | No |
| Runtime and native compiler | Not ported | No |
| Nxxxx diagnostics | N2001–N2005 included in Rust lexer | Partial |
| AI `ai` profile | Not ported | No |

See [Rust migration acceptance criteria](RUST_MIGRATION.md). The project **must not be described as Rust-only or Python-free** until parity gates pass.


## Rust 0.23 source snapshot

| Capability | Implementation | Verification |
|---|---|---|
| Complete English roadmap with audited checklist | [ROADMAP.md](ROADMAP.md) | Committed |
| Rust AST parser for declarations and expressions (`sk`, `en`, `ai`) | [syntax.rs](../rust/src/syntax.rs) | Code authored; **compilation/CI not yet verified** |
| Rust CLI `parse <lang> FILE` | [main.rs](../rust/src/main.rs) | Source committed |
| All other 37 syntax profiles in Rust | Not implemented | Planned |
| Rust parsing of functions, conditions and loops | Not implemented | Planned |
| Rust interpreter/native backend and Python removal | Not implemented | Planned |

The Python reference remains experimental and is **not** the target runtime. See [Rust migration contract](RUST_MIGRATION.md).


## Rust 0.24 AST parsing capabilities

| Capability | State |
|---|---|
| SK / EN / AI tokenization and basic AST | Rust source implementation, unverified build |
| Branching: if, else-if, else | Rust parser source implemented |
| While and for loops | Rust parser source implemented |
| Function declarations, return, break, continue | Rust parser source implemented |
| Runtime execution of these Rust AST nodes | Not implemented |
| All 39 human syntax profiles in Rust | Not implemented |
| Static typing, bytecode, native backend and self-hosting | Not implemented |
| CI and compilation | Workflow updated, successful run **not yet confirmed** |

The complete [English development roadmap](ROADMAP.md) is the source of truth for remaining work.


## Rust 0.25 — partial contextual semantics

| Capability | State |
|---|---|
| Context checks for return, break, continue | Implemented in Rust source; unverified compilation |
| Function lexical boundary respected by loop-control validation | Implemented in Rust source |
| Numbered N3008/N3009 diagnostics with exact fixes | Implemented in catalogue |
| Native `validate` CLI and integration tests | Authored; build and test results unverified |
| Accurate per-node line/column AST spans | Still planned; validator does not invent them |
| Rust runtime execution and complete type checking | Not implemented |

This is not a Python replacement yet. [Rust migration rules](RUST_MIGRATION.md) still govern deletion of legacy Python components.


## Rust 0.26 — language-profile and AI syntax migration

| Capability | Current status |
|---|---|
| 39 human syntax tables + new `ai` in Rust | Rust source data committed; tests authored but build unverified |
| Original native-script keyword spellings | Preserved in Rust profile data, requiring native-speaker and Unicode review |
| Revised recognizable AI syntax (`let`, `fn`, `if`, `return`) | Rust parser source implementation |
| Legacy `ai_` keyword migration | Token-aware Rust converter; preserves strings and comments |
| Numbered N3010 migration diagnostic | Implemented and documented |
| Source-to-source syntax translation for 40 profile tables | Preliminary Rust implementation, not full grammar-aware semantics |
| Runtime for all 40 languages | Not implemented in Rust |
| Python deletion | Blocked pending parity and successful builds |
| Verified Rust compile, tests and CI | **Not yet confirmed** |

[Canonical AI syntax](AI_SYNTAX.md) · [Rust migration criteria](RUST_MIGRATION.md).

## 0.27 translator safety

The Rust translator rejects non-keyword source identifiers that are reserved in the selected destination syntax with error **N3011**. It includes specific source location and a concrete fix. **Implemented in source, Cargo build not verified.** A future compiler IR must replace conservative token translation to safely handle scope/hygiene and guarantee semantics. [Details](TRANSLATION_SAFETY.md).
