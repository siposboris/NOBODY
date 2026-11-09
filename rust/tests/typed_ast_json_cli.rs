//! CLI regression tests: typed AST JSON v2 is Rust-only and profile-neutral.
use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Rust crate has a repository parent")
        .join(path)
        .to_string_lossy()
        .into_owned()
}

fn run(command: &str, profile: &str, path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args([command, profile, path])
        .output()
        .expect("Nobody CLI must be built")
}

#[test]
fn v2_cli_preserves_typed_signatures_across_three_profiles() {
    let mut json = Vec::new();
    for profile in ["ai", "sk", "en"] {
        let path = fixture(&format!(
            "rust/examples/native_typed_signatures_{profile}.nbd"
        ));
        let output = run("ast-json-v2", profile, &path);
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(r#""schema":"nobody.ast.v2""#));
        assert!(text.contains(r#""value":["choose",["flag","a","b"],["bool","int","int"],"int"]"#));
        assert!(text.contains(r#""kind":"return","line":2"#));
        json.push(text);
    }
    // The signatures normalize across profiles, but ordinary source names
    // retain their selected syntax (Slovak `vypis` versus English `print`).
    assert_ne!(json[0], json[1]);
    assert!(json[1].contains(r#""value":"vypis""#));
    assert_eq!(json[0], json[2]);
}

#[test]
fn v1_cli_refuses_typed_signatures_instead_of_erasing_them() {
    let source = fixture("rust/examples/native_typed_signatures_ai.nbd");
    let output = run("ast-json", "ai", &source);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no typed function signature"));
}

#[test]
fn v1_untyped_fixture_stays_compatible() {
    let source = fixture("tests/fixtures/ast_json_parity/sk_function.nbd");
    let output = run("ast-json", "sk", &source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(r#""schema":"nobody.ast.v1""#));
}
