# Nobody — universal configurable multilingual programming language

**Status:** experimental, independent open-source project. Not production-ready.

Nobody aims to enable programming in original human writing systems, with optional language/runtime features, while preserving a common semantic core. It is **not** an official OpenAI product.

## Progress — 2026-11-08
- **249 language/script variants** recorded in a [reference catalogue](nobody/data/google_translate_249.txt), following a publicly reported August 2026 Google Translate list. This is a *planning catalogue*, not 249 implemented syntax translations.
- **39 experimental human syntax dictionaries** and **one machine-friendly `ai` profile** with source-to-source translation (Nobody → Nobody). **210 human catalogue entries remain planned**. Five native-script profiles and `ai` were added in 0.15.
- One language profile per file; original writing systems; no mixed-language keyword syntax. Lowercase writing where that distinction exists. Latin-script profiles generally use ASCII-friendly forms, but Vietnamese retains accents.
- Arabic, Urdu, Hebrew and Persian have RTL direction metadata, with logical Unicode source order. No full direction-aware editor yet.
- Basic [lexer](nobody/lexer.py), [AST parser](nobody/parser.py), [AST evaluator](nobody/ast_vm.py), [source translator](nobody/translator.py) and [CLI](nobody/__main__.py).
- The new AST evaluator runs supported declarations, arithmetic, basic types, lists, function calls, branching and loops directly, **without generating Python source code**. The interpreter implementation is still in Python, **not** a native self-hosted compiler.
- Local full prototype archive 0.15 passed **110 tests**; that number is **not** a claim about GitHub Actions. The complete older bootstrap source and its full tests are not yet imported here.

## Try the GitHub frontend

Requires Python 3.11+ (temporary bootstrap):

```bash
python3 -m nobody audit-languages
python3 -m nobody info
python3 -m nobody languages
python3 -m nobody languages --json
python3 -m nobody ast-run examples/hello_sk.nbd --lang sk
python3 -m nobody translate examples/hello_sk.nbd --from-lang sk --to he
python3 -m nobody direction fa
python3 -m nobody ast-run examples/hello_ai.nbd --lang ai
python3 -m nobody ast-json examples/hello_ai.nbd --lang ai
python3 -m unittest discover -s tests -p 'test_013_catalog_vm.py' -v
```

**Never run untrusted Nobody code.** This experimental interpreter is not a security sandbox; its step budget does not enforce a hard RAM limit.

## Project documents

[Roadmap](docs/ROADMAP.md) · [Languages](docs/LANGUAGES.md) · [Feature status](docs/FEATURES.md) · [Changelog](CHANGELOG.md) · [Release process](docs/RELEASING.md) · [RTL](docs/RTL.md) · [MIT license](LICENSE) · [AI disclosure](AI_DISCLOSURE.md) · [Contribution guide](CONTRIBUTING.md)

## Attribution

The independent Nobody project was initiated from its creator's requirements with substantial programming and documentation assistance from **ChatGPT by OpenAI**. OpenAI does not endorse, sponsor, own or guarantee this project.


## 0.14 milestone — six new experimental syntax languages

The language catalogue still contains **249** Google Translate language/script variants. **34** now have experimental Nobody syntax dictionaries; **215** are still planned and unavailable. 0.14 adds Bulgarian (`bg`), Romanian (`ro`), Swedish (`sv`), Finnish (`fi`), Armenian (`hy`), and Georgian (`ka`). All use native writing systems for their keywords where appropriate; terminology needs human review. See [language roster](docs/LANGUAGES.md).

The AST interpreter now supports assignment to existing variables (`count = count + 1`) with rejection of assignments to undeclared variables. Typed reassignments are not yet checked against initial annotations. The full local 0.14 ZIP passed **104 tests**, but GitHub CI remains unconfirmed and the complete Nobody-written bootstrap is not yet uploaded to this repository. [Roadmap](docs/ROADMAP.md).


## 0.15 milestone — AI syntax profile, five more native scripts, typed reassignment

The project now contains **39 experimental human-language syntax profiles plus one special `ai` profile**. The Google Translate reference catalogue retains **249 variants**, with **210 human variants still awaiting a syntax dictionary**. New human profiles: Nepali (`ne`), Marathi (`mr`), Gujarati (`gu`), Kannada (`kn`), Telugu (`te`). Native-speaker validation remains outstanding.

The AI profile is **not an artificial intelligence or a language with special powers**: it is a regular Nobody syntax profile with semantically transparent lowercase ASCII tokens such as `ai_let`, `ai_if`, `ai_print`. [AI profile design](docs/AI_SYNTAX.md).

The frontend now exports a stable versioned JSON AST via `python3 -m nobody ast-json examples/hello_sk.nbd --lang sk`. The experimental AST interpreter rejects reassignments that violate a variable's explicitly declared primitive type. It still requires Python and is **not a security sandbox**.

**110 local tests passed** in the full 0.15 development archive. GitHub CI success is not yet independently verified; the full earlier Nobody-written bootstrap compiler still needs importing to this repo.


## 0.16 AST interoperability milestone

The standalone GitHub frontend now includes an [AST JSON validator and Nobody source re-emitter](nobody/ast_format.py). AI-oriented workflows can emit JSON with `ast-json`, validate a `nobody.ast.v1` document with `ast-validate`, and generate source in a selected Nobody language with `ast-emit --lang ai`. The source emitter is limited to the current parser subset.

```bash
python3 -m nobody ast-json examples/hello_ai.nbd --lang ai > hello.json
python3 -m nobody ast-validate hello.json
python3 -m nobody ast-emit hello.json --lang sk
```

**116 tests passed locally** in the full 0.16 development tree. Do not interpret that as confirmed GitHub CI execution. The AST loader only validates a structure; it is not an execution sandbox. 39 experimental human syntax profiles and one `ai` profile remain unchanged. Native compiler/self-hosting is still planned.
