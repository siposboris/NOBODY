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
