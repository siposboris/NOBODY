use std::{env,fs,process};
use nobody::{parse,scan,TokenKind};

fn usage() {
    eprintln!("N9001: Wrong command or arguments. Repair: use one of:\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>\n  nobody-rs parse <sk|en|ai> <file.nbd>");
}
fn main() {
    let arguments: Vec<String> = env::args().collect();
    let valid = match arguments.get(1).map(String::as_str) {
        Some("lex"|"check") if arguments.len()==3 => true,
        Some("parse") if arguments.len()==4 => true,
        _ => false
    };
    if !valid {usage();process::exit(2);}
    let command=arguments[1].as_str();
    let filename=arguments.last().expect("validated CLI args");
    let source=match fs::read_to_string(filename) {
        Ok(x)=>x,
        Err(e)=>{
            eprintln!("N9001: Cannot read {:?}: {}. Repair: verify file path, permissions and UTF-8.",filename,e);
            process::exit(1);
        }
    };
    if command=="parse" {
        match parse(&source,&arguments[2]) {
            Ok(tree)=>println!("{:#?}",tree),
            Err(error)=>{eprintln!("{}",error);process::exit(1);}
        }
        return;
    }
    match scan(&source) {
        Err(error)=>{eprintln!("{}",error);process::exit(1);}
        Ok(tokens)=>{
            if command=="check" {
                println!("Lexical scan passed ({} tokens). Parse, types and execution NOT checked.",tokens.len());
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
