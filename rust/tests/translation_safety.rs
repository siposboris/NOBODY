//! Cross-profile translation safety and stable diagnostic contracts.
use nobody::{parse,translate_source};
use nobody::profiles::{word,PROFILES};

#[test]
fn source_identifier_conflicting_with_ai_keyword_fails(){
    let bad="nech fn = 7\nvypis(fn)";
    let err=translate_source(bad,"sk","ai").unwrap_err();
    assert_eq!(err.code,3011);
    assert_eq!((err.line,err.column),(1,6));
    assert!(err.to_string().contains("Rename"));
}

#[test]
fn collision_is_detected_on_reference_line_too(){
    let bad="nech x = 7\nvypis(fn)";
    let err=translate_source(bad,"sk","ai").unwrap_err();
    assert_eq!(err.code,3011);
    assert_eq!(err.line,2);
}

#[test]
fn source_keywords_convert_and_identifiers_remain_stable(){
    let source="nech cislo: cele = 2\nvypis(cislo)";
    let translated=translate_source(source,"sk","ai").unwrap();
    assert_eq!(translated,"let cislo: int = 2\nprint(cislo)");
    parse(&translated,"ai").unwrap();
}

#[test]
fn quoted_and_commented_target_keywords_do_not_trigger_collision(){
    let source="nech cislo = \"fn let if\" # fn let if\nvypis(cislo)";
    let transformed=translate_source(source,"sk","ai").unwrap();
    assert_eq!(transformed,"let cislo = \"fn let if\" # fn let if\nprint(cislo)");
}

#[test]
fn human_profile_keyword_tables_are_complete_for_translation(){
    for p in PROFILES {
        let decl=word(p.code,"let").unwrap();
        let print=word(p.code,"print").unwrap();
        let source=format!("{decl} ident = 8\n{print}(ident)");
        let ai=translate_source(&source,p.code,"ai")
            .unwrap_or_else(|err|panic!("{}: {}",p.code,err));
        assert_eq!(ai,"let ident = 8\nprint(ident)","{}",p.code);
    }
}
