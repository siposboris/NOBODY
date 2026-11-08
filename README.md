# Nobody

**Experimental open-source programming language** with configurable human-language syntax and a shared canonical representation. Built independently under the MIT license; **not** production-ready and **not** an official OpenAI product.

<!-- roadmap-progress:start -->
### Roadmap checklist progress — 39%

**44 / 112 roadmap checklist items marked complete** (68 still open)

`████████░░░░░░░░░░░░` **39%**

This measures **completed checklist entries**, not the percentage of a working compiler, feature parity, quality, or production readiness. Some checked items are historical prototypes or partial milestones. Nobody is experimental; its Rust runtime, native compiler and self-hosting are not complete.

See the [full roadmap](docs/ROADMAP.md) and [feature status](docs/FEATURES.md). Regenerate with `python3 scripts/update-roadmap-progress.py`.
<!-- roadmap-progress:end -->

## What works today

- **40 experimental syntax profiles:** 39 human languages plus canonical `ai`. A source file uses one selected syntax profile.
- **Rust frontend:** Unicode lexer, subset parser, language-to-language syntax translator, canonical IR and conservative static checks.
- **Rust execution prototype (0.39 development):** opt-in `run` for basic untyped declarations, assignment, integer arithmetic, text/boolean/null values and `print`; still **missing** full control flow, functions, runtime parity, native compilation and self-hosting. Do not run untrusted programs.
- **Legacy Python reference** retained for comparison until parity is verified.

## Quick start

Requires the Rust toolchain. From the repository root:

```bash
bash scripts/verify-rust.sh
cargo run --locked --manifest-path rust/Cargo.toml -- list-languages
cargo run --locked --manifest-path rust/Cargo.toml -- run ai rust/examples/eval_basic_ai.nbd
cargo run --locked --manifest-path rust/Cargo.toml -- run sk rust/examples/eval_basic_sk.nbd
```

The evaluator examples print `9` and `8` on separate lines. The `run` command is deliberately limited; `parse`, `validate` and `typecheck` do **not** execute source code.

## Project links

[Roadmap](docs/ROADMAP.md) · [Feature status](docs/FEATURES.md) · [Rust guide](rust/README.md) · [Error codes](docs/ERROR_CODES.md) · [Changelog](CHANGELOG.md) · [Historical development notes](docs/README_HISTORY.md) · [MIT license](LICENSE)

Developed with assistance from **ChatGPT by OpenAI**. The project is independent; OpenAI does not sponsor or endorse it.
