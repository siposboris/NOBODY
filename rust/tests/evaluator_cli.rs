//! End-to-end CLI smoke for the deliberately restricted Rust evaluator.
use std::fs;
use std::process::Command;

#[test]
fn run_ai_and_slovak_have_matching_observable_output() {
    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    let root = env!("CARGO_MANIFEST_DIR");
    let ai = Command::new(executable)
        .args(["run", "ai", &format!("{root}/examples/eval_basic_ai.nbd")])
        .output()
        .expect("start AI evaluator");
    let sk = Command::new(executable)
        .args(["run", "sk", &format!("{root}/examples/eval_basic_sk.nbd")])
        .output()
        .expect("start Slovak evaluator");
    assert!(
        ai.status.success(),
        "{}",
        String::from_utf8_lossy(&ai.stderr)
    );
    assert!(
        sk.status.success(),
        "{}",
        String::from_utf8_lossy(&sk.stderr)
    );
    assert_eq!(ai.stdout, b"9\n8\n");
    assert_eq!(ai.stdout, sk.stdout);
}

#[test]
fn parsing_an_example_never_executes_it() {
    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    let root = env!("CARGO_MANIFEST_DIR");
    let parsed = Command::new(executable)
        .args(["parse", "ai", &format!("{root}/examples/eval_basic_ai.nbd")])
        .output()
        .expect("start parser");
    assert!(parsed.status.success());
    assert!(!String::from_utf8_lossy(&parsed.stdout).starts_with("9\n8\n"));
}

#[test]
fn failed_run_reports_numbered_diagnostic() {
    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    let root = env!("CARGO_MANIFEST_DIR");
    let failed = Command::new(executable)
        .args([
            "run",
            "ai",
            &format!("{root}/examples/eval_bad_zero_ai.nbd"),
        ])
        .output()
        .expect("start evaluator");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("N4002"));
    assert!(failed.stdout.is_empty());
}

#[test]
fn no_legacy_python_execution_dependency() {
    // The Rust CLI is a directly invoked compiled executable, not a Python subprocess.
    let main = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/evaluator.rs"))
        .expect("evaluator source exists");
    assert!(!main.contains("std::process::Command"));
}
