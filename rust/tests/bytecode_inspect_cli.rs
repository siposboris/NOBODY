//! Inspection should validate persisted bytecode without executing it
//! and without displaying untrusted text or function identifiers.
use std::fs;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn inspector_reports_v2_v3_metadata_without_executing_or_editing() {
    let dir = std::env::temp_dir().join(format!(
        "nobody-inspect-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    for (profile, filename, version) in [
        ("ai", "eval_basic_ai.nbd", 2u16),
        ("sk", "bytecode_text_sk.nbd", 3u16),
        ("ai", "native_len_ai.nbd", 3u16),
    ] {
        let source = format!("{}/examples/{filename}", env!("CARGO_MANIFEST_DIR"));
        let target = dir.join(format!("{profile}-{filename}.nbc"));
        let path = target.to_str().unwrap();
        let generated = cli(&["compile-bytecode", profile, &source, path]);
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );
        let original = fs::read(&target).unwrap();
        assert_eq!(u16::from_le_bytes([original[8], original[9]]), version);
        let inspected = cli(&["inspect-bytecode", path]);
        assert!(inspected.status.success(), "{}", String::from_utf8_lossy(&inspected.stderr));
        let output = String::from_utf8(inspected.stdout).unwrap();
        assert!(output.contains("format: NOBBC001\n"));
        assert!(output.contains(&format!("wire_version: {version}\n")));
        assert!(output.contains("verified: true\n"));
        assert!(output.contains("executed: false\n"));
        assert!(output.contains("main_instructions: "));
        assert!(output.contains("function_count: "));
        assert!(output.contains("function_instructions: "));
        assert!(output.contains("unicode_length_ops: "));
        assert_eq!(fs::read(&target).unwrap(), original, "inspector changed bytecode");
        // Repeated inspection is deterministic and doesn't run the bytecode.
        assert_eq!(cli(&["inspect-bytecode", path]).stdout, output.as_bytes());
        if filename.contains("native_len") {
            assert!(output.contains("unicode_length_ops: 3\n") || output.contains("unicode_length_ops: 4\n"));
        }
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inspector_rejects_corrupt_or_missing_files_and_never_executes_them() {
    let dir = std::env::temp_dir().join(format!(
        "nobody-inspect-corrupt-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let invalid = dir.join("corrupt.nbc");
    fs::write(&invalid, b"NOBBC001").unwrap();
    let result = cli(&["inspect-bytecode", invalid.to_str().unwrap()]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let missing = dir.join("missing.nbc");
    let result = cli(&["inspect-bytecode", missing.to_str().unwrap()]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    fs::remove_dir_all(dir).unwrap();
}
