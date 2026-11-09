//! CLI regression tests: V2 JSON AST -> Rust syntax AST, no execution.
use std::fs;
use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Rust crate parent exists")
        .join(path)
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("Rust CLI executable exists")
}

#[test]
fn source_and_imported_ast_match_in_three_profiles() {
    for profile in ["ai", "sk", "en"] {
        let source = fixture(&format!(
            "rust/examples/native_typed_signatures_{profile}.nbd"
        ));
        let exported = run(&["ast-json-v2", profile, &source]);
        assert!(
            exported.status.success(),
            "export {profile}: {}",
            String::from_utf8_lossy(&exported.stderr)
        );
        let path = std::env::temp_dir().join(format!(
            "nobody_compare_ast_v2_{}_{}.json",
            std::process::id(),
            profile
        ));
        fs::write(&path, &exported.stdout).unwrap();
        let compare = run(&["compare-ast-v2", profile, &source, path.to_str().unwrap()]);
        let _ = fs::remove_file(path);
        assert!(
            compare.status.success(),
            "compare {profile}: {}",
            String::from_utf8_lossy(&compare.stderr)
        );
        assert!(String::from_utf8_lossy(&compare.stdout).contains("MATCH:"));
    }
}

#[test]
fn changed_json_is_not_accepted_as_equivalent() {
    let src = fixture("rust/examples/native_typed_signatures_ai.nbd");
    let exported = run(&["ast-json-v2", "ai", &src]);
    assert!(exported.status.success());
    let mut json = String::from_utf8(exported.stdout).unwrap();
    assert!(json.contains(r#""value":"4""#));
    json = json.replace(r#""value":"4""#, r#""value":"5""#);
    let path = std::env::temp_dir().join(format!(
        "nobody_compare_ast_v2_{}_changed.json",
        std::process::id()
    ));
    fs::write(&path, json).unwrap();
    let output = run(&["compare-ast-v2", "ai", &src, path.to_str().unwrap()]);
    let _ = fs::remove_file(path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("DIFFERENT:"));
}
