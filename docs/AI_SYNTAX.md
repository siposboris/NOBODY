# Nobody `ai` syntax — familiar, regular and machine-readable

**Status:** Rust-first experimental syntax proposal (November 8, 2026).  
**Design goal:** Reduce *unfamiliar syntax* for code-generating AI models while keeping one well-defined Nobody semantic core. Familiarity is a design hypothesis, **not a proven improvement in AI accuracy, token cost or performance**. No model benchmarks have yet been run.

## Design principles

1. Use short, recognizable programming keywords already common in modern programming languages. No artificial `ai_` prefix.
2. Keep source readable to a human and deterministic to a parser.
3. Use braces for blocks, explicit `let` for declarations and `fn` for functions.
4. Keep lowercase `true`, `false`, `null`, `and`, `or`, `not`.
5. Share the same AST and behavior with the human-language syntax profiles; `ai` is a **syntax dialect**, not a different AI-only runtime, intelligence or autonomous agent.
6. Keep JSON AST exchange as a separate tool protocol. A stable AST is better suited for machine interchange than invented unreadable symbols.

## Canonical example

```text
fn double(x) {
    return x * 2
}

let count: int = 3
if count > 1 {
    print(double(count))
} else {
    print(0)
}
```

## Keyword conventions

| Meaning | Old Python-reference `ai` | New Rust-first `ai` |
|---|---|---|
| Declaration | `ai_let` | `let` |
| Function | `ai_def` | `fn` |
| Condition | `ai_if` | `if` |
| Alternative | `ai_else` | `else` |
| Loop | `ai_while` / `ai_for` | `while` / `for` |
| Return | `ai_return` | `return` |
| Boolean | `ai_true` / `ai_false` | `true` / `false` |
| No value | `ai_none` | `null` |
| Print | `ai_print` | `print` |
| Primitive type | `ai_int` | `int` |
| Else-if | `ai_elif` | `elif` (also `else if` in Rust parser) |

The new Rust vocabulary implements all 29 canonical keyword slots. Native human-language profiles are in [`rust/src/profiles.rs`](../rust/src/profiles.rs). Their translations remain experimental and require native-speaker review.

## Safely migrate existing `ai_` files

The original prefixed dialect remains preserved in the Python reference and [legacy example](../examples/hello_ai_legacy.nbd). A Rust tokenizer-based migration tool rewrites **only recognized keyword tokens**, not quoted strings or comments.

```bash
cargo run --manifest-path rust/Cargo.toml -- migrate-ai examples/hello_ai_legacy.nbd > migrated_ai.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai migrated_ai.nbd
```

The converter writes to **stdout**; it never overwrites its input file. The Rust parser raises **N3010** for recognized old `ai_` keywords to explain the required migration. Different runtime behavior between old Python and new Rust code is still possible: full semantic parity has **not** been verified.

## Next acceptance checks

- Compile and run `cargo test` and CI for Rust, then confirm all 40 profiles parse their native-script keywords.
- Validate Unicode normalization, identifier collisions and language-specific reserved words.
- Benchmark the same coding prompts across `ai`, English `en` and representative ordinary languages using independent LLMs, comparing success, tokens and repairs.
- Port full AST JSON export/import and source-to-source translation into Rust.
- Remove the old Python reference only after equivalent Rust frontend/runtime behavior and regression tests have been verified.

[Full roadmap](ROADMAP.md) · [Rust migration](RUST_MIGRATION.md) · [Error catalogue](ERROR_CODES.md).
