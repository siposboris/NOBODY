# Nobody — universal configurable multilingual programming language

**Status:** experimental, independent open-source project. Not production-ready.

Nobody aims to enable programming in original human writing systems, with optional language/runtime features, while preserving a common semantic core. It is **not** an official OpenAI product.

## Progress — 2026-11-08
- **249 language/script variants** recorded in a [reference catalogue](nobody/data/google_translate_249.txt), following a publicly reported August 2026 Google Translate list. This is a *planning catalogue*, not 249 implemented syntax translations.
- **28 experimental syntax dictionaries** with source-to-source translation (Nobody → Nobody), including Hebrew, Greek, Ukrainian, Persian, Swahili and Dutch added in milestone 0.13. **221 catalogue entries remain planned**.
- One language profile per file; original writing systems; no mixed-language keyword syntax. Lowercase writing where that distinction exists. Latin-script profiles generally use ASCII-friendly forms, but Vietnamese retains accents.
- Arabic, Urdu, Hebrew and Persian have RTL direction metadata, with logical Unicode source order. No full direction-aware editor yet.
- Basic [lexer](nobody/lexer.py), [AST parser](nobody/parser.py), [AST evaluator](nobody/ast_vm.py), [source translator](nobody/translator.py) and [CLI](nobody/__main__.py).
- The new AST evaluator runs supported declarations, arithmetic, basic types, lists, function calls, branching and loops directly, **without generating Python source code**. The interpreter implementation is still in Python, **not** a native self-hosted compiler.
- Local full prototype archive 0.13 passed **93 tests**; that number is **not** a claim about GitHub Actions. The complete older bootstrap source and its full tests are not yet imported here.

## Try the GitHub frontend

Requires Python 3.11+ (temporary bootstrap):

```bash
python3 -m nobody info
python3 -m nobody languages
python3 -m nobody languages --json
python3 -m nobody ast-run examples/hello_sk.nbd --lang sk
python3 -m nobody translate examples/hello_sk.nbd --from-lang sk --to he
python3 -m nobody direction fa
python3 -m unittest discover -s tests -p 'test_013_catalog_vm.py' -v
```

**Never run untrusted Nobody code.** This experimental interpreter is not a security sandbox; its step budget does not enforce a hard RAM limit.

## Project documents

[Roadmap](docs/ROADMAP.md) · [Languages](docs/LANGUAGES.md) · [Feature status](docs/FEATURES.md) · [Changelog](CHANGELOG.md) · [Release process](docs/RELEASING.md) · [RTL](docs/RTL.md) · [MIT license](LICENSE) · [AI disclosure](AI_DISCLOSURE.md) · [Contribution guide](CONTRIBUTING.md)

## Attribution

The independent Nobody project was initiated from its creator's requirements with substantial programming and documentation assistance from **ChatGPT by OpenAI**. OpenAI does not endorse, sponsor, own or guarantee this project.
