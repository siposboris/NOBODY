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
