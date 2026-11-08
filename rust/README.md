# Nobody — Rust compiler frontend (experimental)

This is the Rust-first migration of the Nobody compiler frontend. Its goal is
eventually to replace the Python interpreter/compiler and later be rewritten in
Nobody itself. **Parsing does not mean running or native compilation.**

## Current source coverage

- **39 experimental human-language keyword profiles** plus a redesigned
  canonical **`ai`** syntax profile (40 total) in `src/profiles.rs`.
- UTF-8 lexical scanning with source byte offsets and common combining marks
  in `src/lib.rs`.
- AST parser supporting declarations, assignments, expressions, calls,
  functions, conditions, loops, returns and nested blocks in `src/syntax.rs`.
- Context validation for return/break/continue in `src/validate.rs`.
- A token-preserving converter for old `ai_` source in `src/ai_migration.rs`.
- Stable Nxxxx diagnostic categories for known source problems.

**Important:** Rust compilation and the complete 40-profile test matrix are
**not yet independently verified** in this authoring environment. Individual
profiles may still contain Unicode or translation problems; native-speaker
reviews are outstanding. The Python reference remains in the repository until
complete behavioral parity has been demonstrated.

## Build and check

With the stable Rust toolchain installed:

```bash
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path rust/Cargo.toml -- list-languages
cargo run --manifest-path rust/Cargo.toml -- parse sk examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai examples/hello_ai.nbd
```

### Redefined `ai` syntax

```text
fn twice(x) {
    return x * 2
}

let n: int = 2
if n > 1 { print(twice(n)) }
```

The keywords resemble widely used programming languages rather than
inventing new `ai_` terms. `ai` is still an ordinary Nobody **syntax
profile**, with no special runtime or autonomous AI capability. The design
has **not** been benchmarked for LLM token efficiency or coding accuracy.
The canonical AI dictionary has `let`, `fn`, `if`, `else`, `while`,
`for`, `in`, `return`, `true`, `false`, `null`, `and`, `or`,
`not`, `print`, etc. The Rust parser also recognizes `else if` as an
alternative to `elif` in the AI profile.

### Convert legacy AI source safely

```bash
cargo run --manifest-path rust/Cargo.toml -- migrate-ai examples/hello_ai_legacy.nbd > new_ai.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai new_ai.nbd
```

The command prints the transformed program to **stdout**; it does not
overwrite the original. It changes **known keyword tokens only**, leaving
comments, strings and unrelated identifiers unchanged. Using a recognized
legacy keyword without conversion produces **N3010**, documented in
[`ERROR_CODES.md`](../docs/ERROR_CODES.md).

## Commands

| CLI command | Purpose |
|---|---|
| `lex FILE` | Print source tokens with line/column information |
| `check FILE` | **Lexer-only** checks |
| `parse PROFILE FILE` | Create an experimental AST, supports all 40 profile tables |
| `validate PROFILE FILE` | AST parse plus return/break/continue context checks |
| `list-languages` | Display implemented Rust keyword profile codes |
| `migrate-ai FILE` | Print converted legacy-prefixed AI source |

## Still outstanding

- Formal Unicode normalization and confusable detection, native-speaker reviews,
  full foreign-keyword collisions and source-span tracking for AST statements.
- Complete grammar, static typing, true function/runtime behavior,
  source-to-source translation, bytecode, native backend and self-hosting.
- Successful reproducible Rust compilation and CI, differential comparison with
  Python behavior and subsequent safe removal of Python modules.

[Canonical English roadmap](../docs/ROADMAP.md) ·
[AI syntax specification](../docs/AI_SYNTAX.md) ·
[Migration gates](../docs/RUST_MIGRATION.md).

## 0.27 — cross-language keyword collision protection

The Rust `translate FROM TO FILE` command now reports **N3011** if a legal source identifier would become a target-profile reserved keyword. For example, Slovak `nech fn = 3` cannot translate to AI `let fn = 3`. Rename `fn` in the source to a name not reserved in the destination profile, then retry. See [translation-safety specification](../docs/TRANSLATION_SAFETY.md) and [error catalogue](../docs/ERROR_CODES.md).

The check preserves strings/comments and reports source line/column, but full scope-aware renaming and semantic parity are still planned. The Rust build and test results are not yet independently verified.

## 0.30 — Rust static type checks

`cargo run --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/typecheck_good_ai.nbd` checks known primitive types and declarations without executing the program. `typecheck_bad_ai.nbd` intentionally fails with documented **N1001**. Unknown expressions are deferred; this is not a sound whole-program checker. The source is committed but Cargo tests/CI remain **unverified**. See [error catalogue](../docs/ERROR_CODES.md).

## 0.31 — lexical scope diagnostics

The Rust `typecheck` command now also reports **N1003** (assignment to undeclared name) and **N1004** (duplicate declaration or repeated function parameter). Nested blocks may shadow enclosing names. Source and six additional tests are committed; Rust compilation/CI results remain unverified. IR statement source spans, imports and full closure semantics are not yet implemented.

## 0.32 — known function arity

The Rust `typecheck` command also checks direct calls to lexically known functions for argument-count mismatch (**N1005**). Example: `fn add(a,b) { return a+b }` followed by `add(1)` is rejected with an actionable repair. Recursive calls are checked, but dynamic and unknown callees are deferred. Six unit and two CLI tests are authored; Cargo/CI success remains unverified.

## 0.33 — regression hardening

The Rust profile tests now use lowercase canonical semantic keys; the IR type checker invalidates previously known function arity when that variable is reassigned. Two regression tests were added. CI now runs `cargo fmt --all -- --check` without mutating files. **Formatting, compilation and tests have not been independently verified.**

## 0.34 — verification gate

Run `bash scripts/verify-rust.sh` from the repository root. The script checks toolchain prerequisites, runs `cargo fmt --check`, `cargo test --all-targets`, strict `cargo clippy`, and successful/failing CLI smoke cases without changing source files. The Rust GitHub Actions workflow also supports manual dispatch. **This is a verification harness, not proof that the crate compiles**: the authoring environment lacks Cargo and the CI result remains unverified.
