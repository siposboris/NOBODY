//! CLI regression tests for Rust-only Nobody syntax AST source emission.
use std::fs;
use std::path::Path;
use std::process::Command;

fn fixture(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository parent")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("Nobody Rust CLI must be available")
}

#[test]
fn typed_ast_json_emits_source_in_three_profiles() {
    for profile in ["ai", "sk", "en"] {
        let input = fixture(&format!(
            "rust/examples/native_typed_signatures_{profile}.nbd"
        ));
        let exported = run(&["ast-json-v2", profile, &input]);
        assert!(
            exported.status.success(),
            "export {profile}: {}",
            String::from_utf8_lossy(&exported.stderr)
        );
        let filename = std::env::temp_dir().join(format!(
            "nobody_emit_v2_{}_{}.json",
            std::process::id(),
            profile
        ));
        fs::write(&filename, &exported.stdout).unwrap();
        let output = run(&["emit-ast-v2", profile, filename.to_str().unwrap()]);
        let _ = fs::remove_file(filename);
        assert!(
            output.status.success(),
            "emitter {profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let emitted = String::from_utf8(output.stdout).unwrap();
        assert!(emitted.contains("choose"));
        assert!(emitted.contains("flag:"));
        match profile {
            "sk" => assert!(emitted.contains("logicke")),
            _ => assert!(emitted.contains("bool")),
        }
        let formatted = run(&["format-source", profile, &input]);
        assert!(
            formatted.status.success(),
            "format-source {profile}: {}",
            String::from_utf8_lossy(&formatted.stderr)
        );
        assert_eq!(formatted.stdout, emitted.as_bytes());
    }
}

#[test]
fn malformed_import_fails_instead_of_producing_source() {
    let path = std::env::temp_dir().join(format!(
        "nobody_emit_v2_{}_invalid.json",
        std::process::id()
    ));
    fs::write(&path, r#"{"schema":"nobody.ast.v2","ast":null}"#).unwrap();
    let output = run(&["emit-ast-v2", "ai", path.to_str().unwrap()]);
    let _ = fs::remove_file(path);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N5001"));
}
