# Nobody lexical scanner (experimental)

Source: [nobody/lexer.py](../nobody/lexer.py). Regression tests: [tests/test_lexer.py](../tests/test_lexer.py).

The current scanner returns tokens with kind, original spelling, line, column and character offsets. It recognizes lowercase ASCII and native-script Unicode names (including combining marks), number-like sequences, strings, comments and symbols. It preserves Unicode inside strings and comments. Uppercase names are rejected where case exists; individual source-language policies are checked by the translator, not by the lexer alone.

The lexer is a first step, **not a complete grammar/parser**. Numbers and operators are currently tokenized preliminarily. Python bootstrap remains in the full local prototype, and the GitHub repository does not yet include its entire implementation.

### Local stand-alone lexer test

Run from repository root:

```bash
python3 -m unittest discover -s tests -p 'test_lexer.py' -v
```

The lexer cannot by itself execute Nobody programs. Full compiler and runtime integration must be imported and validated separately.

### Next steps
- Grammar specification and token definitions for operators/numbers.
- Dedicated parser and structured syntax tree.
- Reserved-word registry and source language profile.
- Syntax-aware translation that never modifies string contents and avoids identifier collisions.
- Integration tests for full source and a verified source release.


Unicode support is a lexical capability; whether a word is permitted in a particular configured source language is handled by the translator and future semantic checks. [Full language list](LANGUAGES.md).
