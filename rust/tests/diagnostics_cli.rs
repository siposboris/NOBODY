use std::process::Command;

#[test]
fn rust_cli_exposes_legacy_numbered_diagnostics() {
    let exe = env!("CARGO_BIN_EXE_nobody-rs");
    let all = Command::new(exe)
        .arg("errors")
        .output()
        .expect("run errors");
    assert!(all.status.success());
    let text = String::from_utf8(all.stdout).unwrap();
    assert_eq!(text.matches("  Príčina: ").count(), 30);
    assert!(text.contains("N1001 [E_TYPE]: Nesúlad typu"));
    assert!(text.contains("N9999 [UNCLASSIFIED]: Doteraz nezaradená chyba"));

    let one = Command::new(exe).args(["errors", "3004"]).output().unwrap();
    assert!(one.status.success());
    let one = String::from_utf8(one.stdout).unwrap();
    assert!(one.starts_with("N3004 [PARSE_LANGUAGE]:"));
    assert_eq!(one.matches("  Príčina: ").count(), 1);

    let missing = Command::new(exe).args(["errors", "42"]).output().unwrap();
    assert_eq!(missing.status.code(), Some(1));
    let invalid = Command::new(exe)
        .args(["errors", "invalid"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(2));
}
