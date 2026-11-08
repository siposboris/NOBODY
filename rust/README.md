# Nobody Rust frontend (experimental)

This crate is the **first step** in the Python → Rust → Nobody migration.
It does **not** yet compile native programs, execute Nobody, infer types or replace the original parser.

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

- Lowercase native-script identifiers, numerals, string literals and boolean literals;
- `nech x = ...`, `let x = ...`, `ai_let x = ...`;
- Optional primitive annotations: `nech x: cele = 3`, `let x: int = 3`, `ai_let x: ai_int = 3`;
- Variable reassignment, basic arithmetic/comparison, parenthesized expressions and function-call **syntax** (not execution).
- `Expr`, `Statement` and `Program` are Rust typed enums/structs, with no Python dependency.

**Important:** The older `examples/hello_sk.nbd` and `examples/hello_ai.nbd` contain function or condition declarations not supported by this new small parser; use the `rust/examples/basic_*.nbd` fixtures above for the `parse` command.\n\n**Still unsupported:** `if`/`else`, loops, function *declarations*, returns, collections, imports, Python-legacy feature parity, typed analysis, source translation, code generation and execution.

## Error numbers

- Existing lexer errors N2001–N2005 are retained.
- Parser errors N3001 (missing required token), N3003 (invalid identifier/type name), N3005 (invalid expression), N3006 (unsupported statement) and N3007 (unsupported Rust syntax profile).
- N3006/N3007 have documented causes and repairs in [ERROR_CODES.md](../docs/ERROR_CODES.md). No automatic reinterpretation of the requested language is performed.

## Current architecture

`src/lib.rs`: lexer and diagnostics.  
`src/syntax.rs`: tiny recursive-descent/precedence parser and typed AST.  
`src/main.rs`: native Rust CLI.

Next: establish green Rust CI, port all human language tables with canonical semantic tokens, improve normalization and spans, and add a larger parser grammar. Only remove a Python component after functional parity and verified differential tests.

[Full roadmap](../docs/ROADMAP.md) · [Migration acceptance gates](../docs/RUST_MIGRATION.md).
