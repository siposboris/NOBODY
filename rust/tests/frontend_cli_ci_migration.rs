use std::process::Command;

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("launch native Nobody Rust CLI")
}

#[test]
fn frontend_ci_legacy_smoke_commands_are_native() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    for args in [
        vec!["errors", "1001"],
        vec!["type-check", source, "--lang", "sk"],
        vec!["ast-json", source, "--lang", "sk"],
        vec!["translate", source, "--from-lang", "sk", "--to", "ai"],
        vec!["audit-languages"],
        vec!["info"],
        vec!["languages", "--json"],
        vec!["ast-run", source, "--lang", "sk"],
    ] {
        let output = cli(&args);
        assert!(
            output.status.success(),
            "Rust CLI {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.stdout.is_empty(),
            "Rust CLI {:?} returned empty stdout",
            args
        );
    }
}

#[test]
fn native_cli_rejects_mixed_language_source() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    let output = cli(&["ast-run", source, "--lang", "ai"]);
    assert!(
        !output.status.success(),
        "mixed syntax profiles must be rejected"
    );
}
