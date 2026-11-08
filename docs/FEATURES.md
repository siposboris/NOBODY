# Nobody — feature status

**Prototype** means implemented in earlier local bundles but not yet re-audited on GitHub. **Planned** means not implemented. A configuration switch is *not* proof a feature works.

| Capability | Status | Notes |
|---|---|---|
| 22 human syntax profiles | Experimental | Native scripts for non-Latin languages; one syntax language per file. [List](LANGUAGES.md) |
| Lowercase native-script syntax | Experimental | ASCII-friendly Latin spelling except Vietnamese accented lowercase; non-Latin scripts supported. Strings/comments keep Unicode |
| Variables, conditions, loops, functions | Prototype | Limited core |
| Nobody → Nobody source translation | Prototype | .nbd output |
| Nobody-written compiler core | Prototype | Python bootstrap still required |
| Basic inferred/explicit type checks | Prototype | Limited coverage |
| Restricted immutable-only mode | Prototype | Limited constructs |
| Catalog of 22 option families | Prototype | Many unimplemented |
| Formal parser, AST/IR, modules, generics | Planned | |
| Rust-style ownership, GC and manual memory profiles | Planned | |
| Native x86-64/ARM64 and WASM backends | Planned | |
| GPU/tensors, scientific math, SIMD | Planned | |
| Async, threads, actors, distribution | Planned | |
| Hygienic macros and formal proof modes | Planned | |
| LSP, debugger, package manager, reproducible builds | Planned | |
| Native self-hosted toolchain without Python | Planned | |
| Hard resource isolation and security sandbox | Planned | |

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
| Testing | 93 tests passed in full **local** 0.13 source tree; separate GitHub CI outcome unconfirmed |

[Source catalogue](../nobody/data/google_translate_249.txt) · [Implementation statuses](../nobody/catalog.py) · [Full roadmap](ROADMAP.md)
