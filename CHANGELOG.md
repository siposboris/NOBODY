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
