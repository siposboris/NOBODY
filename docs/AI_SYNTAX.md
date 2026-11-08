# Nobody AI syntax profile (`ai`) — experimental 0.15

The `ai` profile is **not a language spoken by AI**. It is a deterministic, explicit source vocabulary intended to be easy for LLMs, coding agents, parsers and code generators to manipulate. It has the same semantics as every other Nobody syntax profile and confers **no extra permissions or autonomy**.

## Example

Slovak:
```text
nech pocet = 3
ak pocet > 2 { vypis("Ahoj") }
```

AI-profile Nobody:
```text
ai_let pocet = 3
ai_if pocet > 2 { ai_print("Ahoj") }
```

## Design rules

- Every keyword begins with `ai_` and uses **lowercase ASCII**, for example `ai_let`, `ai_if`, `ai_else`, `ai_while`, `ai_for`, `ai_function`/canonical `ai_def`, `ai_return`, `ai_true`, `ai_false`, `ai_none`, `ai_and`, `ai_or`, `ai_not`, `ai_print`, `ai_int`.
- The current canonical spellings are built mechanically from internal semantic IDs, not human-language translation. For function declaration the actual spelling is **`ai_def`**.
- Never mix `ai` keyword spellings with `sk`, `en`, or another profile inside a source file.
- String literals, comments and user identifiers are not translated.
- The output remains Nobody source, not Python.

## Machine-readable AST

The `ast-json` CLI command exports a **versioned and deterministic** JSON representation (`nobody.ast.v1`) containing `kind`, `value`, `line` and `children`. This is an AST interchange preview, **not** a full program specification and not bytecode or machine instructions. It is most useful for external AI coding workflows.

From the GitHub frontend:
```bash
python3 -m nobody translate examples/hello_sk.nbd --from-lang sk --to ai
python3 -m nobody ast-json examples/hello_sk.nbd --lang sk
```

Source: [language profiles](../nobody/languages.py), [AST serializer](../nobody/ai_ir.py), [regression tests](../tests/test_015.py).

## Limitations / future milestones

- Only the existing AST grammar subset is supported.
- User identifier naming/keyword collisions across 40 profiles need improved namespace handling.
- No AI-specific model weights, cognition, learning, privileges or autonomy are implemented.
- Future: formal grammar, schema for structured metadata, deterministic round-trip AST loader with validation, native backends and self-hosting.
