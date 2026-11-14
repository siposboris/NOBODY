use nobody::catalog;
use nobody::diagnostics;
use nobody::profiles::{audit_profiles, direction, PROFILES};
use nobody::{
    check_source, compile_bytecode_source, emit_ast_json_v2, emit_program, evaluate_source,
    execute_bytecode_source, export_ast_json, export_ast_json_v2, import_ast_json_v2, lower_source,
    migrate_legacy_ai, normalize_ast_json_v2, parse, parse_validated, same_ast_json_v2, same_ir,
    scan, translate_source, TokenKind,
};
use std::io::{Read, Write};
use std::{env, fs, process};

fn usage() {
    eprintln!("N9001: Wrong command or arguments. Repair: use one of:\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>\n  nobody-rs parse <profile> <file.nbd>\n  nobody-rs validate <profile> <file.nbd>\n  nobody-rs list-languages\n  nobody-rs catalog [--json]\n  nobody-rs errors [number]\n  nobody-rs direction <profile>\n  nobody-rs ast-json <profile> <file.nbd>\n  nobody-rs ast-json-v2 <profile> <file.nbd>\n  nobody-rs verify-ast-json-v2 <file.json>\n  nobody-rs normalize-ast-json-v2 <file.json>\n  nobody-rs compare-ast-v2 <profile> <file.nbd> <file.json>\n  nobody-rs emit-ast-v2 <profile> <file.json>\n  nobody-rs format-source <profile> <file.nbd>\n  nobody-rs format-check <profile> <file.nbd>\n  nobody-rs bytecode <profile> <file.nbd>\n  nobody-rs run-bytecode <profile> <file.nbd>\n  nobody-rs compile-bytecode <profile> <file.nbd> <output.nbc>\n  nobody-rs verify-bytecode <file.nbc>\n  nobody-rs inspect-bytecode <file.nbc>\n  nobody-rs run-compiled <file.nbc>\n  nobody-rs native-asm <profile> <file.nbd>\n  nobody-rs compile-native <profile> <file.nbd> <output-elf>\n  nobody-rs check-project <manifest.nbproj>\n  nobody-rs run-project <manifest.nbproj>\n  nobody-rs compile-project-bytecode <manifest.nbproj> <output.nbc>\n  nobody-rs compile-project-native <manifest.nbproj> <output-elf>\n  nobody-rs audit-languages\n  nobody-rs migrate-ai <legacy.nbd>\n  nobody-rs translate <from> <to> <file.nbd>\n  nobody-rs ir <profile> <file.nbd>\n  nobody-rs typecheck <profile> <file.nbd>\n  nobody-rs run <profile> <file.nbd>\n  nobody-rs compare-ir <profile-a> <file-a.nbd> <profile-b> <file-b.nbd>");
}
fn main() {
    let arguments: Vec<String> = env::args().collect();
    let valid = match arguments.get(1).map(String::as_str) {
        Some(
            "lex"
            | "check"
            | "direction"
            | "verify-bytecode"
            | "inspect-bytecode"
            | "run-compiled"
            | "verify-ast-json-v2"
            | "normalize-ast-json-v2",
        ) if arguments.len() == 3 => true,
        Some("compile-bytecode" | "compile-native" | "compare-ast-v2") if arguments.len() == 5 => {
            true
        }
        Some(
            "parse" | "validate" | "ir" | "typecheck" | "run" | "ast-json" | "ast-json-v2"
            | "bytecode" | "native-asm" | "run-bytecode" | "emit-ast-v2" | "format-source"
            | "format-check",
        ) if arguments.len() == 4 => true,
        Some("compile-project-bytecode" | "compile-project-native") if arguments.len() == 4 => true,
        Some("check-project" | "run-project") if arguments.len() == 3 => true,
        Some("compare-ir") if arguments.len() == 6 => true,
        Some("list-languages" | "languages" | "catalog" | "info" | "audit-languages")
            if arguments.len() == 2 =>
        {
            true
        }
        Some("catalog" | "languages") if arguments.len() == 3 && arguments[2] == "--json" => true,
        Some("errors") if arguments.len() == 3 && arguments[2].parse::<u16>().is_ok() => true,
        Some("errors") if arguments.len() == 2 => true,
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
        println!(r#"  "native_machine_code_compiler": true,"#);
        println!(r#"  "native_target": "linux-x86_64-elf",""#);
        println!(r#"  "native_scope": "restricted experimental subset",""#);
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
    if command == "errors" {
        if arguments.len() == 3 {
            let number = arguments[2].parse::<u16>().expect("validated error number");
            match diagnostics::lookup(number) {
                Some(entry) => println!("{}", diagnostics::format_entry(entry)),
                None => process::exit(1),
            }
        } else {
            let mut entries = diagnostics::DEFINITIONS.to_vec();
            entries.sort_by_key(|entry| entry.number);
            for entry in entries {
                println!("{}", diagnostics::format_entry(&entry));
            }
        }
        return;
    }
    if command == "catalog" || command == "languages" {
        let entries = match catalog::languages() {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("N9002: {error}");
                process::exit(1);
            }
        };
        if arguments.len() == 3 {
            println!("{}", catalog::json(&entries));
        } else {
            for (index, entry) in entries.iter().enumerate() {
                println!(
                    "{:3}. {}: {} {}",
                    index + 1,
                    entry.name,
                    entry.status,
                    entry.syntax_code.unwrap_or("")
                );
            }
        }
        return;
    }
    if command == "list-languages" {
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
    if matches!(
        command,
        "check-project" | "run-project" | "compile-project-bytecode" | "compile-project-native"
    ) {
        let project =
            nobody::load_project(std::path::Path::new(&arguments[2])).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
        if command == "compile-project-native" {
            let target = std::path::Path::new(&arguments[3]);
            nobody::native_x86_64::compile_native(&project.source, "ai", target).unwrap_or_else(
                |error| {
                    eprintln!("{error}");
                    process::exit(1);
                },
            );
            println!(
                "Built native ELF from {} Nobody modules (restricted Linux x86-64 subset)",
                project.modules.len()
            );
            return;
        }
        let compiled = compile_bytecode_source(&project.source, "ai").unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
        if command == "check-project" {
            println!(
                "Validated project graph, import contracts, namespace isolation and supported bytecode for {} Nobody modules (experimental compiler).",
                project.modules.len()
            );
            return;
        }
        if command == "run-project" {
            let output = nobody::bytecode::execute(&compiled).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print!("{output}");
            return;
        }
        let bytes = nobody::bytecode_format::encode(&compiled).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
        // Refuse to replace existing artifacts (even symbolic links).
        let destination = &arguments[3];
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .unwrap_or_else(|error| {
                eprintln!("N9001: Cannot create project output {destination:?}: {error}");
                process::exit(1);
            });
        if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
            drop(file);
            let _ = fs::remove_file(destination);
            eprintln!("N9001: Cannot finish project bytecode output: {error}");
            process::exit(1);
        }
        println!(
            "Built {} bytes of verified portable Nobody bytecode from {} modules.",
            bytes.len(),
            project.modules.len()
        );
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
    if command == "compile-native" {
        let profile = &arguments[2];
        let source_path = &arguments[3];
        let output_path = &arguments[4];
        let source = fs::read_to_string(source_path).unwrap_or_else(|error| {
            eprintln!(
                "N9001: Cannot read Nobody source {:?}: {error}",
                source_path
            );
            process::exit(1);
        });
        match nobody::native_x86_64::compile_native(
            &source,
            profile,
            std::path::Path::new(output_path),
        ) {
            Ok(()) => println!(
                "Created standalone Linux x86-64 ELF executable: {output_path} (experimental i64 subset)"
            ),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "verify-ast-json-v2" || command == "normalize-ast-json-v2" {
        let path = &arguments[2];
        let file = fs::File::open(path).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot open AST JSON {path:?}: {error}");
            process::exit(1);
        });
        let mut input = String::new();
        file.take(1_048_577)
            .read_to_string(&mut input)
            .unwrap_or_else(|error| {
                eprintln!("N9001: Cannot read UTF-8 AST JSON {path:?}: {error}");
                process::exit(1);
            });
        if command == "verify-ast-json-v2" {
            match import_ast_json_v2(&input) {
                Ok(doc) => println!(
                    "Validated nobody.ast.v2: {} nodes (structural only; not executed).",
                    doc.node_count
                ),
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
            }
        } else {
            match normalize_ast_json_v2(&input) {
                Ok(output) => println!("{output}"),
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(1);
                }
            }
        }
        return;
    }
    if command == "emit-ast-v2" {
        let path = &arguments[3];
        let file = fs::File::open(path).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot open AST JSON {path:?}: {error}");
            process::exit(1);
        });
        let mut json = String::new();
        file.take(1_048_577)
            .read_to_string(&mut json)
            .unwrap_or_else(|error| {
                eprintln!("N9001: Cannot read AST JSON {path:?}: {error}");
                process::exit(1);
            });
        match emit_ast_json_v2(&json, &arguments[2]) {
            Ok(source) => print!("{source}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "compare-ast-v2" {
        let profile = &arguments[2];
        let source = fs::read_to_string(&arguments[3]).unwrap_or_else(|error| {
            eprintln!(
                "N9001: Cannot read Nobody source {:?}: {error}",
                arguments[3]
            );
            process::exit(1);
        });
        let file = fs::File::open(&arguments[4]).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot open AST JSON {:?}: {error}", arguments[4]);
            process::exit(1);
        });
        let mut json = String::new();
        file.take(1_048_577)
            .read_to_string(&mut json)
            .unwrap_or_else(|error| {
                eprintln!(
                    "N9001: Cannot read UTF-8 AST JSON {:?}: {error}",
                    arguments[4]
                );
                process::exit(1);
            });
        match same_ast_json_v2(&source, profile, &json) {
            Ok(true) => println!(
                "MATCH: JSON and source lower to the same validated Rust syntax AST (not executed; source lines not compared)."
            ),
            Ok(false) => {
                println!("DIFFERENT: JSON and Nobody source syntax ASTs do not match");
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
    if matches!(
        command,
        "run-compiled" | "verify-bytecode" | "inspect-bytecode"
    ) {
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
        if command == "inspect-bytecode" {
            // Decode validates the wire format before we expose metrics.
            // Never execute the program or print untrusted names/literals.
            let functions_total: usize = program.functions.iter().map(|f| f.code.len()).sum();
            let instructions = program
                .code
                .iter()
                .chain(program.functions.iter().flat_map(|f| f.code.iter()));
            let (mut text_literals, mut scalar_length_ops) = (0usize, 0usize);
            for opcode in instructions {
                match opcode {
                    nobody::bytecode::Instruction::PushText(_) => text_literals += 1,
                    nobody::bytecode::Instruction::TextLen => scalar_length_ops += 1,
                    _ => {}
                }
            }
            println!("format: NOBBC001");
            println!("wire_version: {}", u16::from_le_bytes([data[8], data[9]]));
            println!("file_bytes: {}", data.len());
            println!("main_instructions: {}", program.code.len());
            println!("function_count: {}", program.functions.len());
            println!("function_instructions: {functions_total}");
            println!("text_literals: {text_literals}");
            println!("unicode_length_ops: {scalar_length_ops}");
            println!("verified: true");
            println!("executed: false");
        } else if command == "verify-bytecode" {
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
    if command == "format-source" || command == "format-check" {
        match parse_validated(&source, &arguments[2])
            .and_then(|program| emit_program(&program, &arguments[2]))
        {
            Ok(formatted) if command == "format-check" => {
                if source == formatted {
                    println!("OK: Nobody source matches the canonical Rust formatter.");
                } else {
                    eprintln!("N5002: Nobody source is not canonically formatted. Repair: run 'nobody-rs format-source {} {}' and review the output.", arguments[2], arguments[3]);
                    process::exit(1);
                }
            }
            Ok(formatted) => print!("{formatted}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "ast-json" || command == "ast-json-v2" {
        let export = if command == "ast-json-v2" {
            export_ast_json_v2
        } else {
            export_ast_json
        };
        match export(&source, &arguments[2]) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "native-asm" {
        match nobody::native_x86_64::compile_to_assembly(&source, &arguments[2]) {
            Ok(assembly) => print!("{assembly}"),
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
