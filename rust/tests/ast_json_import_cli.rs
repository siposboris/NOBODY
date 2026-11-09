//! End-to-end CLI checks for the Rust-only nobody.ast.v2 importer.
use std::fs;
use std::path::Path;
use std::process::Command;

fn source_path(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Rust crate has a repository parent")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("Nobody Rust CLI must run")
}

#[test]
fn exported_typed_ast_is_verified_and_normalized_without_python() {
    let src = source_path("rust/examples/native_typed_signatures_sk.nbd");
    let exported = run(&["ast-json-v2", "sk", &src]);
    assert!(
        exported.status.success(),
        "export: {}",
        String::from_utf8_lossy(&exported.stderr)
    );

    let path =
        std::env::temp_dir().join(format!("nobody_ast_v2_{}_valid.json", std::process::id()));
    fs::write(&path, &exported.stdout).unwrap();
    let name = path.to_str().unwrap();
    let verified = run(&["verify-ast-json-v2", name]);
    let normalized = run(&["normalize-ast-json-v2", name]);
    let _ = fs::remove_file(&path);
    assert!(
        verified.status.success(),
        "verify: {}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert!(String::from_utf8_lossy(&verified.stdout).contains("structural only"));
    assert!(
        normalized.status.success(),
        "normalize: {}",
        String::from_utf8_lossy(&normalized.stderr)
    );
    assert_eq!(normalized.stdout, exported.stdout);
}

#[test]
fn importer_rejects_corrupt_v2_signature_without_running_it() {
    let path = std::env::temp_dir().join(format!("nobody_ast_v2_{}_bad.json", std::process::id()));
    fs::write(
        &path,
        r#"{"ast":{"children":[{"children":[],"kind":"function","line":1,"value":["f",["x"],[],"int"]}],"kind":"module","line":1,"value":null},"schema":"nobody.ast.v2"}"#,
    )
    .unwrap();
    let output = run(&["verify-ast-json-v2", path.to_str().unwrap()]);
    let _ = fs::remove_file(&path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N5001"));
}
