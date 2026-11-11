# Nobody — katalóg číslovaných chýb

**Od Nobody 0.20 (experimentálna verzia).** Každá nová verejná diagnostika musí dostať vlastné **stabilné číslo** vo formáte `Nxxxx`, konkrétny opis skutočnej príčiny, miesto chyby (ak je známe) a krok opravy. Čísla sa nesmú prečíslovať ani opätovne použiť. Symbolické názvy ako `E_TYPE` zostávajú kvôli kompatibilite.

## Rýchly zoznam

| Číslo | Symbol | Presná kategória |
|---|---|---|
| **N1001** | `E_TYPE` | Nesúlad typu |
| **N1002** | `E_BINARY` | Nekompatibilné operandy |
| **N1003** | `E_UNDECLARED` | Nedeklarovaná premenná |
| **N1004** | `E_DUPLICATE` | Duplicitná deklarácia |
| **N1005** | `E_ARITY` | Nesprávny počet argumentov |
| **N1006** | `E_UNARY` | Neplatné číselné znamienko |
| **N1101** | `W_DYNAMIC_NAME` | Neznámy typ mena |
| **N2001** | `LEX_STRING` | Neukončený reťazec |
| **N2002** | `LEX_CASE` | Nepovolené veľké písmená |
| **N2003** | `LEX_COMBINING` | Osamotená diakritická značka |
| **N2004** | `LEX_CHARACTER` | Nepovolený znak |
| **N2005** | `LEX_BIDI` | Skrytý riadiaci znak smeru alebo neviditeľný formátovací znak (U+00AD, U+200B, U+2060, U+FEFF), aj v komentároch a reťazcoch |
| **N2006** | `LEX_NUMBER` | Neplatný desiatkový číselný literál |
| **N2007** | `LEX_MIXED_SCRIPT` | Podozrivé miešanie latinky, gréčtiny a cyriliky v názve |
| **N2008** | `LEX_NFC` | Identifikátor nie je v kanonickej forme Unicode NFC |
| **N3001** | `PARSE_EXPECT` | Chýbajúci prvok |
| **N3002** | `PARSE_BLOCK` | Nesprávne zátvorky bloku |
| **N3003** | `PARSE_IDENTIFIER` | Neplatný názov |
| **N3004** | `PARSE_LANGUAGE` | Cudzie kľúčové slovo |
| **N3005** | `PARSE_EXPRESSION` | Neplatný výraz |
| **N3006** | `PARSE_UNSUPPORTED` | Rust subset parser does not implement this statement |
| **N3007** | `PARSE_PROFILE` | Rust parser does not support the requested syntax profile |
| **N3008** | `RETURN_CONTEXT` | Return outside a function body |
| **N3009** | `LOOP_CONTROL_CONTEXT` | Break or continue outside a loop in the current function |
| **N3010** | `AI_LEGACY_SYNTAX` | Old prefixed `ai_` keyword in the redesigned Rust AI syntax |
| **N3011** | `TRANSLATION_COLLISION` | Source identifier conflicts with a target-language reserved keyword |
| **N4001** | `RUN_NAME` | Neexistujúce meno |
| **N4002** | `RUN_TYPE` | Typová chyba za behu |
| **N4003** | `RUN_LIMIT` | Prekročený limit vykonávania |
| **N4004** | `RUN_CALL` | Neplatné volanie |
| **N5001** | `AST_SCHEMA` | Neplatný AST dokument |
| **N5002** | `FORMAT_DRIFT` | Súbor Nobody sa líši od kanonického formátu; spusti `format-source` a skontroluj výsledok |
| **N5003** | `PROJECT_GRAPH` | Neplatný manifest projektu, nebezpečná cesta, cyklická závislosť, chybný import/export alebo konflikt názvov modulov |
| **N9001** | `CLI_INPUT` | Neplatný vstup alebo konfigurácia |
| **N9999** | `UNCLASSIFIED` | Doteraz nezaradená chyba |

## Podrobné vysvetlenia a opravy

### N1001 — Nesúlad typu (`E_TYPE`)

**Čo presne znamená:** Priraďovaná hodnota má iný známy typ než deklarovaná premenná.

**Ako opraviť:** Oprav hodnotu na deklarovaný typ alebo typovú anotáciu za dvojbodkou.

**Príklad:** `nech vek: cele = "tridsat"` → `nech vek: cele = 30`.

### N1002 — Nekompatibilné operandy (`E_BINARY`)

**Čo presne znamená:** Aritmetický operátor sa používa na typy, pre ktoré daná operácia nie je definovaná.

**Ako opraviť:** Pre odčítanie a násobenie použi číselné hodnoty; na spojenie dvoch textov použi +.

**Príklad:** `nech x = "text" - 3` → použite `nech x = 8 - 3`.

### N1003 — Nedeklarovaná premenná (`E_UNDECLARED`)

**Čo presne znamená:** Priradenie mení názov, ktorý nie je deklarovaný vo viditeľnom rozsahu.

**Ako opraviť:** Najskôr použi nech/ai_let, alebo oprav preklep v názve premennej.

**Príklad:** `pocet = 1` bez deklarácie → `nech pocet = 1`.

### N1004 — Duplicitná deklarácia (`E_DUPLICATE`)

**Čo presne znamená:** Rovnaké meno je deklarované druhýkrát v tom istom lexikálnom rozsahu.

**Ako opraviť:** Premenuj druhú deklaráciu alebo použi jednoduché priradenie bez nech.

**Príklad:** `nech pocet = 1; nech pocet = 2` → druhý príkaz `pocet = 2`.

### N1005 — Nesprávny počet argumentov (`E_ARITY`)

**Čo presne znamená:** Priame volanie známej funkcie nesúhlasí s počtom deklarovaných parametrov.

**Ako opraviť:** Doplň alebo odstráň argumenty podľa parametrov uvedených pri definícii funkcie.

**Príklad:** `funkcia dvoj(x) { vrat x * 2 }; dvoj(1, 2)` → `dvoj(1)`.

### N1006 — Neplatné číselné znamienko (`E_UNARY`)

**Čo presne znamená:** Znamienko + alebo - je použité na známy nečíselný typ.

**Ako opraviť:** Použi celé/desatinné číslo alebo znamienko odstráň.

**Príklad:** `nech x = -"ahoj"` → `nech x = -3`.

### N1101 — Neznámy typ mena (`W_DYNAMIC_NAME`)

**Čo presne znamená:** Statický analyzátor nevie preukázať typ daného identifikátora.

**Ako opraviť:** Skontroluj deklaráciu, typovú anotáciu a miesto, kde je funkcia alebo premenná definovaná.

**Príklad:** `nech n = neznama_funkcia(1)` → deklarujte funkciu alebo použite typovú anotáciu; varovanie samo osebe nie je dôkaz chyby.

### N2001 — Neukončený reťazec (`LEX_STRING`)

**Čo presne znamená:** Za otváracou úvodzovkou chýba ukončovacia úvodzovka.

**Ako opraviť:** Doplň zodpovedajúcu úvodzovku; viacriadkové reťazce uzatváraj tromi.

**Príklad:** `vypis("ahoj)` → `vypis("ahoj")`.

### N2002 — Nepovolené veľké písmená (`LEX_CASE`)

**Čo presne znamená:** Názov obsahuje veľké písmená, hoci syntax vyžaduje lowercase.

**Ako opraviť:** Prepíš konkrétny názov malými písmenami v pôvodnom písme.

**Príklad:** `Nech pocet = 1` → `nech pocet = 1`.

### N2003 — Osamotená diakritická značka (`LEX_COMBINING`)

**Čo presne znamená:** Unicode kombinujúci znak nemá predchádzajúce základné písmeno.

**Ako opraviť:** Vlož základné písmeno pred diakritiku alebo oprav poradie Unicode znakov.

**Príklad:** Samostatné Unicode kombinačné znamienko vložte až za písmeno, ktoré upravuje.

### N2004 — Nepovolený znak (`LEX_CHARACTER`)

**Čo presne znamená:** Lexer nerozpoznal znak mimo komentára a textového literálu. Rust lexer tiež odmieta ASCII riadiace znaky NUL, BEL, ESC a DEL mimo literálov; bežné medzery, tabulátory a konce riadkov sú povolené.

**Ako opraviť:** Odstráň uvedený znak alebo ho vlož medzi úvodzovky.

**Príklad:** Znak `×` mimo textového literálu → násobenie zapíšte ako `*`.

### N2005 — Skrytý riadiaci znak smeru (`LEX_BIDI`)

**Čo presne znamená:** Zdroj obsahuje Unicode znak vizuálne meniaci poradie zdrojového kódu.

**Ako opraviť:** Odstráň konkrétny U+ znak; pre RTL použi nastavenie editora.

**Príklad:** Po skopírovaní arabského kódu odstráňte napríklad neviditeľný U+202E; smer písma nastavte v editore.

### N3001 — Chýbajúci prvok (`PARSE_EXPECT`)

**Čo presne znamená:** Parser očakával konkrétnu zátvorku, operátor alebo iný token.

**Ako opraviť:** Doplň token uvedený v správe na uvedenom mieste.

**Príklad:** `nech pocet 1` → `nech pocet = 1`.

### N3002 — Nesprávne zátvorky bloku (`PARSE_BLOCK`)

**Čo presne znamená:** Blok má prebytočnú alebo chýbajúcu zloženú zátvorku.

**Ako opraviť:** Doplň } alebo odstráň prebytočnú } v označenom bloku.

**Príklad:** `ak pravda { vypis(1)` → doplňte `}`.

### N3003 — Neplatný názov (`PARSE_IDENTIFIER`)

**Čo presne znamená:** Parser očakával identifikátor a našiel neplatný token.

**Ako opraviť:** Po nech/funkcia uveď platný názov, ktorý nie je rezervované slovo.

**Príklad:** `nech = 1` → `nech pocet = 1`.

### N3004 — Cudzie kľúčové slovo (`PARSE_LANGUAGE`)

**Čo presne znamená:** Príkaz patrí inému než nakonfigurovanému jazyku súboru.

**Ako opraviť:** Použi preklad kľúčového slova pre aktuálne --lang.

**Príklad:** Pri `--lang sk` nepoužívajte anglické `if`; použite `ak`.

### N3005 — Neplatný výraz (`PARSE_EXPRESSION`)

**Čo presne znamená:** Výraz má neplatný začiatok alebo chýbajúci operand.

**Ako opraviť:** Skontroluj operand a zátvorky pred miestom chyby.

**Príklad:** `nech x = +` → `nech x = +3`.

### N4001 — Neexistujúce meno (`RUN_NAME`)

**Čo presne znamená:** Počas vykonávania premenná nie je dostupná v aktuálnom rozsahu.

**Ako opraviť:** Deklaruj ju pred použitím alebo ju odovzdaj ako parameter.

**Príklad:** `vypis(pocet)` bez deklarácie → najprv `nech pocet = 1`.

### N4002 — Typová chyba za behu (`RUN_TYPE`)

**Čo presne znamená:** Dynamicky vypočítaná hodnota sa nezhoduje s požadovaným typom.

**Ako opraviť:** Skontroluj typ pri deklarácii aj skutočnú hodnotu výrazu.

**Príklad:** `nech n: cele = 1; n = "ahoj"` → `n = 2`.

### N4003 — Prekročený limit vykonávania (`RUN_LIMIT`)

**Čo presne znamená:** Prekročený počet krokov alebo hĺbka rekurzívnych volaní.

**Ako opraviť:** Skontroluj ukončenie cyklu/rekurzie, až potom zváž zvýšenie limitu.

**Príklad:** `pokial pravda { vypis(1) }` → pridajte podmienku, ktorá sa môže zmeniť na nepravdu.

### N4004 — Neplatné volanie (`RUN_CALL`)

**Čo presne znamená:** Nesedí počet argumentov alebo cieľ nie je dovolená funkcia.

**Ako opraviť:** Porovnaj parametre deklarácie s volaním a dostupnými built-in funkciami.

**Príklad:** `dvoj(1,2)` pri jednom parametri → `dvoj(1)`; nepovolené volanie inej funkcie nahraďte existujúcou.

### N5001 — Neplatný AST dokument (`AST_SCHEMA`)

**Čo presne znamená:** JSON obsahuje neplatnú schému alebo štruktúru uzla.

**Ako opraviť:** Vygeneruj platný nobody.ast.v1 cez ast-json a skontroluj povolené polia.

**Príklad:** AST JSON s `"schema":"nobody.ast.v999"` → použite `"nobody.ast.v1"` a platné uzly.

### N9001 — Neplatný vstup alebo konfigurácia (`CLI_INPUT`)

**Čo presne znamená:** Súbor neexistuje alebo nastavenie nemá platnú hodnotu.

**Ako opraviť:** Skontroluj cestu, UTF-8, jazykový profil a názov nastavenia.

**Príklad:** `python -m nobody type-check neexistuje.nbd --lang sk` → opravte cestu k súboru.

### N9999 — Doteraz nezaradená chyba (`UNCLASSIFIED`)

**Čo presne znamená:** Modul vyvolal chybu, ktorá ešte nemá samostatné číslo v katalógu.

**Ako opraviť:** Zachovaj presnú hlášku a reprodukčný .nbd súbor, aby vznikol nový konkrétny kód.

**Príklad:** Ak sa objaví N9999, do hlásenia priložte presný text a najmenší `.nbd` program, ktorý ho vyvolá.

## Formát hlášky

```text
N1001 [E_TYPE] riadok 1: 'vek' is int, but expression is str. Oprava: Oprav hodnotu na deklarovaný typ alebo typovú anotáciu za dvojbodkou.
```

Presný názov premennej, operátor, typy, počet argumentov a riadok sa preberajú z konkrétnej chyby, nie z všeobecnej šablóny. Systém je zatiaľ experimentálny: staršie knižničné API môžu stále vyhadzovať pôvodné Python výnimky; CLI ich klasifikuje do číselného katalógu. `N9999` dočasne označuje neklasifikované staré chyby a **nie je cieľovým stavom**.

## Použitie

```bash
python3 -m nobody errors          # celý katalóg
python3 -m nobody errors 1001     # význam a oprava chyby N1001
python3 -m nobody type-check program.nbd --lang sk
```

### Pravidlo pre budúci vývoj

Pri každej **novej kategórii** chýb: vybrať ďalšie voľné číslo v príslušnom rozsahu, pridať opis príčiny a opravný krok do `nobody/errors.py`, doplniť túto dokumentáciu a vytvoriť test. Ak sa mení iba konkrétny názov premennej v tej istej chybe, používa sa existujúce číslo.

Rozsahy: 1000–1999 typy a diagnostiky, 2000–2999 lexer/Unicode, 3000–3999 parser, 4000–4999 vykonávanie, 5000–5999 AST a kompilátor, 9000–9999 CLI/neklasifikované.

[Zdroj čísiel](../nobody/errors.py) · [Regresné testy](../tests/test_020_errors.py) · [Roadmap](ROADMAP.md).

## Additional Rust parser errors (English, 0.23 experimental)

### N3006 — Unsupported or misplaced construct in the current Rust parser
**Exact cause:** The Rust parser encounters a statement/keyword not supported at the current position or a reserved keyword from a not-yet-implemented part of the grammar. A detached `inak` (`else`) outside an `ak` (`if`) is an example.
**How to fix:** Attach `inak` directly after the preceding `ak { ... }` block. For other missing features, see [Rust supported syntax](../rust/README.md) and use the Python legacy reference only until the Rust migration covers that feature.
**Example:** `nobody-rs parse sk program.nbd` with `inak { vypis(1) }` but no preceding `ak` produces N3006. A correctly formed `ak pravda { vypis(1) } inak { vypis(2) }` now parses successfully.

### N3007 — Unsupported language profile in the current Rust parser
**Exact cause:** The selected `--lang` equivalent for the current Rust CLI is not one of `sk`, `en`, `ai`. Other human languages exist in the Python reference but have not been ported into the Rust parser.
**How to fix:** Use `nobody-rs parse sk FILE`, `nobody-rs parse en FILE` or `nobody-rs parse ai FILE` with matching source. Do not silently reinterpret the file as another language.
**Example:** `nobody-rs parse ja program.nbd` produces N3007.

### N3008 — Return outside a function

**Exact cause:** The `vrat` (`return`, `ai_return`) instruction appears at the top level or in a top-level conditional/loop rather than in the body of a function. A return can terminate a function call, not a whole source file.

**How to fix:** Move the return statement inside a named function definition, or remove it if the top-level code should simply finish.

**Incorrect example:**
```nobody
nech x = 3
vrat x
```

**Correct example:**
```nobody
funkcia get_x() { vrat 3 }
```

The Rust `validate` command reports `N3008` while leaving a precise explanation and remedy.

### N3009 — Break or continue outside a loop in the current function

**Exact cause:** `prerus` / `pokracuj` (`break` / `continue`, `ai_break` / `ai_continue`) is not lexically nested in a `while` or `for` loop belonging to the same function. **A surrounding loop does not count for a separately declared nested function**, since that function may be invoked after the loop ends.

**How to fix:** Move the instruction inside a `while` or `for` loop defined within the same function. Do not place it in a function merely defined in an outer loop.

**Incorrect example:**
```nobody
funkcia f() { prerus }
```

**Correct example:**
```nobody
funkcia f() {
    pokial pravda { prerus }
}
```

**Known diagnostic limitation:** Rust parser AST nodes do not yet retain their original line and column. The validator therefore reports the N-code and cause without inventing a source location. Accurate diagnostic spans remain an open roadmap task.

### N3010 — Legacy prefixed AI keyword

**Exact cause:** Source selected as Rust syntax profile `ai` contains a recognized keyword from the original Python-reference AI dialect, such as `ai_let`, `ai_def`, `ai_if` or `ai_print`. The redesigned Rust-first AI dialect instead uses familiar keywords `let`, `fn`, `if` and `print`.

**How to fix:** Convert the file with the token-aware migration command, which preserves quoted text and comments:

```bash
cargo run --manifest-path rust/Cargo.toml -- migrate-ai old_ai.nbd > new_ai.nbd
cargo run --manifest-path rust/Cargo.toml -- validate ai new_ai.nbd
```

**Incorrect:** `ai_let n: ai_int = 3`

**Correct:** `let n: int = 3`

The converter writes to standard output and does **not** overwrite the original file. This message does not imply that Rust execution or whole-program parity is implemented.

### N3011 — Identifier conflicts with a target-language keyword

**Exact cause:** A source program contains an identifier that is a legal name in the original Nobody syntax, but becomes a reserved keyword (or a built-in keyword token) in the requested target syntax. Silently translating the rest of the program would change how that identifier is parsed, so the Rust translator stops and reports the exact identifier, target language, line and column.

**How to fix:** Rename the identifier in the *source* program to a neutral name that is not reserved in the target profile, then translate again. This is not resolved by changing the spelling of the keyword in the destination after translation.

**Incorrect Slovak source for AI translation:**

```text
nech fn = 3
vypis(fn)
```

In the redesigned AI syntax, `fn` is reserved for function declarations. Translating `nech fn` to `let fn` would produce invalid/ambiguous code. The translator reports **N3011**, pointing at `fn`.

**Corrected source:**

```text
nech hodnota = 3
vypis(hodnota)
```

This safely becomes:

```text
let hodnota = 3
print(hodnota)
```

Quoted string literals and comments are not changed and cannot trigger N3011 merely by containing a reserved word. This is an experimental **fail-closed token check**, not yet a full scope-aware renaming and semantic-equivalence proof.

### N1001 — Known static type mismatch (Rust IR checker)

**Exact cause:** A typed variable is initialized with a value whose type is definitely incompatible with its declared type, or a later assignment to that variable has a definitely incompatible type. For example, a string is not an integer; a Boolean is not an integer. The Rust checker permits assigning an integer to a `float` declaration. Expressions with unknown types are deferred rather than rejected.

**Incorrect AI-profile source:**

```text
let count: int = "wrong"
```

The checker reports `N1001` and names `count`, the expected type `Int` and the known actual type `Str`.

**Correct:**

```text
let count: int = 3
```

**Run:**

```bash
cargo run --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/typecheck_bad_ai.nbd
```

**Important limitation:** Canonical IR v1 currently has no statement-level source spans, so this diagnostic **does not invent a line/column**. It gives the exact variable and mismatched types. The type checker is conservative: unknown function results, undeclared assignment analysis, control-flow joins and runtime guarantees require future work. A successful check does not prove that a program is safe or executable.


### N1003 — Assignment to an undeclared name (Rust scope checker)

**Exact cause:** An assignment targets a name that has not been declared by `let` or a visible function/parameter in the current or enclosing lexical scope.

**Incorrect:**
```text
count = 3
```

**Correct:**
```text
let count = 3
count = 4
```

**Repair:** Declare the variable before its first assignment, or correct the misspelled name. The checker permits assignments to a variable declared in an enclosing block. This check is lexical and does not yet model imports, closures or runtime globals.

### N1004 — Duplicate declaration in one lexical block (Rust scope checker)

**Exact cause:** The same variable or function name is declared more than once in a single block, or a function declares the same parameter twice. An inner block may independently shadow an outer declaration.

**Incorrect:**
```text
let count = 1
let count = 2
```

**Correct:**
```text
let count = 1
count = 2
```

**Also incorrect:** `fn sum(x, x) { return x }`.

**Repair:** Use ordinary assignment to update a variable, rename the duplicate declaration, or give each function parameter a unique name. A separate child block can intentionally introduce a local shadowing declaration.

**Location limitation:** The IR checker still lacks statement-level source spans, so N1003 and N1004 name the affected symbol but do not fabricate line or column positions.

### N1005 — Incorrect argument count for a known function (Rust type checker)

**Exact cause:** A direct call to a function declared in the visible lexical scope supplies a different number of positional arguments than the function's parameter list. For example, a function declared with two parameters is called with one argument. Recursive calls are checked, too.

**Incorrect:**

```text
fn add(a, b) { return a + b }
add(1)
```

**Correct:**

```text
fn add(a, b) { return a + b }
add(1, 2)
```

**Repair:** Add or remove positional arguments to match the declared function parameters, or correct the function signature. A parameter or local variable that shadows an outer function disables the outer function's arity assumption. Unknown dynamic callees are deferred rather than guessed.

**Run:** `cargo run --manifest-path rust/Cargo.toml -- typecheck ai rust/examples/arity_bad_ai.nbd`.

**Limitations:** IR v1 does not yet preserve source spans for N1005, and complete closure/overload semantics are not implemented. Successful Rust build/test execution remains unverified.

### N1002 — Definitely incompatible arithmetic operands (Rust checker)

**Exact cause:** A primitive arithmetic expression uses two *known* types that do not support the selected operator. For example, `"text" - 1` or `"text" + 1`. Numeric operands can be combined arithmetically, while `+` also supports two strings.

**Incorrect:** `let total = "five" + 5`

**Correct:** `let total = 5 + 5` (numeric addition), or `let total = "five" + " items"` (string concatenation).

**Repair:** Change both operands to compatible numbers, or use two strings for concatenation. Nested expressions and function-call arguments are checked. Unknown operand types are deferred; a passing result does **not** establish runtime safety. IR v1 still lacks statement-level source spans.

### N1006 — Invalid operand for unary numeric operator (Rust checker)

**Exact cause:** Unary `+` or `-` is applied to a value whose primitive type is known to be nonnumeric, such as a string or boolean.

**Incorrect:** `let x = -"oops"`

**Correct:** `let x = -3` or `let x = +2.5`.

**Repair:** Use an integer or floating-point value, or explicitly convert the input to a numeric type. Unknown/dynamic values are deferred rather than assumed invalid. The Rust IR checker currently lacks statement source spans; N1006 does not invent a line number.

### N2006 — Neplatný desiatkový číselný literál (Rust lexer)

**Príčina:** Číselný token obsahuje neplatné znaky alebo nesprávne umiestnené oddeľovače, napríklad `12abc`, `1__2`, `1e+` alebo `0xFF`. Súčasný Rust lexer podporuje iba desiatkové celé a desatinné čísla, s voliteľným exponentom a podčiarkovníkmi medzi číslicami.

**Oprava:** Použi `12`, `12.5`, `1_000` alebo `1e-3`. Ak má nasledovať identifikátor, oddeľ ho podľa gramatiky; hexadecimálne literály zatiaľ nie sú podporované. Chyba hlási riadok a stĺpec začiatku číselného tokenu. **Úspešná kompilácia a Rust CI ešte nie sú overené.**

### N2007 — Miešanie zameniteľných písiem v identifikátore (Rust lexer)

**Presná príčina:** Jeden názov premennej, funkcie alebo kľúčového slova obsahuje súčasne písmená najmenej dvoch písiem z množiny **latinka, gréčtina a cyrilika**. Vizuálne podobné písmená z rôznych písiem môžu zakryť preklep alebo podvrhnutie názvu. Napríklad v `let pаypal = 3` je druhý znak názvu cyrilské `а` (U+0430).

**Oprava:** Použi v danom názve písmená jedného z týchto troch písiem. Skontroluj problematické písmeno na uvedenom riadku a stĺpci. Textové literály a komentáre sa týmto pravidlom neobmedzujú.

**Rozsah:** Ide o **úzku ochranu**, nie o všeobecnú detekciu Unicode confusables či implementáciu UTS #39. Iné písma a bežné kombinácie, napríklad japonské kandži a hiragana, sa automaticky nezakazujú. Unicode NFC/NFKC normalizácia zostáva otvorenou úlohou.

### N2008 — Identifikátor nie je normalizovaný do NFC (Rust lexer)

**Príčina:** Názov obsahuje kanonicky ekvivalentný, ale nenormalizovaný zápis Unicode. Napríklad text \`cafe\u{301}\` obsahuje kombináciu \`e\` a U+0301, zatiaľ čo \`café\` používa vopred zložené U+00E9. Bez jednotného pravidla sa vizuálne totožné názvy môžu správať ako odlišné premenné.

**Oprava:** Normalizuj iba názvy identifikátorov do štandardnej Unicode NFC formy, napríklad použi \`café\`. Lexer nenormalizuje ani neprepisuje zdrojový text potichu: vráti N2008 na začiatku problematického identifikátora. Zachováva presné bajtové pozície, textové literály a komentáre.

**Rozsah a závislosť:** Kontrola používa pripnutú knižnicu \`unicode-normalization = "=0.1.24"\`. Nejde o NFKC, transliteráciu, automatické opravovanie názvov ani úplnú ochranu proti vizuálnym zámenám (UTS #39). N2007 a N2008 sú nezávislé.


### N1007 — Unknown explicit primitive type annotation (Rust checker)

**Exact cause:** A declaration explicitly names a type that the current Rust type checker does not recognize. The supported primitive types are `int`, `float`, `str`, and `bool`, or their canonical keyword translations in the selected Nobody syntax profile. An unrecognized annotation is **not** silently treated as dynamic typing.

**Incorrect:** `let age: intr = 3` (typo in `int`).

**Correct:** `let age: int = 3`. In Slovak syntax, `nech vek: cele = 3` is valid.

**Repair:** Use a recognized type keyword or remove the annotation if inference / unknown typing is intended. Unannotated dynamic expressions remain conservatively accepted.

**Location and limitations:** The Rust parser reports N1007 at the unknown annotation token with its actual line and Unicode-scalar column. The lower-level IR type checker also rejects unrecognized explicit annotations with N1007, but standalone IR v1 does not carry source locations and therefore does not invent them. Custom types, generics, aliases and modules are **not yet implemented**; this restriction is not permanent. The change does not provide a sound whole-program type checker.


### N1008 — Definitely zero numeric literal divisor (Rust static checker)

**Exact cause:** A division, floor division or remainder operator (`/`, `//`, `%`) receives a divisor that is syntactically certain to be the numeric literal zero. Examples: `let x = 5 / 0`, `let x = 5 // -0.0`, `let x = 5 % 0e10`.

**Repair:** Replace the literal zero with a nonzero divisor. If the value comes from input or calculation, guard it before division. The diagnostic explicitly identifies the operator but, since IR v1 has no expression source spans, it does not invent a line or column.

**Expanded narrow constant checking:** N1008 additionally evaluates exact integer-only expressions built from literals, unary `+`/`-` and binary `+`/`-`/`*`, using checked `i128` arithmetic. For example, `10 / (5 - 5)` now reports N1008. This does not execute source code or perform general constant folding. Integer overflows, floating-point arithmetic, division inside the divisor, variables and function calls remain unknown rather than being guessed. Literal zero remains recognized for decimal fractions and exponent notation. A successful static check is **not** a runtime guarantee against division by zero. Type errors such as `"text" / 0` retain N1002 priority; this is part of `typecheck`, not the lexer-only check.


### N4006 — Zero range step at runtime (experimental Rust evaluator)

The opt-in Rust `run` command rejects `for i in range(start, stop, 0)` with N4006. **Cause:** the third integer argument of `range` is zero, which cannot advance the iterator. **Repair:** supply a nonzero step; for descending sequences, use a negative step. This does not imply support for arbitrary iterables, generators, or complete Python-like semantics.

## Rust evaluator prototype (0.39): N4001–N4005

These diagnostics apply to the explicitly invoked `nobody-rs run PROFILE FILE` command, not to source-checking commands. This limited Rust evaluator is not a complete interpreter or compiler. Runtime IR locations are not yet available.

- **N4001 — Unknown variable:** A variable was read or assigned before declaration. Declare it first.
- **N4002 — Zero divisor:** Integer division or remainder received zero. Check the divisor.
- **N4003 — Integer overflow:** Signed 128-bit arithmetic or integer literal exceeds the supported range. Use smaller values.
- **N4004 — Unsupported runtime feature:** A parsed statement or expression is not in the evaluator subset. Nested function definitions, closures, function values, typed declarations and floating-point execution remain unsupported.
- **N4005 — Resource counter reached:** The evaluator has exceeded its step, nesting, text, or output budget. These limits are not an operating-system sandbox or complete resource isolation.

The `run` command supports integer arithmetic, basic text/boolean/null values, simple declarations, assignments, conditionals, bounded loops, named non-recursive function calls with parameters and `return`, and unshadowed `print`. Function calls use a snapshot of top-level variables and an isolated parameter/local scope; mutations to globals inside calls are not persisted. Parsing and type-checking do not execute programs. Parsing and type-checking do not execute programs.

### N4007 — Wrong runtime function argument count

The experimental Rust `run` evaluator reports **N4007** if a named function is called with an incorrect number of arguments. **Repair:** pass exactly the number of parameters declared for that function. This runtime diagnostic does not replace the conservative static arity check N1005. Recursion remains deliberately rejected as N4005 until a safe execution design is established.
