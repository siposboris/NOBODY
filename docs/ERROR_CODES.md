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
| **N2005** | `LEX_BIDI` | Skrytý riadiaci znak smeru |
| **N3001** | `PARSE_EXPECT` | Chýbajúci prvok |
| **N3002** | `PARSE_BLOCK` | Nesprávne zátvorky bloku |
| **N3003** | `PARSE_IDENTIFIER` | Neplatný názov |
| **N3004** | `PARSE_LANGUAGE` | Cudzie kľúčové slovo |
| **N3005** | `PARSE_EXPRESSION` | Neplatný výraz |
| **N4001** | `RUN_NAME` | Neexistujúce meno |
| **N4002** | `RUN_TYPE` | Typová chyba za behu |
| **N4003** | `RUN_LIMIT` | Prekročený limit vykonávania |
| **N4004** | `RUN_CALL` | Neplatné volanie |
| **N5001** | `AST_SCHEMA` | Neplatný AST dokument |
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

**Čo presne znamená:** Lexer nerozpoznal znak mimo komentára a textového literálu.

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
