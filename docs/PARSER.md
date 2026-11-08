# Nobody 0.10 — experimental AST parser

The independent [parser](../nobody/parser.py) uses the [lexer](../nobody/lexer.py) to create a tree of `Node(kind, line, value, children)` objects. It does **not** execute a program or emit Python.

## Supported subset
- One explicit human syntax language per file: Slovak (`sk`), English (`en`), Czech (`cs`), German (`de`).
- Lowercase ASCII names; Unicode accepted in comments and strings.
- `let`-style declaration, `if/elif/else`, `while`, `for/in`, functions, returns, break and continue.
- Braces for blocks, arithmetic/comparison/logical operator precedence, function calls, indexing and list literals.
- Source spans from lexer, line-aware AST nodes and clear failures on unsupported grammar.

## Known limitations
- **No typed declarations** in this parser yet, even though earlier experimental interpreter modes support restricted type annotations.
- **No reassignment statements**, indentation-only blocks, full numeric grammar or full error recovery.
- Cross-language keyword collisions may reject identifiers that are otherwise meaningful.
- Current parser is bootstrapped in Python, not implemented in Nobody.
- Only lexer/parser and their tests are present in this GitHub repo; the complete previous experimental interpreter/compiler source is not imported yet.
- A parser is **not** a native compiler, runtime, sandbox or proof of self-hosting.

## Try the parser
From the repository root (Python 3.11+):

```bash
python3 -m unittest discover -s tests -p 'test_parser.py' -v
python3 - <<'PY'
from nobody.parser import parse_source
tree = parse_source('nech cislo = 10\nvypis(cislo)', 'sk')
print(tree)
PY
```

## Tests
The complete local 0.10 tree passed 65 tests. GitHub Actions has a dedicated [frontend workflow](../.github/workflows/frontend-tests.yml), but its execution result has not yet been verified.
