//! Preliminary Nobody-to-Nobody keyword translator for all 40 Rust profiles.
//! Does not generate Python. A full syntax-aware translator is still planned.
use crate::{scan,Diagnostic,TokenKind};
use crate::profiles;

fn unsupported(code:&str)->Diagnostic{
    Diagnostic{
        code:3007,line:1,column:1,
        detail:format!("Unknown Nobody syntax profile {code:?}"),
        repair:"Choose one of the 39 supported human profiles or ai.",
    }
}

/// Translate recognized source-profile keyword tokens without touching strings
/// or comments. This does not rename user identifiers or analyze scope.
pub fn translate_source(source:&str,from:&str,to:&str)->Result<String,Diagnostic>{
    if profiles::profile(from).is_none(){return Err(unsupported(from));}
    if profiles::profile(to).is_none(){return Err(unsupported(to));}
    let tokens=scan(source)?;
    let mut output=String::with_capacity(source.len());
    let mut cursor=0;
    for token in tokens {
        output.push_str(&source[cursor..token.start]);
        let replaced=if token.kind==TokenKind::Name {
            let semantic=profiles::semantic(from,&token.value)
                .or_else(|| if from=="sk" && token.value=="kym" {Some("while")}
                    else if from=="en" && token.value=="def" {Some("def")}
                    else {None});
            semantic.and_then(|term|profiles::word(to,term))
        }else{None};
        match replaced{
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
}
