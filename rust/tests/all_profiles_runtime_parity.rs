//! Phase 1/4 cross-profile acceptance: language dictionaries must produce
//! equivalent canonical IR, AST evaluator output and standalone native ELF.
//! This suite is generated from the actual 40 registered Rust profiles.
//! It deliberately avoids hardcoding a partial ai/sk/en-only profile list.
use nobody::profiles::{word, PROFILES};
use nobody::{check_source, evaluate_source, lower_source, parse_validated, same_ir};

fn kw(profile: &str, semantic: &str) -> &'static str {
    word(profile, semantic).expect("all audited profiles implement the canonical word")
}

fn sample(profile: &str) -> String {
    format!(
        "{let_word} counter_value = 2
{let_word} total_value = 0
{for_word} loop_ix {in_word} {range_word}(4) {{
  {if_word} loop_ix >= 2 {{
    total_value = total_value + counter_value
  }} {else_word} {{
    total_value = total_value + loop_ix
  }}
}}
{def_word} scale_value(input_value: {int_word}): {int_word} {{
  {return_word} input_value * 3
}}
{print_word}(scale_value(total_value), {true_word}, \"Život ☸\")
",
        let_word = kw(profile, "let"),
        int_word = kw(profile, "int"),
        for_word = kw(profile, "for"),
        in_word = kw(profile, "in"),
        range_word = kw(profile, "range"),
        if_word = kw(profile, "if"),
        else_word = kw(profile, "else"),
        def_word = kw(profile, "def"),
        return_word = kw(profile, "return"),
        print_word = kw(profile, "print"),
        true_word = kw(profile, "true"),
    )
}

#[test]
fn every_registered_profile_has_matching_runtime_ir_and_typed_parse() {
    assert_eq!(
        PROFILES.len(),
        40,
        "update this test for new official profiles"
    );
    let baseline = sample("ai");
    let expected = "15 true Život ☸\n";
    let baseline_ir = lower_source(&baseline, "ai").unwrap();
    for profile in PROFILES {
        let source = sample(profile.code);
        let ast = parse_validated(&source, profile.code)
            .unwrap_or_else(|err| panic!("{} AST: {err}", profile.code));
        assert!(!ast.statements.is_empty());
        let ir = lower_source(&source, profile.code)
            .unwrap_or_else(|err| panic!("{} IR: {err}", profile.code));
        assert_eq!(ir, baseline_ir, "{}: canonical IR diverged", profile.code);
        assert!(
            same_ir(&baseline, "ai", &source, profile.code).unwrap(),
            "{}: IR compare failed",
            profile.code
        );
        check_source(&source, profile.code)
            .unwrap_or_else(|err| panic!("{} typecheck: {err}", profile.code));
        let output = evaluate_source(&source, profile.code)
            .unwrap_or_else(|err| panic!("{} evaluator: {err}", profile.code));
        assert_eq!(output, expected, "{}: runtime output differs", profile.code);
    }
}

#[test]
fn mixed_profile_keyword_files_do_not_silently_execute() {
    // The keywords for a file are selected by its profile. A second profile
    // must not switch the active keyword dictionary halfway through the file.
    for (profile, source) in [
        (
            "sk",
            "nech total_value = 2\nif pravda { vypis(total_value) }",
        ),
        ("en", "let total_value = 2\nak true { print(total_value) }"),
        ("ai", "let total_value = 2\nak true { print(total_value) }"),
    ] {
        assert!(
            parse_validated(source, profile).is_err(),
            "{profile}: mixed syntax was silently accepted"
        );
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn all_40_profiles_produce_standalone_elf_with_identical_stdout() {
    use nobody::native_x86_64::compile_native;
    use std::process::Command;
    let folder = std::env::temp_dir().join(format!("nobody-all-profiles-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    for profile in PROFILES {
        let source = sample(profile.code);
        let target = folder.join(format!("native-{}", profile.code));
        compile_native(&source, profile.code, &target)
            .unwrap_or_else(|err| panic!("{}: native compile failed: {err}", profile.code));
        let bytes = std::fs::read(&target).unwrap();
        assert!(bytes.starts_with(b"\x7fELF"), "{}: not ELF", profile.code);
        let result = Command::new(&target)
            .output()
            .unwrap_or_else(|err| panic!("{}: ELF did not launch: {err}", profile.code));
        assert!(
            result.status.success(),
            "{}: ELF exit {:?}",
            profile.code,
            result.status
        );
        assert_eq!(
            result.stdout,
            evaluate_source(&source, profile.code).unwrap().as_bytes(),
            "{}: ELF/AST differential output failure",
            profile.code
        );
    }
    std::fs::remove_dir_all(folder).unwrap();
}
