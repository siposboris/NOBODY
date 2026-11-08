//! Nobody's Rust-native lexer foundation.
//! Source offsets are UTF-8 byte offsets and line/column positions use Unicode
//! scalar values. This is an experimental migration, not yet a full compiler.
//! No Python runtime dependency. Unicode NFC validation uses a pinned Rust crate.

use std::fmt;
use unicode_normalization::is_nfc;

pub mod evaluator;
pub use evaluator::{evaluate_ir, evaluate_source, RuntimeValue};
pub mod ir;
pub mod profiles;
pub mod typecheck;
pub use ir::{
    lower_source, same_ir, IrExpr, IrStatement, Module as IrModule, Name as IrName, IR_VERSION,
};
pub use typecheck::{check_ir, check_source, Type as StaticType};
pub mod translator;
pub use translator::translate_source;
pub mod ai_migration;
pub use ai_migration::migrate_legacy_ai;
pub mod syntax;
pub mod validate;
pub use syntax::{parse, Expr, Language, Program, Statement};
pub use validate::{parse_validated, validate_program};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Name,
    Number,
    String,
    Comment,
    Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: u16,
    pub line: usize,
    pub column: usize,
    pub detail: String,
    pub repair: &'static str,
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 || self.column == 0 {
            // Unknown AST location: never present fabricated line 0 to the user.
            write!(
                f,
                "N{:04}: {}. Oprava: {}",
                self.code, self.detail, self.repair
            )
        } else {
            write!(
                f,
                "N{:04} riadok {}, stlpec {}: {}. Oprava: {}",
                self.code, self.line, self.column, self.detail, self.repair
            )
        }
    }
}
impl std::error::Error for Diagnostic {}

fn failure(code: u16, line: usize, column: usize, detail: impl Into<String>) -> Diagnostic {
    let repair = match code {
        2001 => "Doplň zodpovedajúcu ukončovaciu úvodzovku.",
        2002 => "Prepíš názov malými písmenami v pôvodnom písme.",
        2003 => "Vlož diakritiku až za príslušné základné písmeno.",
        2004 => "Odstráň nepovolený znak alebo ho uzavri do textového literálu.",
        2005 => "Odstráň skrytý znak Unicode; RTL nastav v editore.",
        2006 => "Použi platný desiatkový literál, napríklad 12, 12.5 alebo 1e-3; oddeľ identifikátor medzerou.",
        2007 => "Použi v jednom identifikátore iba jedno z písiem latinka, gréčtina alebo cyrilika; skontroluj znaky, ktoré vyzerajú rovnako.",
        2008 => "Zjednoť identifikátor do Unicode NFC; neupravuj textové literály ani komentáre.",
        _ => "Oprav uvedený zdrojový úsek.",
    };
    Diagnostic {
        code,
        line,
        column,
        detail: detail.into(),
        repair,
    }
}

pub fn is_bidi_control(c: char) -> bool {
    matches!(c as u32, 0x061C | 0x200E | 0x200F |
        0x202A..=0x202E | 0x2066..=0x2069)
}

fn next_char(source: &str, at: usize) -> Option<char> {
    source.get(at..)?.chars().next()
}

fn is_combining_mark(c: char) -> bool {
    // Common combining marks including Tamil syllable signs. Full Unicode normalization
    // and grapheme boundary handling remain tracked in the roadmap.
    matches!(c as u32,
        0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05BD |
        0x05BF | 0x05C1..=0x05C2 | 0x05C4..=0x05C5 |
        0x0610..=0x061A | 0x064B..=0x065F | 0x0670 |
        0x06D6..=0x06ED | 0x0900..=0x0903 | 0x093A..=0x094F |
        0x0951..=0x0957 | 0x0981..=0x0983 | 0x09BC..=0x09CD |
        0x0A3C..=0x0A4D | 0x0ABC..=0x0ACD | 0x0B3C..=0x0B4D | 0x0BBE..=0x0BCD |
        0x0C00..=0x0C04 | 0x0C3E..=0x0C56 | 0x0CBC..=0x0CCD |
        0x0D3B..=0x0D4D | 0x0E31..=0x0E4E)
}

fn is_name_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}
fn is_name_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_combining_mark(c)
}
fn is_lowercase(c: char) -> bool {
    let lowered: String = c.to_lowercase().collect();
    lowered == c.to_string()
}

/// Narrow spoofing guard: Latin, Greek and Cyrillic have many look-alike
/// letters. Do not apply the rule to other script combinations such as
/// Japanese Han+Hiragana, and do not modify original Unicode input.
fn confusable_script(c: char) -> Option<&'static str> {
    match c as u32 {
        0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF
            if c.is_alphabetic() =>
        {
            Some("Latin")
        }
        0x0370..=0x03FF | 0x1F00..=0x1FFF if c.is_alphabetic() => Some("Greek"),
        0x0400..=0x052F | 0x1C80..=0x1C8F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F
            if c.is_alphabetic() =>
        {
            Some("Cyrillic")
        }
        _ => None,
    }
}

/// Return the first mixed-script letter's 1-based scalar offset and the
/// conflicting script names, without inspecting strings or comments.
fn mixed_confusable_script(name: &str) -> Option<(usize, &'static str, &'static str)> {
    let mut first = None;
    for (index, ch) in name.chars().enumerate() {
        if let Some(script) = confusable_script(ch) {
            match first {
                Some(previous) if previous != script => return Some((index + 1, previous, script)),
                None => first = Some(script),
                _ => {}
            }
        }
    }
    None
}

/// Numeric literals are intentionally decimal-only in the current lexer.
/// Reject malformed forms early instead of silently treating them as integers.
fn valid_decimal_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut at = 0;
    let mut digits = 0;
    // Underscores may separate digits but cannot begin/end a group.
    let consume_digits = |at: &mut usize, count: &mut usize| {
        let start = *at;
        while *at < bytes.len() {
            if bytes[*at].is_ascii_digit() {
                *count += 1;
                *at += 1;
            } else if bytes[*at] == b'_'
                && *at > start
                && *at + 1 < bytes.len()
                && bytes[*at - 1].is_ascii_digit()
                && bytes[*at + 1].is_ascii_digit()
            {
                *at += 1;
            } else {
                break;
            }
        }
    };
    consume_digits(&mut at, &mut digits);
    if digits == 0 {
        return false;
    }
    if at < bytes.len() && bytes[at] == b'.' {
        at += 1;
        let mut fractional = 0;
        consume_digits(&mut at, &mut fractional);
        if fractional == 0 {
            return false;
        }
    }
    if at < bytes.len() && matches!(bytes[at], b'e' | b'E') {
        at += 1;
        if at < bytes.len() && matches!(bytes[at], b'+' | b'-') {
            at += 1;
        }
        let mut exponent = 0;
        consume_digits(&mut at, &mut exponent);
        if exponent == 0 {
            return false;
        }
    }
    at == bytes.len()
}

fn advance_position(text: &str, line: &mut usize, col: &mut usize) {
    for c in text.chars() {
        if c == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
    }
}

/// Tokenize a UTF-8 Nobody source. No code execution takes place.
pub fn scan(source: &str) -> Result<Vec<Token>, Diagnostic> {
    // Detect visually deceptive controls even in comments and quoted strings.
    let (mut check_line, mut check_col) = (1, 1);
    for ch in source.chars() {
        if is_bidi_control(ch) {
            return Err(failure(
                2005,
                check_line,
                check_col,
                format!("Skryty Unicode riadiaci znak U+{:04X}", ch as u32),
            ));
        }
        advance_position(&ch.to_string(), &mut check_line, &mut check_col);
    }
    let mut result = Vec::new();
    let (mut pos, mut line, mut col) = (0, 1, 1);
    while pos < source.len() {
        let c = next_char(source, pos).expect("valid byte boundary");
        if c.is_whitespace() {
            let end = pos + c.len_utf8();
            advance_position(&source[pos..end], &mut line, &mut col);
            pos = end;
            continue;
        }
        let (start, tok_line, tok_col) = (pos, line, col);
        let (kind, end) = if c == '#' {
            let end = source[pos..]
                .find('\n')
                .map(|n| pos + n)
                .unwrap_or(source.len());
            (TokenKind::Comment, end)
        } else if c == '"' || c == '\'' {
            let opener = if source[pos..].starts_with(&c.to_string().repeat(3)) {
                3
            } else {
                1
            };
            let mut end = pos + opener;
            let mut closed = false;
            while end < source.len() {
                let t = next_char(source, end).expect("valid boundary");
                if t == '\\' {
                    end += t.len_utf8();
                    if end < source.len() {
                        end += next_char(source, end).unwrap().len_utf8();
                    }
                    continue;
                }
                if opener == 1 && t == '\n' {
                    return Err(failure(
                        2001,
                        tok_line,
                        tok_col,
                        "Textovy literal presahuje koniec riadku",
                    ));
                }
                if source[end..].starts_with(&c.to_string().repeat(opener)) {
                    end += opener;
                    closed = true;
                    break;
                }
                end += t.len_utf8();
            }
            if !closed {
                return Err(failure(
                    2001,
                    tok_line,
                    tok_col,
                    "Chyba uzatvaracia uvodzovka",
                ));
            }
            (TokenKind::String, end)
        } else if is_name_start(c) {
            let mut end = pos + c.len_utf8();
            while let Some(t) = next_char(source, end) {
                if !is_name_continue(t) {
                    break;
                }
                end += t.len_utf8();
            }
            let value = &source[pos..end];
            if value.chars().any(|ch| !is_lowercase(ch)) {
                return Err(failure(
                    2002,
                    tok_line,
                    tok_col,
                    format!("Nazov {:?} obsahuje velke pismena", value),
                ));
            }
            if !is_nfc(value) {
                return Err(failure(
                    2008,
                    tok_line,
                    tok_col,
                    format!("Identifikator {:?} nie je v Unicode NFC", value),
                ));
            }
            if let Some((offset, first, second)) = mixed_confusable_script(value) {
                return Err(failure(
                    2007,
                    tok_line,
                    tok_col + offset - 1,
                    format!(
                        "Identifikator {:?} miesa pismo {} a {}",
                        value, first, second
                    ),
                ));
            }
            (TokenKind::Name, end)
        } else if c.is_ascii_digit() {
            let mut end = pos + 1;
            while let Some(t) = next_char(source, end) {
                if t.is_ascii_alphanumeric()
                    || t == '.'
                    || t == '_'
                    || ((t == '+' || t == '-')
                        && end > pos
                        && matches!(source.as_bytes()[end - 1], b'e' | b'E'))
                {
                    end += t.len_utf8();
                } else {
                    break;
                }
            }
            if !valid_decimal_literal(&source[pos..end]) {
                return Err(failure(
                    2006,
                    tok_line,
                    tok_col,
                    format!("Neplatny ciselny literal {:?}", &source[pos..end]),
                ));
            }
            (TokenKind::Number, end)
        } else if is_combining_mark(c) {
            return Err(failure(
                2003,
                tok_line,
                tok_col,
                format!("Osamotena kombinujuca znacka U+{:04X}", c as u32),
            ));
        } else if !c.is_ascii() {
            return Err(failure(
                2004,
                tok_line,
                tok_col,
                format!("Nepovoleny Unicode znak U+{:04X} mimo textu", c as u32),
            ));
        } else {
            (TokenKind::Symbol, pos + c.len_utf8())
        };
        result.push(Token {
            kind,
            value: source[start..end].into(),
            line: tok_line,
            column: tok_col,
            start,
            end,
        });
        advance_position(&source[pos..end], &mut line, &mut col);
        pos = end;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slovak_and_unicode_position() {
        let tokens = scan("nech n = 1\nvypis(\"Život\")").unwrap();
        assert_eq!(tokens[0].value, "nech");
        assert_eq!(tokens[4].line, 2);
        assert_eq!(tokens[4].column, 1);
    }
    #[test]
    fn native_keywords() {
        for language in [
            "إذا صحيح { اطبع(\"مرحبا\") }",
            "もし 真 { 表示(\"こんにちは\") }",
            "如果 真 { 打印(\"你好\") }",
            "եթե ճշմարիտ { }",
        ] {
            assert!(!scan(language).unwrap().is_empty());
        }
    }
    #[test]
    fn prohibit_visual_override_in_comments() {
        let error = scan("# secret\u{202E}").unwrap_err();
        assert_eq!(error.code, 2005);
    }
    #[test]
    fn unmatched_string_has_number() {
        assert_eq!(scan("vypis(\"missing)").unwrap_err().code, 2001);
    }
    #[test]
    fn uppercase_has_number() {
        assert_eq!(scan("Nech x = 1").unwrap_err().code, 2002);
    }
    #[test]
    fn numeric_literals_accept_decimal_and_exponent() {
        for input in ["0", "123", "1_000", "12.50", "1e3", "1e-3", "1.2E+10"] {
            let tokens = scan(input).unwrap();
            assert_eq!(tokens.len(), 1, "{input}");
            assert_eq!(tokens[0].kind, TokenKind::Number, "{input}");
            assert_eq!(tokens[0].value, input);
        }
    }
    #[test]
    fn malformed_numeric_literals_report_n2006() {
        for input in ["12abc", "1__2", "12_", "1.2.3", "1e", "1e+", "1._2", "0xFF"] {
            let error = scan(input).unwrap_err();
            assert_eq!(error.code, 2006, "{input}");
            assert_eq!((error.line, error.column), (1, 1));
        }
    }
    #[test]
    fn binary_minus_is_not_absorbed_by_number() {
        let tokens = scan("1-2").unwrap();
        assert_eq!(
            tokens.iter().map(|t| t.value.as_str()).collect::<Vec<_>>(),
            vec!["1", "-", "2"]
        );
    }
    #[test]
    fn string_preserved() {
        let t = scan("vypis(\"ak 如果 إذا\")").unwrap();
        assert_eq!(t.iter().filter(|t| t.kind == TokenKind::String).count(), 1);
    }
}
