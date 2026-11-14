//! Rust port of the Python bootstrap's numbered diagnostic catalogue.
//! Keep identifiers and Slovak user-facing explanations compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Definition {
    pub number: u16,
    pub symbol: &'static str,
    pub title: &'static str,
    pub cause: &'static str,
    pub fix: &'static str,
}

pub const DEFINITIONS: &[Definition] = &[
    Definition {
        number: 1001,
        symbol: "E_TYPE",
        title: "Nesúlad typu",
        cause: "Priraďovaná hodnota má iný známy typ než deklarovaná premenná.",
        fix: "Oprav hodnotu na deklarovaný typ alebo typovú anotáciu za dvojbodkou.",
    },
    Definition {
        number: 1002,
        symbol: "E_BINARY",
        title: "Nekompatibilné operandy",
        cause: "Aritmetický operátor sa používa na typy, pre ktoré daná operácia nie je definovaná.",
        fix: "Pre odčítanie a násobenie použi číselné hodnoty; na spojenie dvoch textov použi +.",
    },
    Definition {
        number: 1003,
        symbol: "E_UNDECLARED",
        title: "Nedeklarovaná premenná",
        cause: "Priradenie mení názov, ktorý nie je deklarovaný vo viditeľnom rozsahu.",
        fix: "Najskôr použi nech/ai_let, alebo oprav preklep v názve premennej.",
    },
    Definition {
        number: 1004,
        symbol: "E_DUPLICATE",
        title: "Duplicitná deklarácia",
        cause: "Rovnaké meno je deklarované druhýkrát v tom istom lexikálnom rozsahu.",
        fix: "Premenuj druhú deklaráciu alebo použi jednoduché priradenie bez nech.",
    },
    Definition {
        number: 1005,
        symbol: "E_ARITY",
        title: "Nesprávny počet argumentov",
        cause: "Priame volanie známej funkcie nesúhlasí s počtom deklarovaných parametrov.",
        fix: "Doplň alebo odstráň argumenty podľa parametrov uvedených pri definícii funkcie.",
    },
    Definition {
        number: 1006,
        symbol: "E_UNARY",
        title: "Neplatné číselné znamienko",
        cause: "Znamienko + alebo - je použité na známy nečíselný typ.",
        fix: "Použi celé/desatinné číslo alebo znamienko odstráň.",
    },
    Definition {
        number: 1101,
        symbol: "W_DYNAMIC_NAME",
        title: "Neznámy typ mena",
        cause: "Statický analyzátor nevie preukázať typ daného identifikátora.",
        fix: "Skontroluj deklaráciu, typovú anotáciu a miesto, kde je funkcia alebo premenná definovaná.",
    },
    Definition {
        number: 2001,
        symbol: "LEX_STRING",
        title: "Neukončený reťazec",
        cause: "Za otváracou úvodzovkou chýba ukončovacia úvodzovka.",
        fix: "Doplň zodpovedajúcu úvodzovku; viacriadkové reťazce uzatváraj tromi.",
    },
    Definition {
        number: 2002,
        symbol: "LEX_CASE",
        title: "Nepovolené veľké písmená",
        cause: "Názov obsahuje veľké písmená, hoci syntax vyžaduje lowercase.",
        fix: "Prepíš konkrétny názov malými písmenami v pôvodnom písme.",
    },
    Definition {
        number: 2003,
        symbol: "LEX_COMBINING",
        title: "Osamotená diakritická značka",
        cause: "Unicode kombinujúci znak nemá predchádzajúce základné písmeno.",
        fix: "Vlož základné písmeno pred diakritiku alebo oprav poradie Unicode znakov.",
    },
    Definition {
        number: 2004,
        symbol: "LEX_CHARACTER",
        title: "Nepovolený znak",
        cause: "Lexer nerozpoznal znak mimo komentára a textového literálu.",
        fix: "Odstráň uvedený znak alebo ho vlož medzi úvodzovky.",
    },
    Definition {
        number: 2005,
        symbol: "LEX_BIDI",
        title: "Skrytý riadiaci znak smeru",
        cause: "Zdroj obsahuje Unicode znak vizuálne meniaci poradie zdrojového kódu.",
        fix: "Odstráň konkrétny U+ znak; pre RTL použi nastavenie editora.",
    },
    Definition {
        number: 3001,
        symbol: "PARSE_EXPECT",
        title: "Chýbajúci prvok",
        cause: "Parser očakával konkrétnu zátvorku, operátor alebo iný token.",
        fix: "Doplň token uvedený v správe na uvedenom mieste.",
    },
    Definition {
        number: 3002,
        symbol: "PARSE_BLOCK",
        title: "Nesprávne zátvorky bloku",
        cause: "Blok má prebytočnú alebo chýbajúcu zloženú zátvorku.",
        fix: "Doplň } alebo odstráň prebytočnú } v označenom bloku.",
    },
    Definition {
        number: 3003,
        symbol: "PARSE_IDENTIFIER",
        title: "Neplatný názov",
        cause: "Parser očakával identifikátor a našiel neplatný token.",
        fix: "Po nech/funkcia uveď platný názov, ktorý nie je rezervované slovo.",
    },
    Definition {
        number: 3004,
        symbol: "PARSE_LANGUAGE",
        title: "Cudzie kľúčové slovo",
        cause: "Príkaz patrí inému než nakonfigurovanému jazyku súboru.",
        fix: "Použi preklad kľúčového slova pre aktuálne --lang.",
    },
    Definition {
        number: 3005,
        symbol: "PARSE_EXPRESSION",
        title: "Neplatný výraz",
        cause: "Výraz má neplatný začiatok alebo chýbajúci operand.",
        fix: "Skontroluj operand a zátvorky pred miestom chyby.",
    },
    Definition {
        number: 3006,
        symbol: "PARSE_UNSUPPORTED",
        title: "Nepodporovaná konštrukcia v Rust parseri",
        cause: "Rust parser zatiaľ nerozpoznáva uvedený príkaz, napríklad podmienku ak alebo funkciu.",
        fix: "Použi podporovanú deklaráciu, priradenie alebo výraz; na ostatné konštrukcie počkaj na port parsera.",
    },
    Definition {
        number: 3007,
        symbol: "PARSE_PROFILE",
        title: "Nepodporovaný jazykový profil Rust parsera",
        cause: "Nový čiastočný Rust parser zatiaľ pozná iba profily sk, en a ai.",
        fix: "Spusti nobody-rs parse sk, en alebo ai so správnym súborom; ďalšie profily zatiaľ používa len starý referenčný prototyp.",
    },
    Definition {
        number: 3008,
        symbol: "RETURN_CONTEXT",
        title: "Return mimo funkcie",
        cause: "Príkaz return/vrat/ai_return sa nachádza mimo tela deklarovanej funkcie.",
        fix: "Presuň return do funkcie alebo ho odstráň; na najvyššej úrovni súboru sa return nepoužíva.",
    },
    Definition {
        number: 3009,
        symbol: "LOOP_CONTROL_CONTEXT",
        title: "Break alebo continue mimo cyklu",
        cause: "Príkaz break/continue sa nenachádza vo vnútri cyklu while/for aktuálnej funkcie.",
        fix: "Presuň príkaz do cyklu tej istej funkcie. Cyklus okolo definície vnorenej funkcie sa nepočíta.",
    },
    Definition {
        number: 3010,
        symbol: "AI_LEGACY_SYNTAX",
        title: "Zastaralá AI syntax s predponami",
        cause: "Zdrojový súbor s profilom ai obsahuje staré kľúčové slovo napríklad ai_let alebo ai_if, ktoré nová Rust syntax nepoužíva.",
        fix: "Preveď súbor príkazom nobody-rs migrate-ai stary.nbd > novy.nbd; nová ai syntax používa let, fn, if, return, true, false.",
    },
    Definition {
        number: 3011,
        symbol: "TRANSLATION_COLLISION",
        title: "Kolízia identifikátora s cieľovým kľúčovým slovom",
        cause: "Názov používateľskej premennej alebo funkcie je rezervovaným slovom vo zvolenej cieľovej syntaxi, preto by preklad zmenil význam programu.",
        fix: "Premenuj tento identifikátor v zdrojovom Nobody programe na názov, ktorý nie je rezervovaný v cieľovom profile; potom preklad zopakuj.",
    },
    Definition {
        number: 4001,
        symbol: "RUN_NAME",
        title: "Neexistujúce meno",
        cause: "Počas vykonávania premenná nie je dostupná v aktuálnom rozsahu.",
        fix: "Deklaruj ju pred použitím alebo ju odovzdaj ako parameter.",
    },
    Definition {
        number: 4002,
        symbol: "RUN_TYPE",
        title: "Typová chyba za behu",
        cause: "Dynamicky vypočítaná hodnota sa nezhoduje s požadovaným typom.",
        fix: "Skontroluj typ pri deklarácii aj skutočnú hodnotu výrazu.",
    },
    Definition {
        number: 4003,
        symbol: "RUN_LIMIT",
        title: "Prekročený limit vykonávania",
        cause: "Prekročený počet krokov alebo hĺbka rekurzívnych volaní.",
        fix: "Skontroluj ukončenie cyklu/rekurzie, až potom zváž zvýšenie limitu.",
    },
    Definition {
        number: 4004,
        symbol: "RUN_CALL",
        title: "Neplatné volanie",
        cause: "Nesedí počet argumentov alebo cieľ nie je dovolená funkcia.",
        fix: "Porovnaj parametre deklarácie s volaním a dostupnými built-in funkciami.",
    },
    Definition {
        number: 5001,
        symbol: "AST_SCHEMA",
        title: "Neplatný AST dokument",
        cause: "JSON obsahuje neplatnú schému alebo štruktúru uzla.",
        fix: "Vygeneruj platný nobody.ast.v1 cez ast-json a skontroluj povolené polia.",
    },
    Definition {
        number: 9001,
        symbol: "CLI_INPUT",
        title: "Neplatný vstup alebo konfigurácia",
        cause: "Súbor neexistuje alebo nastavenie nemá platnú hodnotu.",
        fix: "Skontroluj cestu, UTF-8, jazykový profil a názov nastavenia.",
    },
    Definition {
        number: 9999,
        symbol: "UNCLASSIFIED",
        title: "Doteraz nezaradená chyba",
        cause: "Modul vyvolal chybu, ktorá ešte nemá samostatné číslo v katalógu.",
        fix: "Zachovaj presnú hlášku a reprodukčný .nbd súbor, aby vznikol nový konkrétny kód.",
    },
];

pub fn lookup(number: u16) -> Option<&'static Definition> {
    DEFINITIONS.iter().find(|entry| entry.number == number)
}

pub fn format_entry(entry: &Definition) -> String {
    format!(
        "N{:04} [{}]: {}\n  Príčina: {}\n  Oprava: {}",
        entry.number, entry.symbol, entry.title, entry.cause, entry.fix
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn legacy_diagnostics_are_unique_and_complete() {
        assert_eq!(DEFINITIONS.len(), 30);
        let numbers: HashSet<_> = DEFINITIONS.iter().map(|d| d.number).collect();
        let symbols: HashSet<_> = DEFINITIONS.iter().map(|d| d.symbol).collect();
        assert_eq!(numbers.len(), DEFINITIONS.len());
        assert_eq!(symbols.len(), DEFINITIONS.len());
        assert_eq!(lookup(1001).unwrap().symbol, "E_TYPE");
        assert_eq!(lookup(9999).unwrap().symbol, "UNCLASSIFIED");
        assert!(lookup(42).is_none());
    }
}

 
/// Classify legacy bootstrap error messages without launching Python.
/// Pattern precedence and ASCII case-insensitive matching mirror errors.py.
pub fn classify_message(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    let patterns: &[(&[&str], &str)] = &[
        (&["unterminated string"], "LEX_STRING"),
        (&["must be lowercase"], "LEX_CASE"),
        (&["combining mark"], "LEX_COMBINING"),
        (&["non-ascii character"], "LEX_CHARACTER"),
        (&["forbidden bidirectional control"], "LEX_BIDI"),
        (&["foreign keyword"], "PARSE_LANGUAGE"),
        (
            &["expected identifier", "expected user identifier"],
            "PARSE_IDENTIFIER",
        ),
        (&["unclosed block", "closing brace"], "PARSE_BLOCK"),
        (&["expected expression"], "PARSE_EXPRESSION"),
        (
            &[
                "expected ",
                "unexpected end",
                "assignment target",
                "supported types",
            ],
            "PARSE_EXPECT",
        ),
        (&["unknown variable", "unknown name"], "RUN_NAME"),
        (&["type mismatch"], "RUN_TYPE"),
        (&["step limit", "call depth", "recursion"], "RUN_LIMIT"),
        (
            &[
                "wrong argument count",
                "only whitelisted functions",
                "call not permitted",
            ],
            "RUN_CALL",
        ),
        (
            &[
                "ast schema",
                "ast root",
                "ast node",
                "ast nesting",
                "ast json",
                "invalid ast",
                "unknown ast",
            ],
            "AST_SCHEMA",
        ),
        (
            &[
                "unsupported language",
                "unsupported syntax language",
                "unsupported output",
                "configuration",
                "missing ",
                "unknown setting",
                "no such file",
            ],
            "CLI_INPUT",
        ),
    ];
    for &(needles, symbol) in patterns {
        if needles.iter().any(|needle| lower.contains(needle)) {
            return symbol;
        }
    }
    "UNCLASSIFIED"
}

/// Render a legacy-style numbered diagnostic, including its actionable fix.
pub fn format_message(message: &str, line: Option<usize>) -> String {
    let symbol = classify_message(message);
    let entry = DEFINITIONS
        .iter()
        .find(|entry| entry.symbol == symbol)
        .expect("all classification symbols must be in the diagnostic catalogue");
    let location = line.map_or_else(String::new, |number| format!("riadok {number}: "));
    format!(
        "N{:04} [{}] {}{}. Oprava: {}",
        entry.number, symbol, location, message, entry.fix
    )
}

#[cfg(test)]
mod legacy_classification_tests {
    use super::*;

    #[test]
    fn preserves_legacy_pattern_priority_and_case_insensitivity() {
        let cases = [
            ("Unterminated string", "LEX_STRING"),
            ("MUST BE LOWERCASE", "LEX_CASE"),
            ("combining mark", "LEX_COMBINING"),
            ("non-ascii character", "LEX_CHARACTER"),
            ("Forbidden bidirectional control", "LEX_BIDI"),
            ("Foreign keyword", "PARSE_LANGUAGE"),
            ("Expected user identifier", "PARSE_IDENTIFIER"),
            ("Unclosed block", "PARSE_BLOCK"),
            ("Expected expression", "PARSE_EXPRESSION"),
            ("Unexpected end", "PARSE_EXPECT"),
            ("Unknown variable", "RUN_NAME"),
            ("Type mismatch", "RUN_TYPE"),
            ("recursion", "RUN_LIMIT"),
            ("Wrong argument count", "RUN_CALL"),
            ("Invalid AST", "AST_SCHEMA"),
            ("No such file", "CLI_INPUT"),
            ("completely new failure", "UNCLASSIFIED"),
            (
                "Expected expression and Unknown variable",
                "PARSE_EXPRESSION",
            ),
        ];
        for (message, expected) in cases {
            assert_eq!(classify_message(message), expected, "{message}");
        }
    }

    #[test]
    fn formats_numbered_diagnostics_with_optional_line() {
        let diagnostic = format_message("Unknown name x", Some(7));
        assert!(diagnostic.starts_with("N4001 [RUN_NAME] riadok 7: Unknown name x. Oprava: "));
        assert!(format_message("unrecognized failure", None).starts_with("N9999 [UNCLASSIFIED] "));
    }
}
