# Nobody — 22 syntax languages and native writing systems

**Experimental support, 2026-11-08.** This is a syntax-profile catalog, not a claim of native-speaker certified translations. The 22 entries are a practical multilingual selection, not a verified UN or Ethnologue ranking of the 22 largest languages.

| Kód | Jazyk | Písmo | Podmienka `if` |
|---|---|---|---|
| `sk` | Slovenčina | latinka bez diakritiky | `ak` |
| `en` | Angličtina | latinka | `if` |
| `cs` | Čeština | latinka bez diakritiky | `pokud` |
| `de` | Nemčina | latinka bez diakritiky | `wenn` |
| `es` | Španielčina | latinka bez diakritiky | `si` |
| `fr` | Francúzština | latinka bez diakritiky | `si` |
| `pt` | Portugalčina | latinka bez diakritiky | `se` |
| `ru` | Ruština | cyrilika | `если` |
| `zh` | Mandarínska čínština | čínske znaky | `如果` |
| `hi` | Hindčina | dévanágarí | `अगर` |
| `ar` | Arabčina | arabské písmo | `إذا` |
| `bn` | Bengálčina | bengálske písmo | `যদি` |
| `id` | Indonézština | latinka | `jika` |
| `ur` | Urdčina | arabské písmo (nastalík) | `اگر` |
| `ja` | Japončina | japonské písmo | `もし` |
| `ko` | Kórejčina | hangul | `만약` |
| `tr` | Turečtina | latinka bez diakritiky | `eger` |
| `vi` | Vietnamčina | latinka s vietnamskou diakritikou | `nếu` |
| `ta` | Tamilčina | tamilské písmo | `எனில்` |
| `it` | Taliančina | latinka | `se` |
| `th` | Thajčina | thajské písmo | `ถ้า` |
| `pl` | Poľština | latinka bez diakritiky | `jezeli` |

## Source-file policy

- Select **exactly one** syntax language for a source file; don't mix foreign-language keywords in that file.
- Original scripts are mandatory for Chinese, Japanese, Korean, Hindi, Bengali, Arabic, Urdu, Tamil, Thai and Russian keywords. **No romanization** in those native-script syntax profiles.
- Latin-script profiles use lowercase spelling. Most use the project's ASCII-friendly spellings without diacritics; Vietnamese deliberately keeps its native accented lowercase orthography (`nếu`, `hàm`).
- Uppercase spellings of keywords are rejected where case exists.
- User strings and comments are never translated and may contain arbitrary Unicode.
- Translation changes recognized syntax keywords and selected built-in names only; variable names are not translated.
- Non-Latin glyph segmentation, source normalization, naming collisions, native lexical ambiguity and all 22 grammatical spellings require further verification. This is **not** natural-language translation of arbitrary user programs.

## Examples

Slovak:
```text
nech pocet = 3
ak pocet > 1 {
    vypis("Ahoj!")
}
```

Chinese:
```text
设 pocet = 3
如果 pocet > 1 {
    打印("Ahoj!")
}
```

Japanese:
```text
置く pocet = 3
もし pocet > 1 {
    表示("Ahoj!")
}
```

Arabic:
```text
دع pocet = 3
إذا pocet > 1 {
    اطبع("Ahoj!")
}
```

## Implementation

- [Keyword profiles](../nobody/languages.py)
- [Unicode tokenization](../nobody/lexer.py)
- [Source-to-source translator](../nobody/translator.py)
- [Native-script regression tests](../tests/test_native_scripts.py)

The translator is a **Nobody → Nobody** source transformer. A complete native compiler or Python-free runtime is **not implemented**. The separate interpreter/compiler prototype remains in transition to GitHub.

## Review checklist

For each of the 22 profiles, native speakers should inspect the spelling and naturalness of all reserved words, identify ambiguous multiword forms, confirm script/case conventions and check round-trip transformations and context-sensitive identifier collisions before any stable language specification is declared.

## RTL display and source ordering

Arabic (`ar`) and Urdu (`ur`) now carry explicit RTL direction metadata in [bidi.py](../nobody/bidi.py). Their original script is retained; source is never byte-reversed or character-reversed. Hidden bidi override/isolation controls are rejected in lexer input. **Editor rendering is not automated yet.** See [RTL guide](RTL.md).
