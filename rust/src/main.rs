use nobody::catalog;
use nobody::diagnostics;
use nobody::evaluator::evaluate_source_with_max_steps;
use nobody::profiles::{audit_profile_issues, direction, AuditSeverity, PROFILES};
use nobody::{
    check_source_with_return_types, compile_bytecode_source, emit_ast_json_v1, emit_ast_json_v2,
    emit_program, evaluate_source, execute_bytecode_source, export_ast_json, export_ast_json_v2,
    import_ast_json_v1, import_ast_json_v2, lower_source, migrate_legacy_ai, normalize_ast_json_v2,
    parse, parse_validated, same_ast_json_v2, same_ir, scan, translate_source, TokenKind,
};
use std::io::{Read, Write};
use std::{env, fs, process};

fn usage() {
    eprintln!("N9001: Wrong command or arguments. Repair: use one of:\n  nobody-rs lex <file.nbd>\n  nobody-rs check <file.nbd>\n  nobody-rs parse <profile> <file.nbd>\n  nobody-rs validate <profile> <file.nbd>\n  nobody-rs list-languages\n  nobody-rs catalog [--json]\n  nobody-rs errors [number]\n  nobody-rs direction <profile>\n  nobody-rs ast-json <profile> <file.nbd>\n  nobody-rs ast-emit <file.json> --lang <profile>\n  nobody-rs ast-json-v2 <profile> <file.nbd>\n  nobody-rs verify-ast-json-v2 <file.json>\n  nobody-rs normalize-ast-json-v2 <file.json>\n  nobody-rs compare-ast-v2 <profile> <file.nbd> <file.json>\n  nobody-rs emit-ast-v2 <profile> <file.json>\n  nobody-rs format-source <profile> <file.nbd>\n  nobody-rs format-check <profile> <file.nbd>\n  nobody-rs bytecode <profile> <file.nbd>\n  nobody-rs run-bytecode <profile> <file.nbd>\n  nobody-rs compile-bytecode <profile> <file.nbd> <output.nbc>\n  nobody-rs verify-bytecode <file.nbc>\n  nobody-rs inspect-bytecode <file.nbc>\n  nobody-rs run-compiled <file.nbc>\n  nobody-rs native-asm <profile> <file.nbd>\n  nobody-rs compile-native <profile> <file.nbd> <output-elf>\n  nobody-rs check-project <manifest.nbproj>\n  nobody-rs run-project <manifest.nbproj>\n  nobody-rs compile-project-bytecode <manifest.nbproj> <output.nbc>\n  nobody-rs compile-project-native <manifest.nbproj> <output-elf>\n  nobody-rs audit-languages\n  nobody-rs migrate-ai <legacy.nbd>\n  nobody-rs translate <from> <to> <file.nbd>\n  nobody-rs ir <profile> <file.nbd>\n  nobody-rs typecheck <profile> <file.nbd>\n  nobody-rs run <profile> <file.nbd>\n  nobody-rs compare-ir <profile-a> <file-a.nbd> <profile-b> <file-b.nbd>");
}
// Accept the legacy Python frontend's translation flags while keeping the
// native Rust positional CLI as the canonical interface.
fn normalize_translate_args(arguments: Vec<String>) -> Vec<String> {
    if arguments.get(1).map(String::as_str) != Some("translate") {
        return arguments;
    }
    // Python argparse accepts these options in either order, before or
    // after the source file. Preserve that behavior without invoking Python.
    let mut source = None;
    let mut from = None;
    let mut to = None;
    let mut index = 2;
    while index < arguments.len() {
        // argparse also accepts --option=value for every long option.
        if let Some(value) = arguments[index].strip_prefix("--from-lang=") {
            if from.is_some() {
                return arguments;
            }
            from = Some(value.to_owned());
            index += 1;
            continue;
        }
        if let Some(value) = arguments[index].strip_prefix("--to=") {
            if to.is_some() {
                return arguments;
            }
            to = Some(value.to_owned());
            index += 1;
            continue;
        }
        match arguments[index].as_str() {
            "--from-lang" if from.is_none() && index + 1 < arguments.len() => {
                from = Some(arguments[index + 1].clone());
                index += 2;
            }
            "--to" if to.is_none() && index + 1 < arguments.len() => {
                to = Some(arguments[index + 1].clone());
                index += 2;
            }
            value if !value.starts_with('-') && source.is_none() => {
                source = Some(value.to_owned());
                index += 1;
            }
            _ => return arguments,
        }
    }
    match (source, from, to) {
        (Some(source), Some(from), Some(to)) => {
            vec![
                arguments[0].clone(),
                "translate".to_owned(),
                from,
                to,
                source,
            ]
        }
        _ => arguments,
    }
}

// Preserve the old Python frontend's file-first --lang interface for commands
// already implemented natively in Rust. Execution stays inside the Rust CLI.
fn normalize_legacy_profile_args(arguments: Vec<String>) -> Vec<String> {
    if arguments.len() < 2 {
        return arguments;
    }
    let command = match arguments[1].as_str() {
        "ast-emit" => "ast-emit",
        "type-check" => "typecheck",
        "ast-run" => "run",
        "parse" => "parse",
        "ast-json" => "ast-json",
        _ => return arguments,
    };
    // argparse accepts --lang before or after the file and --max-steps
    // in either position. Normalize all supported legacy spellings to
    // the native Rust CLI; never delegate execution to Python.
    let mut profile = None;
    let mut file = None;
    let mut max_steps = None;
    let mut index = 2;
    while index < arguments.len() {
        if let Some(value) = arguments[index].strip_prefix("--lang=") {
            if profile.is_some() {
                return arguments;
            }
            profile = Some(value.to_owned());
            index += 1;
            continue;
        }
        if let Some(value) = arguments[index].strip_prefix("--max-steps=") {
            if command != "run" || max_steps.is_some() {
                return arguments;
            }
            max_steps = Some(value.to_owned());
            index += 1;
            continue;
        }
        match arguments[index].as_str() {
            "--lang" if profile.is_none() && index + 1 < arguments.len() => {
                profile = Some(arguments[index + 1].clone());
                index += 2;
            }
            "--max-steps"
                if command == "run" && max_steps.is_none() && index + 1 < arguments.len() =>
            {
                max_steps = Some(arguments[index + 1].clone());
                index += 2;
            }
            arg if !arg.starts_with('-') && file.is_none() => {
                file = Some(arg.to_owned());
                index += 1;
            }
            _ => return arguments,
        }
    }
    match (profile, file) {
        (Some(profile), Some(file)) => {
            if command == "ast-emit" {
                return vec![
                    arguments[0].clone(),
                    command.to_owned(),
                    file,
                    "--lang".to_owned(),
                    profile,
                ];
            }
            let mut normalized = vec![arguments[0].clone(), command.to_owned(), profile, file];
            if command == "run" {
                // Python ast-run defaults to 10_000 steps. Preserve that
                // legacy contract without changing the native Rust run default.
                normalized.push("--max-steps".to_owned());
                normalized.push(max_steps.unwrap_or_else(|| "10000".to_owned()));
            }
            normalized
        }
        _ => arguments,
    }
}

fn main() {
    let raw_arguments: Vec<String> = env::args().collect();
    // The Python frontend used argparse: both global and subcommand help
    // exit successfully. Keep that production CLI contract in native Rust.
    if raw_arguments.len() >= 2
        && (raw_arguments[1] == "--help"
            || raw_arguments[1] == "-h"
            || (raw_arguments.len() == 3
                && (raw_arguments[2] == "--help" || raw_arguments[2] == "-h")))
    {
        usage();
        return;
    }
    let arguments = normalize_legacy_profile_args(normalize_translate_args(raw_arguments));
    let valid = match arguments.get(1).map(String::as_str) {
        Some(
            "lex"
            | "check"
            | "direction"
            | "verify-bytecode"
            | "inspect-bytecode"
            | "run-compiled"
            | "verify-ast-json-v2"
            | "normalize-ast-json-v2"
            | "ast-validate",
        ) if arguments.len() == 3 => true,
        Some("compile-bytecode" | "compile-native" | "compare-ast-v2") if arguments.len() == 5 => {
            true
        }
        Some(
            "parse" | "validate" | "ir" | "typecheck" | "run" | "ast-json" | "ast-json-v2"
            | "bytecode" | "native-asm" | "run-bytecode" | "emit-ast-v2" | "format-source"
            | "format-check",
        ) if arguments.len() == 4 => true,
        Some("run")
            if arguments.len() == 6
                && arguments[4] == "--max-steps"
                && arguments[5].parse::<usize>().is_ok() =>
        {
            true
        }
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
        Some("ast-emit") if arguments.len() == 5 && arguments[3] == "--lang" => true,
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
        // Preserve the legacy frontend info fields for existing CLI consumers.
        println!(r#"  "frontend_version": "0.20-experimental","#);
        println!(r#"  "reference_entries": 249,"#);
        println!(r#"  "syntax_profiles": {},"#, PROFILES.len());
        println!(r#"  "native_compiler": true,"#);
        println!(r#"  "runtime": "Rust","#);
        println!(r#"  "all_249_syntax_implemented": false,"#);
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
        let issues = audit_profile_issues();
        let errors = issues
            .iter()
            .filter(|item| item.severity == AuditSeverity::Error)
            .count();
        let warnings = issues
            .iter()
            .filter(|item| item.severity == AuditSeverity::Warning)
            .count();
        let notes = issues
            .iter()
            .filter(|item| item.severity == AuditSeverity::Note)
            .count();
        println!(
            "{{\n  \"profiles\": {},\n  \"errors\": {},\n  \"warnings\": {},\n  \"notes\": {}\n}}",
            PROFILES.len(),
            errors,
            warnings,
            notes
        );
        for issue in &issues {
            eprintln!("{}: {:?}: {}", issue.language, issue.severity, issue.detail);
        }
        if errors > 0 {
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
    if command == "ast-emit" {
        let path = &arguments[2];
        let source = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot read AST JSON {path:?}: {error}");
            process::exit(1);
        });
        match emit_ast_json_v1(&source, &arguments[4]) {
            Ok(emitted) => print!("{emitted}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if command == "ast-validate" {
        let path = &arguments[2];
        let source = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("N9001: Cannot read AST JSON {path:?}: {error}");
            process::exit(1);
        });
        match import_ast_json_v1(&source) {
            Ok(_) => println!("Valid nobody.ast.v1"),
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
    let filename = if command == "run" && arguments.len() == 6 {
        &arguments[3]
    } else {
        arguments.last().expect("validated CLI args")
    };
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
        let result = if arguments.len() == 6 {
            evaluate_source_with_max_steps(
                &source,
                &arguments[2],
                arguments[5].parse().expect("validated step limit"),
            )
        } else {
            evaluate_source(&source, &arguments[2])
        };
        match result {
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
        match check_source_with_return_types(&source,&arguments[2]) {
            Ok(_returns)=>println!("Conservative Rust type check passed (known types only; unknown expressions deferred)."),
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

// Legacy Python argparse accepted options in either order. The Rust CLI
// must preserve those invocation shapes without spawning a Python runtime.
#[cfg(test)]
mod legacy_cli_migration_tests {
    use super::normalize_legacy_profile_args;

    fn normalize(args: &[&str]) -> Vec<String> {
        normalize_legacy_profile_args(args.iter().map(|arg| (*arg).to_owned()).collect())
    }

    #[test]
    fn accepts_profile_before_or_after_source() {
        for command in ["parse", "ast-json", "type-check", "ast-run"] {
            let native = match command {
                "type-check" => "typecheck",
                "ast-run" => "run",
                other => other,
            };
            let mut expected = vec!["nobody-rs", native, "sk", "hello.nbd"];
            if command == "ast-run" {
                expected.extend(["--max-steps", "10000"]);
            }
            assert_eq!(
                normalize(&["nobody-rs", command, "hello.nbd", "--lang", "sk"]),
                expected
            );
            assert_eq!(
                normalize(&["nobody-rs", command, "--lang", "sk", "hello.nbd"]),
                expected
            );
        }
    }

    #[test]
    fn accepts_ast_emit_language_flag_before_or_after_json_file() {
        let expected = vec!["nobody-rs", "ast-emit", "tree.json", "--lang", "sk"];
        assert_eq!(
            normalize(&["nobody-rs", "ast-emit", "tree.json", "--lang", "sk"]),
            expected
        );
        assert_eq!(
            normalize(&["nobody-rs", "ast-emit", "--lang", "sk", "tree.json"]),
            expected
        );
    }

    #[test]
    fn accepts_step_limit_in_both_positions() {
        let expected = vec!["nobody-rs", "run", "sk", "hello.nbd", "--max-steps", "42"];
        assert_eq!(
            normalize(&[
                "nobody-rs",
                "ast-run",
                "--max-steps",
                "42",
                "--lang",
                "sk",
                "hello.nbd"
            ]),
            expected
        );
        assert_eq!(
            normalize(&[
                "nobody-rs",
                "ast-run",
                "hello.nbd",
                "--lang",
                "sk",
                "--max-steps",
                "42"
            ]),
            expected
        );
    }

    #[test]
    fn accepts_argparse_equals_syntax_for_legacy_production_commands() {
        for command in ["parse", "ast-json", "type-check", "ast-run", "ast-emit"] {
            let expected = match command {
                "type-check" => vec!["nobody-rs", "typecheck", "sk", "hello.nbd"],
                "ast-run" => vec!["nobody-rs", "run", "sk", "hello.nbd", "--max-steps", "42"],
                "ast-emit" => vec!["nobody-rs", "ast-emit", "hello.nbd", "--lang", "sk"],
                other => vec!["nobody-rs", other, "sk", "hello.nbd"],
            };
            let mut args = vec!["nobody-rs", command, "--lang=sk", "hello.nbd"];
            if command == "ast-run" {
                args.push("--max-steps=42");
            }
            assert_eq!(normalize(&args), expected);
        }
        let input = ["nobody-rs", "parse", "hello.nbd", "--lang=sk", "--lang=en"];
        assert_eq!(normalize(&input), input);
    }

    #[test]
    fn accepts_equals_syntax_for_legacy_translation() {
        let input = [
            "nobody-rs",
            "translate",
            "--to=ai",
            "hello.nbd",
            "--from-lang=sk",
        ];
        let normalized = super::normalize_translate_args(
            input.iter().map(|value| (*value).to_owned()).collect(),
        );
        assert_eq!(
            normalized,
            ["nobody-rs", "translate", "sk", "ai", "hello.nbd"]
        );
    }

    #[test]
    fn rejects_duplicate_or_unrelated_flags_without_rewriting() {
        let invalid = [
            "nobody-rs",
            "parse",
            "hello.nbd",
            "--lang",
            "sk",
            "--lang",
            "en",
        ];
        assert_eq!(normalize(&invalid), invalid);
        let invalid = [
            "nobody-rs",
            "parse",
            "hello.nbd",
            "--lang",
            "sk",
            "--max-steps",
            "3",
        ];
        assert_eq!(normalize(&invalid), invalid);
    }
}
