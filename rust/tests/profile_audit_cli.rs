//! Verify active-profile structural audit runs in the compiled Rust CLI.
use std::process::Command;

#[test]
fn audit_40_profiles_without_python() {
    let output = Command::new(env!("CARGO_BIN_EXE_nobody"))
        .arg("audit-languages")
        .output()
        .expect("launch native Nobody");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("40 active syntax profiles; 0 structural errors")
    );
}
