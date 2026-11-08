use nobody::profiles::PROFILES;
use nobody::{
    check_source, evaluate_source, lower_source, migrate_legacy_ai, parse, parse_validated,
    same_ir, scan, translate_source, TokenKind,
};
use std::{env, fs, process};

fn usage() {
    eprintln!("N9001: Wrong command or arguments. Repair: use one of:\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>\n  nobody-rs parse <profile> <file.nbd>\n  nobody-rs validate <profile> <file.nbd>\n  nobody-rs list-languages\n  nobody-rs migrate-ai <legacy.nbd>\n  nobody-rs translate <from> <to> <file.nbd>\n  nobody-rs ir <profile> <file.nbd>\n  nobody-rs typecheck <profile> <file.nbd>\n  nobody-rs run <profile> <file.nbd>\n  nobody-rs compare-ir <profile-a> <file-a.nbd> <profile-b> <file-b.nbd>");
}
fn main() {
    let arguments: Vec<String> = env::args().collect();
    let valid = match arguments.get(1).map(String::as_str) {
        Some("lex" | "check") if arguments.len() == 3 => true,
        Some("parse" | "validate" | "ir" | "typecheck" | "run") if arguments.len() == 4 => true,
        Some("compare-ir") if arguments.len() == 6 => true,
        Some("list-languages") if arguments.len() == 2 => true,
        Some("migrate-ai") if arguments.len() == 3 => true,
        Some("translate") if arguments.len() == 5 => true,
        _ => false,
    };
    if !valid {
        usage();
        process::exit(2);
    }
    let command = arguments[1].as_str();
    if command == "list-languages" {
        for profile in PROFILES {
            println!("{}", profile.code);
        }
        return;
    }
    if command == "compare-ir" {
        let read = |path: &str| {
            fs::read_to_string(path).unwrap_or_else(|error| {
                eprintln!(
                    "N9001: Cannot read {:?}: {}. Repair: verify path, permissions and UTF-8.",
                    path, error
                );
                process::exit(1);
            })
        };
        let left = read(&arguments[3]);
        let right = read(&arguments[5]);
        match same_ir(&left, &arguments[2], &right, &arguments[4]) {
            Ok(true) => println!(
                "MATCH: identical experimental nobody.ir.v1 structure (not a runtime proof)"
            ),
            Ok(false) => {
                println!("DIFFERENT: canonical IR trees do not match");
                process::exit(1);
            }
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    let filename = arguments.last().expect("validated CLI args");
    let source = match fs::read_to_string(filename) {
        Ok(x) => x,
        Err(e) => {
            eprintln!(
                "N9001: Cannot read {:?}: {}. Repair: verify file path, permissions and UTF-8.",
                filename, e
            );
            process::exit(1);
        }
    };
    if command == "translate" {
        match translate_source(&source, &arguments[2], &arguments[3]) {
            Ok(result) => print!("{result}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "migrate-ai" {
        match migrate_legacy_ai(&source) {
            Ok(migrated) => print!("{migrated}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "run" {
        match evaluate_source(&source, &arguments[2]) {
            Ok(output) => print!("{output}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "typecheck" {
        match check_source(&source,&arguments[2]) {
            Ok(())=>println!("Conservative Rust type check passed (known types only; unknown expressions deferred)."),
            Err(error)=>{eprintln!("{error}");process::exit(1);}
        }
        return;
    }
    if command == "ir" {
        match lower_source(&source, &arguments[2]) {
            Ok(module) => println!("{module:#?}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "parse" || command == "validate" {
        let parsed = if command == "validate" {
            parse_validated(&source, &arguments[2])
        } else {
            parse(&source, &arguments[2])
        };
        match parsed {
            Ok(tree) => {
                if command == "validate" {
                    println!("AST context validation passed; {} top-level statements. Types and runtime NOT checked.",tree.statements.len());
                } else {
                    println!("{:#?}", tree);
                }
            }
            Err(error) => {
                eprintln!("{}", error);
                process::exit(1);
            }
        }
        return;
    }
    match scan(&source) {
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
        Ok(tokens) => {
            if command == "check" {
                println!(
                    "Lexical scan passed ({} tokens). Parse, types and execution NOT checked.",
                    tokens.len()
                );
            } else {
                for token in tokens {
                    let kind = match token.kind {
                        TokenKind::Name => "name",
                        TokenKind::Number => "number",
                        TokenKind::String => "string",
                        TokenKind::Comment => "comment",
                        TokenKind::Symbol => "symbol",
                    };
                    println!(
                        "{}:{} {:<7} {:?}",
                        token.line, token.column, kind, token.value
                    );
                }
            }
        }
    }
}
