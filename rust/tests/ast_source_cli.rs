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

#[test]
fn format_check_is_read_only_canonical_and_profile_aware() {
    for (profile, original) in [
        ("ai", "let greeting = \"Nobody\"\nprint(greeting)\n"),
        ("sk", "nech greeting = \"Nobody\"\nvypis(greeting)\n"),
        ("en", "let greeting = \"Nobody\"\nprint(greeting)\n"),
    ] {
        let path = std::env::temp_dir().join(format!(
            "nobody_format_check_{}_{}.nbd",
            std::process::id(),
            profile
        ));
        fs::write(&path, original).unwrap();
        let original_bytes = fs::read(&path).unwrap();
        let args = ["format-check", profile, path.to_str().unwrap()];
        let unformatted = run(&args);
        assert!(!unformatted.status.success(), "{profile}: source not canonical");
        assert!(unformatted.stdout.is_empty());
        assert!(String::from_utf8_lossy(&unformatted.stderr).contains("N5002"));
        assert_eq!(fs::read(&path).unwrap(), original_bytes);

        let formatted = run(&["format-source", profile, path.to_str().unwrap()]);
        assert!(formatted.status.success(), "{profile}");
        fs::write(&path, &formatted.stdout).unwrap();
        let checked = run(&args);
        assert!(
            checked.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&checked.stderr)
        );
        assert_eq!(fs::read(&path).unwrap(), formatted.stdout);
        let rerun = run(&["format-source", profile, path.to_str().unwrap()]);
        assert_eq!(rerun.stdout, formatted.stdout, "{profile}: idempotence");
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn format_check_rejects_invalid_syntax_without_editing() {
    let path = std::env::temp_dir().join(format!(
        "nobody_format_check_bad_{}.nbd",
        std::process::id()
    ));
    fs::write(&path, "let x = ???").unwrap();
    let result = run(&["format-check", "ai", path.to_str().unwrap()]);
    assert!(!result.status.success());
    assert_eq!(fs::read(&path).unwrap(), b"let x = ???");
    fs::remove_file(path).unwrap();
}
