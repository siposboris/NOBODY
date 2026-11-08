use std::{path::Path, process::Command};
fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
        .to_string_lossy()
        .into_owned()
}
#[test]
fn wrong_arity_is_reported_as_n1005() {
    let path = fixture("arity_bad_ai.nbd");
    let result = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(["typecheck", "ai", &path])
        .output()
        .unwrap();
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("N1005"), "{stderr}");
    assert!(stderr.contains("add"), "{stderr}");
}
#[test]
fn correct_arity_passes() {
    let path = fixture("arity_good_ai.nbd");
    let result = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(["typecheck", "ai", &path])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
