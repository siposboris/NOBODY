# Translation safety in Nobody Rust

**Milestone:** 0.27 development source. Rust compilation and CI are still awaiting independent verification.

## What the translator does

`nobody-rs translate FROM TO FILE` performs an experimental Nobody-to-Nobody **token-aware keyword substitution** for the 39 human profiles and the redesigned `ai` profile.

It does not emit Python and does not modify comments or quoted literals. The current translator preserves whitespace, punctuation and non-keyword identifiers, and uses canonical semantic keyword positions.

```bash
cargo run --manifest-path rust/Cargo.toml -- translate sk ai rust/examples/basic_sk.nbd
```

Slovak source:

```text
nech cislo: cele = 2
vypis(cislo)
```

AI syntax output:

```text
let cislo: int = 2
print(cislo)
```

## N3011: deterministic refusal rather than wrong program

A variable called `fn` is legal in the Slovak source profile, but `fn` is a function-declaration keyword in the redesigned `ai` profile.

```text
nech fn = 3
vypis(fn)
```

Earlier versions could output invalid `let fn = 3`. Now the Rust translator detects that the *source identifier* becomes a reserved keyword in the target profile and reports:

```text
N3011 line 1, column 6: Identifier "fn" is a reserved keyword
in target profile "ai"; the generated program would change meaning.
Repair: Rename the source variable/function/parameter to a name that
is not reserved in the target profile, then translate again.
```

The correct fix is to rename the **source** variable, such as `fn` → `hodnota`, and retry. Source strings (`"fn"`) and comments (`# fn`) never trigger this error merely by containing the name.

The N3011 error is append-only in [ERROR_CODES.md](ERROR_CODES.md); it is not a generic catch-all. The Rust unit and integration tests cover the error number, source location and 40-profile keyword translation.

## Limits: not yet a proof of semantics

- The guard is deliberately conservative: any source non-keyword name matching a reserved target keyword is rejected, even when a later semantic pass might prove it harmless.
- No scope-aware automatic identifier renaming is currently performed.
- No static type or runtime semantic-equivalence checking is performed by `translate`.
- Unicode normalization, native-speaker profile review, reserved built-in shadowing and dynamically generated code still need comprehensive tests.
- The Cargo build and GitHub Actions success have **not** been independently verified in the authoring environment.

**Future goal:** parse → canonical typed AST/IR → regenerate target Nobody source, with identifier hygiene, symbol tables and round-trip tests across all profiles.

[English roadmap](ROADMAP.md) · [Rust migration contract](RUST_MIGRATION.md) · [Numbered error catalogue](ERROR_CODES.md).
