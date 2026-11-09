use nobody::profiles::{audit_profiles, direction, PROFILES};
use nobody::{
    check_source, compile_bytecode_source, evaluate_source, execute_bytecode_source,
    export_ast_json, lower_source, migrate_legacy_ai, parse, parse_validated, same_ir, scan,
    translate_source, TokenKind,
};
use std::io::{Read, Write};
use std::{env, fs, process};

fn usage() {
    eprintln!("N9001: Wrong command or arguments. Repair: use one of:\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>\n  nobody-rs parse <profile> <file.nbd>\n  nobody-rs validate <profile> <file.nbd>\n  nobody-rs list-languages\n  nobody-rs direction <profile>\n  nobody-rs ast-json <profile> <file.nbd>\n  nobody-rs bytecode <profile> <file.nbd>\n  nobody-rs run-bytecode <profile> <file.nbd>\n  nobody-rs compile-bytecode <profile> <file.nbd> <output.nbc>\n  nobody-rs verify-bytecode <file.nbc>\n  nobody-rs run-compiled <file.nbc>\n  nobody-rs audit-languages\n  nobody-rs migrate-ai <legacy.nbd>\n  nobody-rs translate <from> <to> <file.nbd>\n  nobody-rs ir <profile> <file.nbd>\n  nobody-rs typecheck <profile> <file.nbd>\n  nobody-rs run <profile> <file.nbd>\n  nobody-rs compare-ir <profile-a> <file-a.nbd> <profile-b> <file-b.nbd>");
}
fn main() {
    let arguments: Vec<String> = env::args().collect();
    let valid = match arguments.get(1).map(String::as_str) {
        Some("lex" | "check" | "direction" | "verify-bytecode" | "run-compiled")
            if arguments.len() == 3 =>
        {
            true
        }
        Some("compile-bytecode") if arguments.len() == 5 => true,
        Some(
            "parse" | "validate" | "ir" | "typecheck" | "run" | "ast-json" | "bytecode"
            | "run-bytecode",
        ) if arguments.len() == 4 => true,
        Some("compare-ir") if arguments.len() == 6 => true,
        Some("list-languages" | "languages" | "info" | "audit-languages")
            if arguments.len() == 2 =>
        {
            true
        }
        Some("migrate-ai") if arguments.len() == 3 => true,
        Some("translate") if arguments.len() == 5 => true,
        _ => false,
    };
    if !valid {
        usage();
        process::exit(2);
    }
    let command = arguments[1].as_str();
    if command == "info" {
        println!("{{");
        println!(r#"  "name": "Nobody","#);
        println!(r#"  "implementation": "Rust","#);
        println!(r#"  "compiler_version": "{}","#, env!("CARGO_PKG_VERSION"));
        println!(r#"  "implemented_syntax_profiles": {},"#, PROFILES.len());
        println!(r#"  "native_machine_code_compiler": false,"#);
        println!(r#"  "self_hosted": false,"#);
        println!(r#"  "python_required_for_rust_cli": false"#);
        println!("}}");
        return;
    }
    if command == "audit-languages" {
        let problems = audit_profiles();
        println!(
            "Nobody Rust profile audit: {} active syntax profiles; {} structural errors",
            PROFILES.len(),
            problems.len()
        );
        for problem in &problems {
            eprintln!("{problem}");
        }
        if !problems.is_empty() {
            process::exit(1);
        }
        return;
    }
    if command == "list-languages" || command == "languages" {
        for profile in PROFILES {
            println!("{}", profile.code);
        }
        return;
    }
    if command == "direction" {
        match direction(&arguments[2]) {
            Some("rtl") => {
                println!("rtl");
                println!("Use a UTF-8 source editor with RTL paragraph direction for Arabic/Urdu/Hebrew/Persian text; preserve logical character order and ASCII operator order. Never insert invisible bidi control characters into a .nbd source file.");
            }
            Some("ltr") => {
                println!("ltr");
                println!("Use UTF-8 and normal left-to-right code layout; retain the selected language native keyword spellings.");
            }
            _ => {
                eprintln!("N3007: Unknown Nobody syntax profile {:?}. Repair: select one of the 40 available syntax profiles.", arguments[2]);
                process::exit(1);
            }
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
    // .nbc never goes through source parsing or UTF-8 decoding.
    if command == "run-compiled" || command == "verify-bytecode" {
        let filename = &arguments[2];
        let file = fs::File::open(filename).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot open {:?}: {error}", filename);
            process::exit(1);
        });
        let mut data = Vec::new();
        let mut capped = file.take((nobody::bytecode_format::MAX_FILE_BYTES + 1) as u64);
        capped.read_to_end(&mut data).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot read bytecode {:?}: {error}", filename);
            process::exit(1);
        });
        let program = nobody::bytecode_format::decode(&data).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
        if command == "verify-bytecode" {
            println!(
                "Verified experimental bytecode: {} main instructions, {} functions (not a security sandbox).",
                program.code.len(),
                program.functions.len()
            );
        } else {
            let output = nobody::bytecode::execute(&program).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print!("{output}");
        }
        return;
    }
    if command == "compile-bytecode" {
        let profile = &arguments[2];
        let source_path = &arguments[3];
        let destination = &arguments[4];
        let source = fs::read_to_string(source_path).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot read source {:?}: {error}", source_path);
            process::exit(1);
        });
        let compiled = compile_bytecode_source(&source, profile).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
        let bytes = nobody::bytecode_format::encode(&compiled).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
        // Refuse to truncate an existing file, including a symbolic link.
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .unwrap_or_else(|error| {
                eprintln!("N9001: Cannot create new bytecode file {:?} (existing files are never overwritten): {error}", destination);
                process::exit(1);
            });
        if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
            drop(file);
            let _ = fs::remove_file(destination);
            eprintln!(
                "N9001: Cannot finish writing bytecode {:?}: {error}",
                destination
            );
            process::exit(1);
        }
        println!(
            "Wrote {} bytes of experimental bytecode to {}",
            bytes.len(),
            destination
        );
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
    if command == "ast-json" {
        match export_ast_json(&source, &arguments[2]) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "bytecode" {
        match compile_bytecode_source(&source, &arguments[2]) {
            Ok(program) => println!("{program:#?}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "run-bytecode" {
        match execute_bytecode_source(&source, &arguments[2]) {
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
