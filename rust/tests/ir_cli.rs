//! Native Rust CLI tests for canonical Nobody IR.
use std::path::Path;
use std::process::Command;

fn repo_path(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().expect("rust crate has repository parent")
        .join(relative).to_string_lossy().into_owned()
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args).output().expect("Rust CLI must be built")
}

#[test]
fn ir_subcommand_outputs_versioned_module() {
    let file=repo_path("rust/examples/basic_sk.nbd");
    let output=run(&["ir","sk",&file]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nobody.ir.v1"));
    assert!(stdout.contains("Builtin("));
}

#[test]
fn compare_ir_reports_matching_languages() {
    let sk=repo_path("rust/examples/basic_sk.nbd");
    let en=repo_path("rust/examples/basic_en.nbd");
    let output=run(&["compare-ir","sk",&sk,"en",&en]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("MATCH"));
}

#[test]
fn compare_ir_rejects_mismatched_programs() {
    let sk=repo_path("rust/examples/basic_sk.nbd");
    let different=repo_path("examples/hello_sk.nbd");
    let output=run(&["compare-ir","sk",&sk,"sk",&different]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("DIFFERENT"));
}

#[test]
fn ir_does_not_execute_programs() {
    let file=repo_path("examples/hello_ai.nbd");
    let output=run(&["ir","ai",&file]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nobody.ir.v1"));
    assert!(!stdout.contains("AST context validation passed"));
}
