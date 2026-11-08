# Rust → Nobody migration contract

**Decision (2026-11-08):** Rust is the temporary production-compiler implementation language. The long-term goal is a Nobody compiler and runtime written in Nobody itself. Python is legacy reference material, not the target runtime.

## Hard rules

1. **New core implementation work goes to Rust**, not to newly written Python modules. Existing Python code is temporarily retained for comparison and to prevent data loss.
2. Port modules **by observable behavior**: preserve documented syntax (39 human profiles + `ai`), native scripts, logical Unicode order, Nxxxx error numbers, type semantics and source-to-source translation.
3. Each component must pass `cargo test`, an integration fixture suite and a Python-vs-Rust differential check for relevant legacy examples *before* the Python equivalent is deleted. Differential tests may temporarily invoke Python in CI, but shipping Rust binaries must not depend on it.
4. Mark a component **Rust-replaced** only when every feature it replaces is implemented, tests agree and the Rust CLI exposes an equivalent user-facing path. Otherwise mark **partial**.
5. Remove superseded Python files with a separate, reviewed Git commit **only after** the compatibility gate passes. Do not silently delete earlier user work or claim Python-free self-hosting prematurely.
6. Port Rust components into Nobody itself after the Nobody compiler/runtime can express them; perform reproducible stage-1 → stage-2 → stage-3 bootstrap tests before retiring Rust.

## Current migration inventory

| Component | Python reference | Rust replacement | Status |
|---|---|---|---|
| Unicode source tokenization | `nobody/lexer.py`, `nobody/bidi.py` | `rust/src/lib.rs` | **Partial Rust port**: byte offsets, strings/comments, native-script names, N2001–N2005. Not yet complete Unicode equivalence |
| Human and AI language profiles | `nobody/languages.py` | Planned Rust tables | Python remains reference |
| Parser and typed AST | `nobody/parser.py` | Planned Rust parser | Python remains reference |
| Source-to-source translation | `nobody/translator.py` | Planned Rust translator | Python remains reference |
| AST interpreter/scopes/closures | `nobody/ast_vm.py`, `nobody/scopes.py` | Planned Rust VM | Python remains reference |
| Static type analysis | `nobody/static_types.py`, `nobody/return_types.py` | Planned Rust checker | Python remains reference |
| Error-code catalogue | `nobody/errors.py` | N2001–N2005 in Rust lexer | Partial; all other numbered codes pending |
| AST JSON schema/emitter | `nobody/ai_ir.py`, `nobody/ast_format.py` | Planned Rust IR | Python remains reference |
| Bootstrap Nobody compiler | `compiler/bootstrap.nbd` and legacy bootstrap | Rust native compiler frontend/backend pending | Self-hosting not proved |

## Build current Rust prototype

Install a Rust toolchain on your own machine, then from the repository root:

```bash
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo run --manifest-path rust/Cargo.toml -- lex examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- check examples/hello_sk.nbd
```

**`check` currently checks lexical validity only**, not full parse or type correctness. The executable is an experimental lexer frontend, not a complete compiler.

The generation environment used for the first Rust commit did not contain `rustc` or `cargo`; the source was authored and published but **could not be compiled or tested locally**. A GitHub Actions workflow exists for remote verification; its outcome is not yet confirmed.

## Next steps

1. Establish successful GitHub Rust CI, fix compiler/linter errors without masking failures.
2. Port the language table and Unicode lexer with the missing combining-mark ranges and exact source spans; differential regression suite.
3. Implement Rust parser and AST, including the canonical `ai` profile.
4. Port numbered errors, source translator and static checker.
5. Port runtime with scopes and closures, then bytecode and native backend.
6. Remove Python files **individually** when their gate passes, and finally remove the Python CI/bootstrap requirements.
7. Rewrite core compiler in Nobody, demonstrate native self-compilation and reproducible outputs; then retire Rust if fully replaced.


## 0.23 incremental update

A small Rust AST parser is now authored in [`rust/src/syntax.rs`](../rust/src/syntax.rs) and exposed via `nobody-rs parse sk|en|ai FILE`. Parser coverage is intentionally restricted to variable declarations, optional primitive annotations, assignments, literals, basic operators and call syntax. Seven Rust unit tests and three CLI fixtures were added, but **none have been verified as passing in the authoring environment** (Rust toolchain absent). This is **partial migration** only. Python parser and other modules must remain until all existing reference behavior is replaced and differential checks pass.


## Rust 0.24 migration update

The Rust parser now has AST node definitions and source parsing for branching, loops, functions, returns and nested blocks for `sk`, `en` and `ai`. This extends the 0.23 subset but **still does not establish feature parity** with the existing Python parser, language tables, translator or runtime. No Python component is eligible for deletion yet.

The Rust build is not confirmed because the local toolchain is unavailable; GitHub Actions must be checked before accepting any further component as verified. New diagnostic categories still require stable Nxxxx numbers and specific documentation.


## 0.25 incremental migration update

The Rust compiler frontend now has a **standalone contextual AST validator**, capable of checking whether `return`, `break` and `continue` are legal within the current function/loop nesting. The validator adds N3008 and N3009 to the append-only error catalogue, plus CLI integration tests. This is still only a **partial** replacement of the Python reference parser and evaluator.

Rust compilation and CI must be demonstrated before marking the component tested. No historical Python file is eligible for removal yet.
