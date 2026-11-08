//! Verify Python-free canonical Nobody executable and conservative CLI claims.
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(args)
        .output()
        .expect("launch Rust nobody executable")
}

#[test]
fn native_nobody_reports_true_implementation_status() {
    let output = run(&["info"]);
    assert!(output.status.success());
    let message = String::from_utf8(output.stdout).unwrap();
    assert!(message.contains(r#""implementation": "Rust""#));
    assert!(message.contains(r#""implemented_syntax_profiles": 40"#));
    assert!(message.contains(r#""self_hosted": false"#));
    assert!(message.contains(r#""python_required_for_rust_cli": false"#));
    assert!(message.contains(r#""native_machine_code_compiler": false"#));
}

#[test]
fn native_nobody_lists_only_available_syntax_profiles() {
    let output = run(&["languages"]);
    assert!(output.status.success());
    let message = String::from_utf8(output.stdout).unwrap();
    let codes: Vec<&str> = message.lines().collect();
    assert_eq!(codes.len(), 40);
    assert!(codes.contains(&"ai"));
    assert!(codes.contains(&"sk"));
}

#[test]
fn native_nobody_executes_without_invoking_python() {
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/eval_basic_sk.nbd");
    let output = run(&["run", "sk", file]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"9\n8\n");
}

#[test]
fn native_nobody_errors_on_unsupported_subcommand() {
    let output = run(&["ast-json", "file.nbd"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N9001"));
}
