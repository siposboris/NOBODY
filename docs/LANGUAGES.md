# Nobody — 39 experimental human syntax profiles + canonical AI

**Current Rust-first source snapshot (0.26):** 39 human language keyword
tables + one new `ai` profile. The original 249 human language/script entries
remain a **planning catalogue**; 210 reference variants are not yet implemented
as Nobody syntax profiles. [Rust tables](../rust/src/profiles.rs) include the
original writing systems and preliminary native-script parser tests.

**Important:** Original native-speaker terminology and Unicode edge cases are
not certified. The Rust build and full 40-profile runtime parity have **not yet
been verified**. The `ai` vocabulary now uses `let/fn/if/return/true`, unlike
the old Python-reference `ai_` dialect.
[AI syntax documentation](AI_SYNTAX.md).

## Historical first 22 language profiles

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


## Expanded scope: 249 entries, 28 experimental syntax dictionaries (2026-11-08)

The 22 rows above remain the **original working experimental profiles**, not a full listing of all language variants in Google Translate. Six experimental profiles have been added:

| Code | Language | Native script / case | Conditional keyword |
|---|---|---|---|
| `he` | Hebrew | Hebrew, RTL | `אם` |
| `el` | Greek | Greek lowercase | `αν` |
| `uk` | Ukrainian | Cyrillic lowercase | `якщо` |
| `fa` | Persian | Persian script, RTL | `اگر` |
| `sw` | Swahili | Latin lowercase | `ikiwa` |
| `nl` | Dutch | Latin lowercase | `als` |

[Full catalogue of 249 variant names](../nobody/data/google_translate_249.txt). The catalogue includes dialects and script variants, so it must **not** be described as 249 fully supported syntax languages. All other 221 entries are planned, not functional. This reference is based on a publicly reported August 2026 Google Translate list; project status is not affiliated with Google.

The source-code profiles are provisional and still require native-speaker review and collision testing. Each file uses one syntax language. Text and comments are not translated.


## Experimental Nobody 0.14 — six more syntax profiles

| Code | Language | Writing system | `if` keyword |
|---|---|---|---|
| `bg` | Bulharčina | cyrilika | `ако` |
| `ro` | Rumunčina | latinka | `daca` |
| `sv` | Švédčina | latinka | `om` |
| `fi` | Fínčina | latinka | `jos` |
| `hy` | Arménčina | arménske písmo | `եթե` |
| `ka` | Gruzínčina | gruzínske písmo | `თუ` |

**Current total: 34 experimental syntax profiles and 215 remaining catalogue variants.** All six use original writing systems where appropriate; no compulsory romanization for Bulgarian, Armenian or Georgian. Each syntax profile still needs expert linguistic review. The previous 22-row table above documents the original first batch; the six 0.13 profiles are listed in the preceding section. [All 249 catalogue variants](../nobody/data/google_translate_249.txt).


## New in 0.15 — five human languages and the AI profile

| Code | Language/profile | Original writing system | `if` keyword |
|---|---|---|---|
| `ne` | Nepali | Devanagari | `यदि` |
| `mr` | Marathi | Devanagari | `जर` |
| `gu` | Gujarati | Gujarati | `જો` |
| `kn` | Kannada | Kannada | `ಯದಿ` |
| `te` | Telugu | Telugu | `ఒకవేళ` |
| `ai` | Machine-friendly canonical Nobody syntax | Lowercase ASCII | `ai_if` |

**Counts:** 39 experimental human syntaxes + 1 machine-friendly `ai` syntax profile = 40. The catalogue remains 249 human language/script variants, of which 210 are pending. `ai` is not a human language or a Google Translate catalogue entry. These human translations are provisional, not native-speaker certified. [AI profile guide](AI_SYNTAX.md).

## Current Rust profile codes

`ai ar bg bn cs de el en es fa fi fr gu he hi hy id it ja ka kn ko mr ne nl pl pt ro ru sk sv sw ta te th tr uk ur vi zh`

The 39 human codes and `ai` are implemented as Rust keyword tables and can be listed with `cargo run --manifest-path rust/Cargo.toml -- list-languages`. A complete checked runtime for all languages is **not yet implemented**.
