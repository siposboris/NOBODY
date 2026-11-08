//! Legacy AI prefix migration and syntax-table utilities.
//! Transformation uses token byte spans: never rewrite quoted strings/comments.
use crate::profiles;
use crate::{scan, Diagnostic, TokenKind};

pub fn migrate_legacy_ai(source: &str) -> Result<String, Diagnostic> {
    let tokens = scan(source)?;
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0;
    for token in tokens {
        result.push_str(&source[cursor..token.start]);
        let replacement = if token.kind == TokenKind::Name {
            token.value.strip_prefix("ai_").and_then(|suffix| {
                let semantic = if suffix == "none" { "none" } else { suffix };
                if profiles::CANONICAL.contains(&semantic) {
                    profiles::word("ai", semantic)
                } else {
                    None
                }
            })
        } else {
            None
        };
        match replacement {
            Some(word) => result.push_str(word),
            None => result.push_str(&source[token.start..token.end]),
        }
        cursor = token.end;
    }
    result.push_str(&source[cursor..]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrate_old_keywords() {
        let code = "ai_let x: ai_int = 3\nai_if ai_true { ai_print(x) }";
        let expected = "let x: int = 3\nif true { print(x) }";
        assert_eq!(migrate_legacy_ai(code).unwrap(), expected);
    }
    #[test]
    fn strings_and_comments_are_untouched() {
        let code = "ai_let s = \"ai_if ai_print\" # ai_while\nai_print(s)";
        assert_eq!(
            migrate_legacy_ai(code).unwrap(),
            "let s = \"ai_if ai_print\" # ai_while\nprint(s)"
        );
    }
    #[test]
    fn ordinary_identifiers_are_not_rewritten() {
        assert_eq!(migrate_legacy_ai("ai_hello = 3").unwrap(), "ai_hello = 3");
    }
    #[test]
    fn migration_supports_ai_def_and_literals() {
        let code = "ai_def f() { ai_return ai_none }\nai_let b = ai_false";
        assert_eq!(
            migrate_legacy_ai(code).unwrap(),
            "fn f() { return null }\nlet b = false"
        );
    }
}
