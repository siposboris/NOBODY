# Nobody — universal configurable programming language

**Status: experimental prototype. Not production-ready.**

Nobody is an independent open-source programming language project aiming to combine useful ideas from Python, Rust, C/C++, Go, Erlang, Haskell, Lisp, Julia, Ada/SPARK, Lean and more. Not all features can be active simultaneously.

## Core design rules
- **One human syntax language per source file** (no mixing Slovak, English, Czech, German keywords).
- **Lowercase ASCII syntax and identifiers** (without diacritics); Unicode is allowed in string literals and comments.
- Source-to-source translation: Nobody `.nbd` → Nobody `.nbd`, **not Python output**.
- Configurable typing, memory, performance, safety and target modes — **only implemented modes may run**. Planned options must fail clearly.
- Long-term aim: a compiler written in Nobody that generates native executables. Python is currently a temporary bootstrap runtime.

## Project status
The earlier Nobody 0.7 prototype was prepared outside this repository. Its code/tests are **not yet verified as a GitHub release**. This repository currently contains design documents, not a complete working release. See [feature status](docs/FEATURES.md).

## Documentation
- [Complete roadmap](docs/ROADMAP.md)
- [Feature implementation status](docs/FEATURES.md)
- [Version history](CHANGELOG.md)
- [Release process](docs/RELEASING.md)
- [Release template](docs/RELEASE_TEMPLATE.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [AI assistance](AI_DISCLOSURE.md)
- [MIT license](LICENSE)

## Attribution
Created as an independent project from its founder's requirements with software development and documentation assistance from **ChatGPT by OpenAI**. This is **not** an official OpenAI project, and OpenAI does not sponsor or certify it.


## Development update (2026-11-08)

[Experimental Nobody 0.8 notes](docs/PROTOTYPE_0_8.md), [example](examples/hello_sk.nbd), and [new regression tests](tests/test_language_barriers.py) are published. 49 tests passed in the full **local** tree, but the full source is not yet mirrored here. **No GitHub CI success or official GitHub v0.8 release is claimed.**


## Latest experimental development (2026-11-08)

A stand-alone [Nobody lexer](nobody/lexer.py) and [lexer unit tests](tests/test_lexer.py) are now committed. The complete working 0.9 prototype is available as a separate source archive in the development conversation but **has not yet been fully imported into this repository**. The complete local prototype passed 55 tests. GitHub CI for the complete interpreter has not run; do not interpret these commits as an official verified release.
