use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("launch native Rust CLI")
}

#[test]
fn legacy_python_profile_flags_match_native_rust_commands() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    for (legacy, native) in [
        ("parse", "parse"),
        ("ast-json", "ast-json"),
        ("type-check", "typecheck"),
        ("ast-run", "run"),
    ] {
        let positional = run(&[native, "sk", source]);
        let flagged = run(&[legacy, source, "--lang", "sk"]);
        assert!(
            positional.status.success(),
            "{native} stderr: {}",
            String::from_utf8_lossy(&positional.stderr)
        );
        assert!(
            flagged.status.success(),
            "{legacy} stderr: {}",
            String::from_utf8_lossy(&flagged.stderr)
        );
        assert_eq!(flagged.stdout, positional.stdout, "{legacy} stdout");
        assert_eq!(flagged.stderr, positional.stderr, "{legacy} stderr");
    }
}

#[test]
fn malformed_legacy_profile_flags_are_rejected() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    for command in ["parse", "ast-json", "type-check", "ast-run"] {
        let result = run(&[command, source, "--wrong", "sk"]);
        assert!(!result.status.success(), "{command} accepted wrong flag");
    }
}

#[test]
fn legacy_ast_run_enforces_configurable_step_budget() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    let bounded = run(&["ast-run", source, "--lang", "sk", "--max-steps", "1"]);
    assert!(!bounded.status.success(), "one step must not execute a full program");
    assert!(String::from_utf8_lossy(&bounded.stderr).contains("N4005"));

    let generous = run(&["ast-run", source, "--lang", "sk", "--max-steps", "10000"]);
    let unbounded_cli = run(&["run", "sk", source]);
    assert!(generous.status.success(), "{}", String::from_utf8_lossy(&generous.stderr));
    assert_eq!(generous.stdout, unbounded_cli.stdout);
    assert_eq!(generous.stderr, unbounded_cli.stderr);

    for invalid in ["abc", "-1", "1.5"] {
        let result = run(&["ast-run", source, "--lang", "sk", "--max-steps", invalid]);
        assert!(!result.status.success(), "accepted invalid budget {invalid}");
    }
}

#[test]
fn legacy_ast_run_ai_profile_matches_native_rust_execution() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/basic_ai.nbd");
    let positional = run(&["run", "ai", source]);
    let legacy = run(&["ast-run", source, "--lang", "ai"]);
    assert!(
        positional.status.success(),
        "native AI execution failed: {}",
        String::from_utf8_lossy(&positional.stderr)
    );
    assert!(
        legacy.status.success(),
        "legacy AI execution failed: {}",
        String::from_utf8_lossy(&legacy.stderr)
    );
    assert_eq!(legacy.stdout, positional.stdout);
    assert_eq!(legacy.stderr, positional.stderr);
}

#[test]
fn legacy_ast_run_rejects_mixed_source_profile() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    let legacy = run(&["ast-run", source, "--lang", "ai"]);
    let native = run(&["run", "ai", source]);
    assert!(
        !legacy.status.success(),
        "mixed source profile must be rejected"
    );
    assert_eq!(legacy.status, native.status);
    assert_eq!(legacy.stderr, native.stderr);
}

#[test]
fn native_rust_typed_declaration_checks_annotation() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/basic_ai.nbd");
    let result = run(&["run", "ai", source]);
    assert!(
        result.status.success(),
        "typed declaration should execute: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"31\n");
}
