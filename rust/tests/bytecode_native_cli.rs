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
