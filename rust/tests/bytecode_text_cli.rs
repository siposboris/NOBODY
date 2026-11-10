//! UTF-8 portable .nbc v3 tests: compile, verify, and run without Python.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn compiler(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("run Rust Nobody CLI")
}

fn temporary() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nobody-text-v3-{}-{nanos}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    dir
}

#[test]
fn v3_utf8_programs_compile_verify_and_execute_from_disk_in_three_profiles() {
    let dir = temporary();
    for profile in ["ai", "sk", "en"] {
        let source = format!(
            "{}/examples/bytecode_text_{profile}.nbd",
            env!("CARGO_MANIFEST_DIR")
        );
        let output = dir.join(format!("code_{profile}.nbc"));
        let path = output.to_str().unwrap();
        let build = compiler(&["compile-bytecode", profile, &source, path]);
        assert!(
            build.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let data = fs::read(&output).unwrap();
        assert_eq!(&data[..8], b"NOBBC001");
        assert_eq!(u16::from_le_bytes([data[8], data[9]]), 3);
        let verified = compiler(&["verify-bytecode", path]);
        assert!(
            verified.status.success(),
            "{profile}: {:?}",
            verified.stderr
        );
        let disk = compiler(&["run-compiled", path]);
        let vm = compiler(&["run-bytecode", profile, &source]);
        let ast = compiler(&["run", profile, &source]);
        for result in [&disk, &vm, &ast] {
            assert!(
                result.status.success(),
                "{profile}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        assert_eq!(
            disk.stdout, ast.stdout,
            "{profile}: stored .nbc differs from AST"
        );
        assert_eq!(
            vm.stdout, ast.stdout,
            "{profile}: in-memory VM differs from AST"
        );
        assert!(disk
            .stdout
            .windows("Život ☸".len())
            .any(|x| x == "Život ☸".as_bytes()));
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn stored_v3_unicode_length_matches_ast_vm_and_native_semantics() {
    let dir = temporary();
    for profile in ["ai", "sk", "en"] {
        // These fixtures are also exercised as real standalone ELF programs.
        let source = format!(
            "{}/examples/native_len_{profile}.nbd",
            env!("CARGO_MANIFEST_DIR")
        );
        let target = dir.join(format!("len-{profile}.nbc"));
        let path = target.to_str().unwrap();
        let compilation = compiler(&["compile-bytecode", profile, &source, path]);
        assert!(
            compilation.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let bytes = fs::read(&target).unwrap();
        assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), 3);
        let verify = compiler(&["verify-bytecode", path]);
        assert!(verify.status.success(), "{profile}: {:?}", verify.stderr);
        let stored = compiler(&["run-compiled", path]);
        let vm = compiler(&["run-bytecode", profile, &source]);
        let reference = compiler(&["run", profile, &source]);
        for result in [&stored, &vm, &reference] {
            assert!(
                result.status.success(),
                "{profile}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        assert_eq!(stored.stdout, reference.stdout, "{profile}: v3 disk parity");
        assert_eq!(vm.stdout, reference.stdout, "{profile}: live VM parity");
        assert!(stored.stdout.starts_with(b"7 0 3\n"));
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn text_bytecode_corruption_is_rejected_instead_of_executed() {
    let dir = temporary();
    let source = format!(
        "{}/examples/bytecode_text_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = dir.join("broken.nbc");
    let target = output.to_str().unwrap();
    assert!(compiler(&["compile-bytecode", "ai", &source, target])
        .status
        .success());
    let mut data = fs::read(&output).unwrap();
    data[30] ^= 0xff;
    fs::write(&output, data).unwrap();
    let invalid = compiler(&["verify-bytecode", target]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("N4004"));
    assert!(!compiler(&["run-compiled", target]).status.success());
    fs::remove_dir_all(dir).unwrap();
}
