//! Real Linux x86-64 ELF acceptance checks: the generated program itself
//! runs without Nobody, Cargo, Rust runtime, libc or bytecode VM.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("nobody-native-test-{}-{nanos}", std::process::id()));
    std::fs::create_dir(&path).expect("create temporary test directory");
    path
}

fn nobody(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(args)
        .output()
        .expect("launch Nobody compiler")
}

fn compile(profile: &str, source: &Path, output: &Path) -> Output {
    nobody(&[
        "compile-native",
        profile,
        source.to_str().unwrap(),
        output.to_str().unwrap(),
    ])
}

#[test]
fn real_elf_outputs_match_existing_interpreter_for_ai_sk_en() {
    for (profile, fixture) in [
        ("ai", "native_basic_ai.nbd"),
        ("ai", "native_flow_ai.nbd"),
        ("sk", "native_flow_sk.nbd"),
        ("en", "native_flow_en.nbd"),
    ] {
        let input = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let tmp = temporary();
        let binary = tmp.join("program");
        let compiled = compile(profile, Path::new(&input), &binary);
        assert!(
            compiled.status.success(),
            "{fixture}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let header = std::fs::read(&binary).unwrap();
        assert!(header.starts_with(b"\x7fELF"), "{fixture} produced no ELF binary");
        let machine = Command::new("readelf").args(["-h", binary.to_str().unwrap()])
            .output().expect("readelf available in CI");
        assert!(machine.status.success());
        let description = String::from_utf8_lossy(&machine.stdout);
        assert!(description.contains("ELF64"));
        assert!(description.contains("X86-64"));

        let direct = Command::new(&binary).output().expect("launch native executable");
        let tree = nobody(&["run", profile, &input]);
        assert!(direct.status.success(), "{fixture} native exit code: {:?}", direct.status);
        assert!(tree.status.success(), "{fixture} tree: {}", String::from_utf8_lossy(&tree.stderr));
        assert_eq!(direct.stdout, tree.stdout, "{fixture} stdout differs");
        assert!(direct.stderr.is_empty());
        std::fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn native_output_runs_after_source_removed_and_no_clobber_of_existing_binary() {
    let tmp = temporary();
    let source = tmp.join("source.nbd");
    let output = tmp.join("standalone");
    std::fs::write(&source, "let n = 12\nprint(n * 3)\n").unwrap();
    let compiled = compile("ai", &source, &output);
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    let before = std::fs::read(&output).unwrap();
    std::fs::remove_file(&source).unwrap();
    let direct = Command::new(&output).output().unwrap();
    assert_eq!(direct.stdout, b"36\n");
    std::fs::write(&source, "print(10)").unwrap();
    let repeat = compile("ai", &source, &output);
    assert!(!repeat.status.success(), "existing output was overwritten");
    assert_eq!(std::fs::read(&output).unwrap(), before);
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn signed_limits_and_zero_print_exact_decimal_text() {
    let tmp = temporary();
    let source = tmp.join("limits.nbd");
    let binary = tmp.join("limits");
    std::fs::write(&source, "print(0)\nprint(9223372036854775807)\nprint(-9223372036854775807)\n").unwrap();
    let result = compile("ai", &source, &binary);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let direct = Command::new(&binary).output().unwrap();
    assert!(direct.status.success());
    assert_eq!(direct.stdout, b"0\n9223372036854775807\n-9223372036854775807\n");
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn native_overflow_and_infinite_loop_end_with_finite_nonzero_exit() {
    for (fixture, expected_exit) in [
        ("native_overflow_ai.nbd", 3),
        ("native_infinite_ai.nbd", 5),
    ] {
        let source = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let tmp = temporary();
        let binary = tmp.join("program");
        let compiled = compile("ai", Path::new(&source), &binary);
        assert!(compiled.status.success(), "{fixture}: {}", String::from_utf8_lossy(&compiled.stderr));
        let direct = Command::new(&binary).output().unwrap();
        assert_eq!(direct.status.code(), Some(expected_exit), "{fixture}");
        std::fs::remove_dir_all(tmp).unwrap();
    }
}

#[test]
fn unsupported_features_are_rejected_before_creating_output() {
    let tmp = temporary();
    let source = tmp.join("unsupported.nbd");
    let output = tmp.join("not-created");
    std::fs::write(&source, "fn f(x) { return x + 1 }\nprint(f(3))").unwrap();
    let result = compile("ai", &source, &output);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("N4004"));
    assert!(!output.exists());
    std::fs::remove_dir_all(tmp).unwrap();
}
