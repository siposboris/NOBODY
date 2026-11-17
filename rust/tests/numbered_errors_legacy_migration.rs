//! Native Rust coverage migrated from tests/test_020_errors.py.
//! Legacy Python remains until the full numbered-diagnostics suite is covered.
use nobody::scan;
use std::process::Command;

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("run Rust Nobody CLI")
}

#[test]
fn legacy_unterminated_string_has_numbered_lexical_error() {
    let issue = scan("vypis(\"never closed)").expect_err("unterminated string must fail");
    assert_eq!(issue.code, 2001);
}

#[test]
fn legacy_numbered_error_lookup_is_native() {
    for number in ["1001", "1005"] {
        let output = cli(&["errors", number]);
        assert!(
            output.status.success(),
            "error {number} lookup failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains(number), "error {number} missing from: {text}");
    }
}

#[test]
fn legacy_cli_rejects_unknown_option() {
    let output = cli(&["--nonexistent"]);
    assert!(!output.status.success(), "unknown CLI option was accepted");
}

#[test]
fn legacy_lowercase_ascii_identifier_restriction() {
    for source in ["Vypis(1)", "vypis(1) × 2"] {
        assert!(scan(source).is_err(), "invalid syntax accepted: {source}");
    }
}
