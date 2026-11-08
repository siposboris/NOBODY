"""Append-only numbered diagnostics. Each code has an actionable remedy."""
from dataclasses import dataclass
import re

@dataclass(frozen=True)
class ErrorDefinition:
    number:int
    symbol:str
    title:str
    cause:str
    fix:str

ENTRIES=[
(1001,'E_TYPE','Nesúlad typu','Priraďovaná hodnota má iný známy typ než deklarovaná premenná.','Oprav hodnotu na deklarovaný typ alebo typovú anotáciu za dvojbodkou.'),
(1002,'E_BINARY','Nekompatibilné operandy','Aritmetický operátor sa používa na typy, pre ktoré daná operácia nie je definovaná.','Pre odčítanie a násobenie použi číselné hodnoty; na spojenie dvoch textov použi +.'),
(1003,'E_UNDECLARED','Nedeklarovaná premenná','Priradenie mení názov, ktorý nie je deklarovaný vo viditeľnom rozsahu.','Najskôr použi nech/ai_let, alebo oprav preklep v názve premennej.'),
(1004,'E_DUPLICATE','Duplicitná deklarácia','Rovnaké meno je deklarované druhýkrát v tom istom lexikálnom rozsahu.','Premenuj druhú deklaráciu alebo použi jednoduché priradenie bez nech.'),
(1005,'E_ARITY','Nesprávny počet argumentov','Priame volanie známej funkcie nesúhlasí s počtom deklarovaných parametrov.','Doplň alebo odstráň argumenty podľa parametrov uvedených pri definícii funkcie.'),
(1006,'E_UNARY','Neplatné číselné znamienko','Znamienko + alebo - je použité na známy nečíselný typ.','Použi celé/desatinné číslo alebo znamienko odstráň.'),
(1101,'W_DYNAMIC_NAME','Neznámy typ mena','Statický analyzátor nevie preukázať typ daného identifikátora.','Skontroluj deklaráciu, typovú anotáciu a miesto, kde je funkcia alebo premenná definovaná.'),
(2001,'LEX_STRING','Neukončený reťazec','Za otváracou úvodzovkou chýba ukončovacia úvodzovka.','Doplň zodpovedajúcu úvodzovku; viacriadkové reťazce uzatváraj tromi.'),
(2002,'LEX_CASE','Nepovolené veľké písmená','Názov obsahuje veľké písmená, hoci syntax vyžaduje lowercase.','Prepíš konkrétny názov malými písmenami v pôvodnom písme.'),
(2003,'LEX_COMBINING','Osamotená diakritická značka','Unicode kombinujúci znak nemá predchádzajúce základné písmeno.','Vlož základné písmeno pred diakritiku alebo oprav poradie Unicode znakov.'),
(2004,'LEX_CHARACTER','Nepovolený znak','Lexer nerozpoznal znak mimo komentára a textového literálu.','Odstráň uvedený znak alebo ho vlož medzi úvodzovky.'),
(2005,'LEX_BIDI','Skrytý riadiaci znak smeru','Zdroj obsahuje Unicode znak vizuálne meniaci poradie zdrojového kódu.','Odstráň konkrétny U+ znak; pre RTL použi nastavenie editora.'),
(3001,'PARSE_EXPECT','Chýbajúci prvok','Parser očakával konkrétnu zátvorku, operátor alebo iný token.','Doplň token uvedený v správe na uvedenom mieste.'),
(3002,'PARSE_BLOCK','Nesprávne zátvorky bloku','Blok má prebytočnú alebo chýbajúcu zloženú zátvorku.','Doplň } alebo odstráň prebytočnú } v označenom bloku.'),
(3003,'PARSE_IDENTIFIER','Neplatný názov','Parser očakával identifikátor a našiel neplatný token.','Po nech/funkcia uveď platný názov, ktorý nie je rezervované slovo.'),
(3004,'PARSE_LANGUAGE','Cudzie kľúčové slovo','Príkaz patrí inému než nakonfigurovanému jazyku súboru.','Použi preklad kľúčového slova pre aktuálne --lang.'),
(3005,'PARSE_EXPRESSION','Neplatný výraz','Výraz má neplatný začiatok alebo chýbajúci operand.','Skontroluj operand a zátvorky pred miestom chyby.'),
(3006,'PARSE_UNSUPPORTED','Nepodporovaná konštrukcia v Rust parseri','Rust parser zatiaľ nerozpoznáva uvedený príkaz, napríklad podmienku ak alebo funkciu.','Použi podporovanú deklaráciu, priradenie alebo výraz; na ostatné konštrukcie počkaj na port parsera.'),
(3007,'PARSE_PROFILE','Nepodporovaný jazykový profil Rust parsera','Nový čiastočný Rust parser zatiaľ pozná iba profily sk, en a ai.','Spusti nobody-rs parse sk, en alebo ai so správnym súborom; ďalšie profily zatiaľ používa len starý referenčný prototyp.'),
(3008,'RETURN_CONTEXT','Return mimo funkcie','Príkaz return/vrat/ai_return sa nachádza mimo tela deklarovanej funkcie.','Presuň return do funkcie alebo ho odstráň; na najvyššej úrovni súboru sa return nepoužíva.'),
(3009,'LOOP_CONTROL_CONTEXT','Break alebo continue mimo cyklu','Príkaz break/continue sa nenachádza vo vnútri cyklu while/for aktuálnej funkcie.','Presuň príkaz do cyklu tej istej funkcie. Cyklus okolo definície vnorenej funkcie sa nepočíta.'),
(3010,'AI_LEGACY_SYNTAX','Zastaralá AI syntax s predponami','Zdrojový súbor s profilom ai obsahuje staré kľúčové slovo napríklad ai_let alebo ai_if, ktoré nová Rust syntax nepoužíva.','Preveď súbor príkazom nobody-rs migrate-ai stary.nbd > novy.nbd; nová ai syntax používa let, fn, if, return, true, false.'),
(4001,'RUN_NAME','Neexistujúce meno','Počas vykonávania premenná nie je dostupná v aktuálnom rozsahu.','Deklaruj ju pred použitím alebo ju odovzdaj ako parameter.'),
(4002,'RUN_TYPE','Typová chyba za behu','Dynamicky vypočítaná hodnota sa nezhoduje s požadovaným typom.','Skontroluj typ pri deklarácii aj skutočnú hodnotu výrazu.'),
(4003,'RUN_LIMIT','Prekročený limit vykonávania','Prekročený počet krokov alebo hĺbka rekurzívnych volaní.','Skontroluj ukončenie cyklu/rekurzie, až potom zváž zvýšenie limitu.'),
(4004,'RUN_CALL','Neplatné volanie','Nesedí počet argumentov alebo cieľ nie je dovolená funkcia.','Porovnaj parametre deklarácie s volaním a dostupnými built-in funkciami.'),
(5001,'AST_SCHEMA','Neplatný AST dokument','JSON obsahuje neplatnú schému alebo štruktúru uzla.','Vygeneruj platný nobody.ast.v1 cez ast-json a skontroluj povolené polia.'),
(9001,'CLI_INPUT','Neplatný vstup alebo konfigurácia','Súbor neexistuje alebo nastavenie nemá platnú hodnotu.','Skontroluj cestu, UTF-8, jazykový profil a názov nastavenia.'),
(9999,'UNCLASSIFIED','Doteraz nezaradená chyba','Modul vyvolal chybu, ktorá ešte nemá samostatné číslo v katalógu.','Zachovaj presnú hlášku a reprodukčný .nbd súbor, aby vznikol nový konkrétny kód.')
]
CATALOG={x[1]:ErrorDefinition(*x) for x in ENTRIES}
BY_NUMBER={x.number:x for x in CATALOG.values()}
assert len(CATALOG)==len(ENTRIES)==len(BY_NUMBER)

def definition(symbol):
    return CATALOG[symbol]

def format_diagnostic(symbol,message,line=None):
    item=definition(symbol)
    where=f'riadok {line}: ' if line is not None else ''
    return f'N{item.number:04d} [{symbol}] {where}{message}. Oprava: {item.fix}'

def classify_exception(exc):
    message=str(exc)
    patterns=[
      ('Unterminated string','LEX_STRING'),
      ('must be lowercase','LEX_CASE'),
      ('combining mark','LEX_COMBINING'),
      ('non-ascii character','LEX_CHARACTER'),
      ('Forbidden bidirectional control','LEX_BIDI'),
      ('Foreign keyword','PARSE_LANGUAGE'),
      ('Expected user identifier','PARSE_IDENTIFIER'),
      ('Unclosed block|closing brace','PARSE_BLOCK'),
      ('Expected expression','PARSE_EXPRESSION'),
      ('Expected |Unexpected end|Assignment target|Supported types','PARSE_EXPECT'),
      ('Unknown variable|Unknown name','RUN_NAME'),
      ('Type mismatch','RUN_TYPE'),
      ('step limit|call depth|recursion','RUN_LIMIT'),
      ('Wrong argument count|Only whitelisted functions|Call not permitted','RUN_CALL'),
      ('AST schema|AST root|AST node|AST nesting|AST JSON|Invalid AST|Unknown AST','AST_SCHEMA'),
      ('Unsupported language|Unsupported syntax language|Unsupported output|configuration|Missing |Unknown setting|No such file','CLI_INPUT'),
    ]
    for pattern,symbol in patterns:
        if re.search(pattern,message,re.I):
            return symbol
    return 'UNCLASSIFIED'

def render_exception(exc):
    return format_diagnostic(classify_exception(exc),str(exc))
