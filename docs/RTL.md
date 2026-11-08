# Nobody 0.12 — Arabic/Urdu RTL direction and source safety

**Implemented:** Native Arabic (`ar`) and Urdu (`ur`) syntax profiles are tagged RTL using `nobody.bidi.direction_for_language()`. The lexer rejects Unicode bidirectional format/override controls which can visually disguise source text (including controls inside strings and comments). Ordinary native-script characters remain supported. Source files and token offsets retain **logical Unicode order**.

**Not implemented:** Nobody does NOT yet set the direction of an external editor, terminal, GitHub code view or browser. There is no Nobody IDE with guaranteed RTL cursor navigation, RTL syntax highlighting or selection behavior. Native glyph rendering may vary by editor, font and platform.

## Why logical order matters

Programming operators, braces, file offsets and expression parsing work in the stored character order. **Never reverse the source string** to make the code appear RTL. Instead, choose an editor capable of right-to-left paragraph layout, while keeping stored source characters unchanged.

Example Arabic syntax:
```text
دع pocet = 3
إذا pocet > 1 {
    اطبع("مرحبا 123")
}
```

## Security rule

Invisible bidi-control characters such as U+202E (RIGHT-TO-LEFT OVERRIDE) and U+2066 (LEFT-TO-RIGHT ISOLATE) are rejected anywhere in source, even inside comments and strings. These are **not necessary** for Arabic or Urdu keywords. Source is UTF-8.

## Sources and tests

- [Direction metadata and validation](../nobody/bidi.py)
- [Lexer integration](../nobody/lexer.py)
- [Regression tests](../tests/test_bidi.py)
- [Language profiles](LANGUAGES.md)

## Next roadmap milestones

- Add direction-aware diagnostics for Arabic/Urdu with precise character offsets.
- Specify Unicode normalization, confusable characters, grapheme segmentation, and mixed-direction punctuation display.
- Build/test an editor extension that can actually set RTL line direction and cursor navigation.
- Evaluate Arabic/Urdu keyword wording with native speakers.
- Finish importing full interpreter/bootstrap code to GitHub, and validate all tests there.

A direction setting is **metadata**, not proof that the program/editor has complete RTL display support.


## Additional RTL profiles — 0.13

Hebrew (`he`) and Persian (`fa`) have been added to the RTL metadata registry alongside Arabic (`ar`) and Urdu (`ur`). No source characters are reversed. The editor still controls visual direction, selection, punctuation layout and cursor movement; the language runtime does not enforce those visual behaviors. The lexical security policy rejects hidden Unicode bidi-formatting controls, including in comments.
