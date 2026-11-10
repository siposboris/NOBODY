//! Cross-backend, cross-language regression for the experimental Unicode
//! length builtin. This is a subset-equivalence test, not full language parity.
use nobody::bytecode::{compile_source, execute};
use nobody::bytecode_format::{decode, encode};
use nobody::profiles::{word, PROFILES};
use nobody::{check_source, evaluate_source, lower_source};

fn kw(profile: &str, semantic: &str) -> &'static str {
    word(profile, semantic).expect("registered profile keyword exists")
}

fn source(profile: &str) -> String {
    format!(
        "{function} count_scalar(text_value: {str_type}): {int_type} {{
  {return_word} {len_word}(text_value)
}}
{let_word} greeting = \"Život ☸\"
{let_word} joined = greeting + \"✨\"
{print_word}(count_scalar(joined), {len_word}(\"👩‍💻\"), {len_word}(\"\"))
",
        function = kw(profile, "def"),
        str_type = kw(profile, "str"),
        int_type = kw(profile, "int"),
        return_word = kw(profile, "return"),
        len_word = kw(profile, "len"),
        let_word = kw(profile, "let"),
        print_word = kw(profile, "print"),
    )
}

#[test]
fn all_40_profiles_have_identical_unicode_length_ast_vm_and_portable_bytecode() {
    assert_eq!(PROFILES.len(), 40, "update acceptance scope if profiles change");
    let baseline = source("ai");
    let baseline_ir = lower_source(&baseline, "ai").unwrap();
    let baseline_program = compile_source(&baseline, "ai").unwrap();
    let baseline_wire = encode(&baseline_program).unwrap();
    assert_eq!(u16::from_le_bytes([baseline_wire[8], baseline_wire[9]]), 3);
    let expected = b"8 3 0\n";

    for profile in PROFILES {
        let example = source(profile.code);
        assert_eq!(
            lower_source(&example, profile.code).unwrap(),
            baseline_ir,
            "{}: localized source lowered differently",
            profile.code
        );
        check_source(&example, profile.code)
            .unwrap_or_else(|err| panic!("{}: incorrect static typing: {err}", profile.code));
        let ast = evaluate_source(&example, profile.code).unwrap();
        assert_eq!(ast.as_bytes(), expected, "{}: AST result", profile.code);
        let vm = compile_source(&example, profile.code).unwrap();
        assert_eq!(
            execute(&vm).unwrap(),
            ast,
            "{}: in-memory VM parity",
            profile.code
        );
        let wire = encode(&vm).unwrap();
        assert_eq!(
            wire, baseline_wire,
            "{}: portable bytecode differs despite same canonical IR",
            profile.code
        );
        let decoded = decode(&wire).unwrap();
        assert_eq!(execute(&decoded).unwrap(), ast, "{}: stored v3 parity", profile.code);
        assert_eq!(encode(&decoded).unwrap(), wire, "{}: bytecode stability", profile.code);
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn all_40_profiles_unicode_length_matches_real_standalone_elf() {
    use nobody::native_x86_64::compile_native;
    use std::process::Command;
    let root = std::env::temp_dir().join(format!("nobody-length-matrix-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for profile in PROFILES {
        let example = source(profile.code);
        let target = root.join(format!("compiled-{}", profile.code));
        compile_native(&example, profile.code, &target)
            .unwrap_or_else(|err| panic!("{}: ELF build failed: {err}", profile.code));
        let data = std::fs::read(&target).unwrap();
        assert!(data.starts_with(b"\x7fELF"), "{}: not a Linux ELF", profile.code);
        let output = Command::new(&target)
            .output()
            .unwrap_or_else(|err| panic!("{}: ELF did not start: {err}", profile.code));
        assert!(output.status.success(), "{}: ELF exit {:?}", profile.code, output.status);
        assert_eq!(
            output.stdout,
            evaluate_source(&example, profile.code).unwrap().into_bytes(),
            "{}: native output differs from Rust interpreter",
            profile.code
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
