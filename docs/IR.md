# Nobody canonical IR v1 — experimental Rust representation

**Introduced:** Rust development snapshot 0.28.  
**Status:** Source and tests committed. The authoring environment has no Cargo or rustc, and a passing Rust build/GitHub CI run has not been confirmed.

## Purpose

The Nobody parser supports human-language source keywords and a familiar `ai` syntax. A compiler should not need a separate interpreter or optimization pass for each of these profiles.

The new Rust module [`rust/src/ir.rs`](../rust/src/ir.rs) lowers a validated source AST into **language-neutral, typed Rust structures**:

- `Module { version: "nobody.ir.v1", statements: [...] }`
- `IrStatement`: declarations, assignments, conditions, loops, function definitions, returns and expression statements.
- `IrExpr`: literals, user names, known built-ins, calls and unary/binary operators.
- `Name::Builtin("print")` for the same built-in across all supported syntax profiles; `Name::User(...)` for ordinary identifiers.

All 39 human syntax tables and the redesigned `ai` syntax use the same IR data model.

## CLI example

Slovak:

```text
nech vek: cele = 30
vek = vek + 1
vypis(vek)
```

Equivalent English source (retaining the same user-defined name):

```text
let vek: int = 30
vek = vek + 1
print(vek)
```

```bash
cargo run --manifest-path rust/Cargo.toml -- ir sk rust/examples/basic_sk.nbd
cargo run --manifest-path rust/Cargo.toml -- compare-ir sk rust/examples/basic_sk.nbd en rust/examples/ir_match_en.nbd
```

`compare-ir` reports `MATCH` only when the resulting **canonical IR trees are structurally equal**. It reports `DIFFERENT` and exits with status 1 otherwise. Parse/validation failures retain the existing specific numbered errors, such as N3007 (unknown language) or N3008 (invalid return context).

The `ir` command currently renders a Rust debug tree; it is **not** a serialized JSON AST schema, executable bytecode or native code.

## Shadowing behavior

Direct calls to known built-ins such as `vypis(...)`, `print(...)` and their equivalents are normalized to the same `Name::Builtin("print")` if there is no local or enclosing declaration with that name.

If a function parameter or local declaration has the same spelling as a built-in, the IR uses `Name::User` instead of pretending that the name refers to the built-in. This makes equivalence checking conservative.

**Current limitation:** scope collection is intentionally conservative and treats a declaration as shadowing its name throughout the lexical block. Temporal declaration order, closures, dynamic binding and complete function semantics are not yet formally specified. User-defined identifier spellings are preserved, not automatically renamed or alpha-normalized.

## What structural equality does NOT establish

- Identical runtime behavior, identical outputs or function purity;
- Full lexical-scope and control-flow semantics;
- Type correctness, ownership or memory safety;
- Full support for every legacy Python frontend language feature;
- Proof that different spellings of user-defined identifiers represent equivalent programs.

Differences in identifier names or literal spelling may produce `DIFFERENT` even for behaviorally equivalent programs. This conservative result is intentional.

## Regression coverage

The new source contains eight authored Rust unit tests for:

- Slovak vs. AI normalizations;
- Different operators yielding different IR;
- Equivalent built-in calls;
- Shadowed built-in parameters retaining user identity;
- Nested loop-control AST lowering;
- Existing N3008 and N3007 errors;
- Translating and lowering keyword programs across all 40 syntax profiles.

The CLI has four additional authored integration tests. The GitHub Actions Rust workflow includes IR commands. **These tests have not been independently verified as passing.**

## Next roadmap dependencies

1. Obtain confirmed Cargo compilation/CI results and fix any compiler/test failures.
2. Add stable source spans to IR and standardized diagnostic positions.
3. Introduce scopes/symbol IDs and alpha-safe identifier handling instead of string-only identity.
4. Port static primitive type checking from the Python reference into a Rust IR pass.
5. Add a deterministic runtime or bytecode representation, then compile/run parity checks.
6. Delete a Python component only when its Rust replacement passes verified behavioral-parity tests.

[Roadmap](ROADMAP.md) · [Rust migration contract](RUST_MIGRATION.md) · [Error catalogue](ERROR_CODES.md).
