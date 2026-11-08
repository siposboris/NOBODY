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
