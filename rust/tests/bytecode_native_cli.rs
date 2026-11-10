//! Native Nobody bytecode compiler and VM CLI acceptance tests.
use std::process::Command;

fn launch(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(args)
        .output()
        .expect("launch native nobody")
}

#[test]
fn bytecode_matches_the_existing_ast_runtime_for_three_profiles() {
    for (language, file) in [
        ("ai", "eval_basic_ai.nbd"),
        ("sk", "eval_basic_sk.nbd"),
        ("en", "bytecode_basic_en.nbd"),
        ("ai", "bytecode_control_ai.nbd"),
        ("sk", "bytecode_control_sk.nbd"),
        ("en", "bytecode_control_en.nbd"),
        ("ai", "bytecode_scopes_ai.nbd"),
        ("ai", "bytecode_for_ai.nbd"),
        ("sk", "bytecode_for_sk.nbd"),
        ("en", "bytecode_for_en.nbd"),
        ("ai", "bytecode_for_nested_ai.nbd"),
        ("ai", "bytecode_for_shadow_ai.nbd"),
        ("ai", "bytecode_for_overflow_ai.nbd"),
        ("ai", "bytecode_functions_ai.nbd"),
        ("sk", "bytecode_functions_sk.nbd"),
        ("en", "bytecode_functions_en.nbd"),
        ("ai", "bytecode_function_control_ai.nbd"),
        ("ai", "bytecode_function_null_ai.nbd"),
        ("ai", "native_typed_signatures_ai.nbd"),
        ("sk", "native_typed_signatures_sk.nbd"),
        ("en", "native_typed_signatures_en.nbd"),
    ] {
        let file = format!("{}/examples/{}", env!("CARGO_MANIFEST_DIR"), file);
        let regular = launch(&["run", language, &file]);
        let vm = launch(&["run-bytecode", language, &file]);
        assert!(
            regular.status.success(),
            "{language}: {}",
            String::from_utf8_lossy(&regular.stderr)
        );
        assert!(
            vm.status.success(),
            "{language}: {}",
            String::from_utf8_lossy(&vm.stderr)
        );
        assert_eq!(regular.stdout, vm.stdout, "{language}");
    }
}

#[test]
fn compiler_exposes_a_real_instruction_stream() {
    let file = format!("{}/examples/eval_basic_ai.nbd", env!("CARGO_MANIFEST_DIR"));
    let output = launch(&["bytecode", "ai", &file]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("nobody.bytecode.experimental.v1"));
    assert!(text.contains("Push("));
    assert!(text.contains("Print("));
    assert!(text.contains("Halt"));
}

#[test]
fn unsupported_ast_is_not_silently_interpreted() {
    let file = format!(
        "{}/examples/bytecode_bad_unsupported_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["run-bytecode", "ai", &file]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N4004"));
}

#[test]
fn branching_and_loops_produce_real_jump_instructions() {
    let file = format!(
        "{}/examples/bytecode_control_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["bytecode", "ai", &file]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let instructions = String::from_utf8(output.stdout).unwrap();
    assert!(instructions.contains("JumpIfFalse("));
    assert!(instructions.contains("Jump("));
    assert!(instructions.contains("Lt"));
    assert!(instructions.contains("Eq"));
}

#[test]
fn infinite_loop_terminates_with_step_budget_not_hang() {
    let file = format!(
        "{}/examples/bytecode_infinite_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["run-bytecode", "ai", &file]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N4005"));
}

#[test]
fn range_compilation_emits_checked_loop_opcodes() {
    let file = format!(
        "{}/examples/bytecode_for_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["bytecode", "ai", &file]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let instructions = String::from_utf8(output.stdout).unwrap();
    assert!(instructions.contains("RangeCheck"));
    assert!(instructions.contains("RangeAdvance"));
}

#[test]
fn zero_range_step_fails_with_n4006_not_a_hang() {
    let file = format!(
        "{}/examples/bytecode_for_bad_step_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["run-bytecode", "ai", &file]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N4006"));
}

#[test]
fn functions_compile_to_separate_bytecode_units() {
    let file = format!(
        "{}/examples/bytecode_functions_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["bytecode", "ai", &file]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let instructions = String::from_utf8(output.stdout).unwrap();
    assert!(instructions.contains("CompiledFunction"));
    assert!(instructions.contains("Call {"));
    assert!(instructions.contains("Return"));
}

#[test]
fn function_arity_errors_are_not_silently_accepted() {
    let file = format!(
        "{}/examples/bytecode_function_bad_arity_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = launch(&["run-bytecode", "ai", &file]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N4007"));
}

fn temporary_bytecode_directory() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nobody-bytecode-{}-{nanos}", std::process::id()));
    std::fs::create_dir(&path).expect("create isolated bytecode test directory");
    path
}

#[test]
fn compiled_file_runs_after_source_is_removed_and_cannot_be_overwritten() {
    let temp = temporary_bytecode_directory();
    let source = temp.join("source.nbd");
    let binary = temp.join("compiled.nbc");
    std::fs::write(
        &source,
        include_str!("../examples/bytecode_functions_ai.nbd"),
    )
    .expect("create temporary source");
    let src = source.to_str().unwrap();
    let dst = binary.to_str().unwrap();
    let compile = launch(&["compile-bytecode", "ai", src, dst]);
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let original = std::fs::read(&binary).unwrap();
    assert!(original.starts_with(b"NOBBC001"));

    std::fs::remove_file(&source).unwrap();
    let verify = launch(&["verify-bytecode", dst]);
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    let run = launch(&["run-compiled", dst]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"13\n6\n");

    let duplicate = temp.join("second-source.nbd");
    std::fs::write(&duplicate, "print(100)").unwrap();
    let overwrite = launch(&["compile-bytecode", "ai", duplicate.to_str().unwrap(), dst]);
    assert!(
        !overwrite.status.success(),
        "existing binaries must not be overwritten"
    );
    assert_eq!(std::fs::read(&binary).unwrap(), original);

    std::fs::remove_dir_all(&temp).unwrap();
}

#[test]
fn compiled_artifacts_are_identical_across_language_profiles() {
    let temp = temporary_bytecode_directory();
    let mut results = Vec::new();
    for (lang, fixture) in [
        ("ai", "bytecode_functions_ai.nbd"),
        ("sk", "bytecode_functions_sk.nbd"),
        ("en", "bytecode_functions_en.nbd"),
    ] {
        let input = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let output = temp.join(format!("{lang}.nbc"));
        let result = launch(&["compile-bytecode", lang, &input, output.to_str().unwrap()]);
        assert!(
            result.status.success(),
            "{lang}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        results.push(std::fs::read(output).unwrap());
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
    std::fs::remove_dir_all(&temp).unwrap();
}

#[test]
fn damaged_binary_is_rejected_by_both_verify_and_run_commands() {
    let temp = temporary_bytecode_directory();
    let input = format!(
        "{}/examples/bytecode_for_en.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let destination = temp.join("valid.nbc");
    let output = launch(&[
        "compile-bytecode",
        "en",
        &input,
        destination.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    let mut bytes = std::fs::read(&destination).unwrap();
    bytes[24] ^= 0x40; // corrupt an opcode without updating checksum
    std::fs::write(&destination, bytes).unwrap();
    for command in ["verify-bytecode", "run-compiled"] {
        let rejected = launch(&[command, destination.to_str().unwrap()]);
        assert!(!rejected.status.success(), "{command} accepted corruption");
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("N4004"));
    }
    std::fs::remove_dir_all(&temp).unwrap();
}

#[test]
fn typed_v2_compiled_files_run_without_sources_and_match_across_profiles() {
    let temp = temporary_bytecode_directory();
    let mut binaries = Vec::new();
    for (lang, fixture) in [
        ("ai", "native_typed_signatures_ai.nbd"),
        ("sk", "native_typed_signatures_sk.nbd"),
        ("en", "native_typed_signatures_en.nbd"),
    ] {
        let source = temp.join(format!("{lang}.nbd"));
        std::fs::write(
            &source,
            std::fs::read(format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"))).unwrap(),
        )
        .unwrap();
        let result_file = temp.join(format!("{lang}.nbc"));
        let compile = launch(&[
            "compile-bytecode",
            lang,
            source.to_str().unwrap(),
            result_file.to_str().unwrap(),
        ]);
        assert!(
            compile.status.success(),
            "{lang}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let bytes = std::fs::read(&result_file).unwrap();
        assert_eq!(&bytes[8..10], &2u16.to_le_bytes());
        binaries.push(bytes);
        std::fs::remove_file(&source).unwrap();
        let verify = launch(&["verify-bytecode", result_file.to_str().unwrap()]);
        assert!(verify.status.success());
        let output = launch(&["run-compiled", result_file.to_str().unwrap()]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"3\n8\nfalse\n5\n");
    }
    assert_eq!(binaries[0], binaries[1]);
    assert_eq!(binaries[0], binaries[2]);
    std::fs::remove_dir_all(temp).unwrap();
}

#[test]
fn typed_bytecode_rejects_incorrect_arguments_and_return_values() {
    for fixture in [
        "native_typed_bad_argument_ai.nbd",
        "native_typed_bad_return_ai.nbd",
    ] {
        let source = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let result = launch(&["run-bytecode", "ai", &source]);
        assert!(!result.status.success(), "{fixture}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("N4004"));
    }
    let source = format!(
        "{}/examples/eval_typed_string_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytecode_result = launch(&["run-bytecode", "ai", &source]);
    let ast_result = launch(&["run", "ai", &source]);
    assert!(bytecode_result.status.success(), "{}", String::from_utf8_lossy(&bytecode_result.stderr));
    assert!(ast_result.status.success());
    assert_eq!(bytecode_result.stdout, ast_result.stdout);
}

#[test]
fn saved_typed_bytecode_still_rejects_runtime_type_mismatches() {
    let temp = temporary_bytecode_directory();
    for (index, fixture) in [
        "native_typed_bad_argument_ai.nbd",
        "native_typed_bad_return_ai.nbd",
    ]
    .iter()
    .enumerate()
    {
        let source = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let binary = temp.join(format!("invalid-at-runtime-{index}.nbc"));
        let compiled = launch(&["compile-bytecode", "ai", &source, binary.to_str().unwrap()]);
        assert!(
            compiled.status.success(),
            "{fixture}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let checked = launch(&["verify-bytecode", binary.to_str().unwrap()]);
        assert!(
            checked.status.success(),
            "{fixture}: {}",
            String::from_utf8_lossy(&checked.stderr)
        );
        let executed = launch(&["run-compiled", binary.to_str().unwrap()]);
        assert!(
            !executed.status.success(),
            "{fixture} bypassed runtime type guards"
        );
        assert!(String::from_utf8_lossy(&executed.stderr).contains("N4004"));
    }
    std::fs::remove_dir_all(temp).unwrap();
}
