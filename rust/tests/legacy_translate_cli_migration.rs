use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("launch native Rust CLI")
}

#[test]
fn python_translate_flags_match_native_rust_translation() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    let native = run(&["translate", "sk", "ai", source]);
    let legacy = run(&["translate", source, "--from-lang", "sk", "--to", "ai"]);
    assert!(
        native.status.success(),
        "native stderr: {}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert!(
        legacy.status.success(),
        "legacy stderr: {}",
        String::from_utf8_lossy(&legacy.stderr)
    );
    assert_eq!(legacy.stdout, native.stdout);
    assert_eq!(legacy.stderr, native.stderr);
}

#[test]
fn malformed_legacy_flags_are_rejected_not_silently_reinterpreted() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/hello_sk.nbd");
    let result = run(&["translate", source, "--from-lang", "sk", "--wrong", "ai"]);
    assert!(!result.status.success());
}
