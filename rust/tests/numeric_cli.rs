use std::{path::Path, process::Command};

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn malformed_number_fails_lex_with_n2006() {
    let output = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(["lex", &fixture("invalid_number_ai.nbd")])
        .output()
        .expect("run Nobody CLI");
    assert!(
        !output.status.success(),
        "invalid numeric literal must be rejected"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("N2006"), "actual stderr: {stderr}");
}

#[test]
fn ordinary_example_still_lexes() {
    let output = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(["lex", &fixture("basic_ai.nbd")])
        .output()
        .expect("run Nobody CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
