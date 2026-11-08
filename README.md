# Nobody

Experimental **open-source programming language** with configurable syntax profiles for human languages and a shared, language-neutral core. Licensed under MIT. **Not production-ready.**

<!-- roadmap-progress:start -->
### Roadmap checklist progress — 39%

**44 / 112 roadmap checklist items marked complete** (68 still open)

`████████░░░░░░░░░░░░` **39%**

This measures **completed checklist entries**, not the percentage of a working compiler, feature parity, quality, or production readiness. Some checked items are historical prototypes or partial milestones. Nobody is experimental; its Rust runtime, native compiler and self-hosting are not complete.

See the [full roadmap](docs/ROADMAP.md) and [feature status](docs/FEATURES.md). Regenerate with `python3 scripts/update-roadmap-progress.py`.
<!-- roadmap-progress:end -->

## Current state

- **40 experimental syntax profiles** — 39 human languages plus canonical `ai`. One syntax language per source file.
- **Rust frontend** — lexer, parser subset, source translation, canonical IR and conservative type checks.
- **Experimental Rust `run`** — variables, integer arithmetic, `print`, conditions, bounded loops and **basic named functions with parameters and `return`**. Closures, recursion, complete semantics, native compilation and self-hosting are not ready.
- **Primary CLI is Rust:** run the compiled `nobody` executable without Python for the documented subset. The Python reference remains **test-only** until full parity is proven.

## Try Nobody

Requires Rust/Cargo. Run from the repository root:

```bash
bash scripts/verify-rust.sh
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- languages
cargo run --locked --manifest-path rust/Cargo.toml --bin nobody -- run ai rust/examples/eval_basic_ai.nbd
```

Do not run untrusted programs; the evaluator is not a security sandbox.

## Documentation

[Roadmap](docs/ROADMAP.md) · [Features](docs/FEATURES.md) · [Rust guide](rust/README.md) · [Changelog](CHANGELOG.md) · [Errors](docs/ERROR_CODES.md) · [Historical notes](docs/README_ARCHIVE.md) · [License](LICENSE)

Independent project developed with assistance from **ChatGPT by OpenAI**; not an OpenAI product or endorsement.
