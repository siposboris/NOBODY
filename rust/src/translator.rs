//! Nobody-to-Nobody keyword translator for all 40 Rust profiles.
//! No Python output, no execution. Refuses target-profile keyword collisions.
use crate::{scan,Diagnostic,TokenKind};
use crate::profiles;

fn unsupported(code:&str)->Diagnostic{
    Diagnostic{
        code:3007,line:1,column:1,
        detail:format!("Unknown Nobody syntax profile {code:?}"),
        repair:"Choose one of the 39 supported human profiles or ai.",
    }
}
fn collision(line:usize,column:usize,identifier:&str,target:&str)->Diagnostic{
    Diagnostic {
        code:3011,line,column,
        detail:format!("Identifier {identifier:?} is a reserved keyword in target profile {target:?}; the generated program would change meaning"),
        repair:"Rename the source variable/function/parameter to a name that is not reserved in the target profile, then translate again.",
    }
}
fn canonical(from:&str,value:&str)->Option<&'static str>{
    profiles::semantic(from,value).or_else(||
        if from=="sk" && value=="kym" {Some("while")}
        else if from=="en" && value=="def" {Some("def")}
        else {None})
}

/// Convert known source keywords to a target language, without rewriting
/// strings/comments. Reject any non-keyword Name that becomes a target keyword.
/// This is not yet a fully scope-aware, grammar-verified source translator.
pub fn translate_source(source:&str,from:&str,to:&str)->Result<String,Diagnostic>{
    if profiles::profile(from).is_none(){return Err(unsupported(from));}
    if profiles::profile(to).is_none(){return Err(unsupported(to));}
    if from==to{return Ok(source.to_string());}
    let tokens=scan(source)?;
    let mut output=String::with_capacity(source.len());
    let mut cursor=0;
    for token in tokens {
        output.push_str(&source[cursor..token.start]);
        let translated=if token.kind==TokenKind::Name {
            if let Some(term)=canonical(from,&token.value){
                profiles::word(to,term)
            } else {
                // Any unrecognized source name might be an identifier.
                // Prefer a precise diagnostic over silently corrupting it.
                if profiles::is_reserved(to,&token.value) ||
                   (to=="en" && token.value=="def") ||
                   (to=="sk" && token.value=="kym") {
                    return Err(collision(token.line,token.column,&token.value,to));
                }
                None
            }
        }else{None};
        match translated {
            Some(spelling)=>output.push_str(spelling),
            None=>output.push_str(&source[token.start..token.end]),
        }
        cursor=token.end;
    }
    output.push_str(&source[cursor..]);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    #[test]
    fn slovak_to_ai_is_readable_and_parseable(){
        let sk="funkcia dvoj(x) { vrat x * 2 }\nnech n: cele = 3\nak pravda { vypis(dvoj(n)) }";
        let ai=translate_source(sk,"sk","ai").unwrap();
        assert_eq!(ai,"fn dvoj(x) { return x * 2 }\nlet n: int = 3\nif true { print(dvoj(n)) }");
        parse(&ai,"ai").unwrap();
    }
    #[test]
    fn preserves_quoted_text_comments_and_independent_identifiers(){
        let sk="nech ai_hello = \"ak pravda vypis\" # funkcia vrat\nvypis(ai_hello)";
        let ai=translate_source(sk,"sk","ai").unwrap();
        assert_eq!(ai,"let ai_hello = \"ak pravda vypis\" # funkcia vrat\nprint(ai_hello)");
    }
    #[test]
    fn native_keyword_roundtrip(){
        let sk="nech n: cele = 3\nak pravda { vypis(n) }";
        for lang in ["ar","he","zh","ja","ru","ta","gu","kn","te","ai"]{
            let target=translate_source(sk,"sk",lang).unwrap();
            let roundtrip=translate_source(&target,lang,"sk").unwrap();
            assert_eq!(roundtrip,sk,"profile: {lang}");
        }
    }
    #[test]
    fn unknown_profile_is_precise_n3007(){
        let err=translate_source("nech x = 1","sk","zz").unwrap_err();
        assert_eq!(err.code,3007);
    }
    #[test]
    fn identifier_collision_is_n3011(){
        let err=translate_source("nech fn = 3\nvypis(fn)","sk","ai").unwrap_err();
        assert_eq!(err.code,3011);
        assert_eq!(err.line,1);
        assert_eq!(err.column,6);
        assert!(err.detail.contains("\"fn\""));
        assert!(err.repair.contains("Rename"));
    }
    #[test]
    fn comment_or_string_with_reserved_word_is_safe(){
        let sk="nech meno = \"fn if let\" # fn if let\nvypis(meno)";
        let ai=translate_source(sk,"sk","ai").unwrap();
        assert_eq!(ai,"let meno = \"fn if let\" # fn if let\nprint(meno)");
    }
    #[test]
    fn same_profile_is_exactly_identity(){
        let source="nech n = 3 # fn and let";
        assert_eq!(translate_source(source,"sk","sk").unwrap(),source);
    }
}
