# Nobody Rust frontend (experimental)

This crate is the **early Rust migration** of the Nobody compiler frontend.
It does **not** compile native programs, execute Nobody, infer static types or
fully replace the original Python-language parser.

## Build and test

Install the Rust stable toolchain (`rustc` and `cargo`) and run from the repository root:

```bash
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo test --manifest-path rust/Cargo.toml --all-targets
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path rust/Cargo.toml -- lex examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- check examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- parse sk rust/examples/basic_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- parse en rust/examples/basic_en.nbd
cargo run --manifest-path rust/Cargo.toml -- parse ai rust/examples/basic_ai.nbd
cargo run --manifest-path rust/Cargo.toml -- parse sk examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- parse ai examples/hello_ai.nbd
```

**Build status:** Source committed, but local compilation **not verified** in the authoring environment (no `cargo`/`rustc`). The GitHub workflow must confirm compatibility before this is a completed migration stage.

## Supported commands

| Command | Meaning |
|---|---|
| `lex FILE` | Tokenize UTF-8 source; print tokens with original spellings and positions |
| `check FILE` | **Lexer only**, not grammar/types/runtime |
| `parse sk FILE` | Parse a small Slovak syntax subset into a Rust AST |
| `parse en FILE` | Parse a small English syntax subset into a Rust AST |
| `parse ai FILE` | Parse a small canonical machine-oriented syntax subset into a Rust AST |

## Parsed grammar subset

- Original-script lowercase identifiers, numbers, quoted strings and boolean literals;
- Declarations in `sk`, `en` and `ai` (`nech`, `let`, `ai_let`) and primitive type annotations;
- Assignments, arithmetic/comparisons, unary +/- and calls;
- **New in 0.24:** `if/elif/else`, `while`, `for ... in`, function definitions, `return`, `break`, `continue`, nested brace-delimited blocks;
- Typed Rust syntax tree nodes `Expr`, `Statement`, `Program`, with no Python dependency.

**Still unsupported:** full 39-human-language dictionary port (Rust currently parses only SK/EN/AI), static typing, unary logical `not`, full operators, dictionaries, modules/imports, exception handling, compile-time validation of break/return placement, execution, native compilation and complete legacy parity.

`check` remains **LEXER-ONLY**. Use `parse` for syntax validation.
`parse` produces an AST; it does **not** run the program.



## 0.25 — AST contextual validation (source milestone)

The Rust crate now includes `src/validate.rs` and a distinct `validate` CLI command. This **checks legal nesting of control-flow instructions** after parsing; it does not run code or verify types.

```bash
cargo run --manifest-path rust/Cargo.toml -- validate sk examples/hello_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai examples/hello_ai.nbd
cargo test --manifest-path rust/Cargo.toml --all-targets
```

- `N3008` means a `return` was written outside any function.
- `N3009` means `break` or `continue` was written outside a loop in the **current** function. A function declared inside an outer loop cannot break that outer loop.
- Tests cover nested functions, loops, branch bodies and CLI errors, including a check that unavailable AST source spans are not fabricated.
- `parse` remains syntax-only; `validate` adds the control-flow context pass; `check` remains lexer-only. None of these commands execute Nobody code.

**Verification:** Rust compiler availability and GitHub Actions test results are still unconfirmed; the source and tests are committed, not claimed green.
