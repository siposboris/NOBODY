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
    for command in ["parse", "ast-json", "type-check"] {
        let result = run(&[command, source, "--wrong", "sk"]);
        assert!(!result.status.success(), "{command} accepted wrong flag");
    }
}
