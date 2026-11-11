//! CLI capability reporting must agree with the implemented compiler backends.
use std::process::Command;

#[test]
fn info_reports_restricted_native_backend_without_claiming_self_hosting() {
    let output = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .arg("info")
        .output()
        .expect("run nobody-rs info");
    assert!(output.status.success(), "info failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).expect("info must be UTF-8");
    assert!(stdout.contains(r#""native_machine_code_compiler": true"#));
    assert!(stdout.contains(r#""native_target": "linux-x86_64-elf""#));
    assert!(stdout.contains(r#""native_scope": "restricted experimental subset""#));
    assert!(stdout.contains(r#""self_hosted": false"#));
    assert!(stdout.contains(r#""python_required_for_rust_cli": false"#));
}
