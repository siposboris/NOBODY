use std::{env,fs,process};
use nobody::{scan,TokenKind};

fn usage() {
    eprintln!("Nobody Rust frontend (experimental)\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>");
}
fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 3 || !matches!(arguments[1].as_str(),"lex"|"check") {
        usage();
        process::exit(2);
    }
    let source=match fs::read_to_string(&arguments[2]) {
        Ok(x)=>x,
        Err(e)=>{
            eprintln!("N9001: Neda sa precitat {:?}: {}. Oprava: skontroluj existenciu suboru a UTF-8.", arguments[2],e);
            process::exit(1);
        }
    };
    match scan(&source) {
        Err(d)=>{eprintln!("{}",d);process::exit(1);}
        Ok(tokens)=>{
            if arguments[1]=="check" {
                println!("Lexer OK: {} tokenov; parser a semantika este nie su portovane do Rustu.", tokens.len());
            } else {
                for token in tokens {
                    let kind=match token.kind {
                        TokenKind::Name=>"name",TokenKind::Number=>"number",
                        TokenKind::String=>"string",TokenKind::Comment=>"comment",
                        TokenKind::Symbol=>"symbol",
                    };
                    println!("{}:{} {:<7} {:?}",token.line,token.column,kind,token.value);
                }
            }
        }
    }
}
