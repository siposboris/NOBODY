//! Real Linux x86-64 ELF acceptance checks: the generated program itself
//! runs without Nobody, Cargo, Rust runtime, libc or bytecode VM.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nobody-native-test-{}-{nanos}", std::process::id()));
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
        ("ai", "bytecode_for_ai.nbd"),
        ("sk", "bytecode_for_sk.nbd"),
        ("en", "bytecode_for_en.nbd"),
        ("ai", "native_range_nested_ai.nbd"),
        ("ai", "native_range_shadow_ai.nbd"),
        ("ai", "native_range_mix_ai.nbd"),
        ("ai", "native_range_overflow_ai.nbd"),
        ("ai", "native_range_empty_ai.nbd"),
        ("ai", "bytecode_functions_ai.nbd"),
        ("sk", "bytecode_functions_sk.nbd"),
        ("en", "bytecode_functions_en.nbd"),
        ("ai", "bytecode_function_control_ai.nbd"),
        ("ai", "native_function_args_ai.nbd"),
        ("ai", "native_function_branches_ai.nbd"),
        ("ai", "native_function_loops_ai.nbd"),
        ("ai", "native_function_scope_ai.nbd"),
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
        assert!(
            header.starts_with(b"\x7fELF"),
            "{fixture} produced no ELF binary"
        );
        let machine = Command::new("readelf")
            .args(["-h", binary.to_str().unwrap()])
            .output()
            .expect("readelf available in CI");
        assert!(machine.status.success());
        let description = String::from_utf8_lossy(&machine.stdout);
        assert!(description.contains("ELF64"));
        assert!(description.contains("X86-64"));

        let direct = Command::new(&binary)
            .output()
            .expect("launch native executable");
        let tree = nobody(&["run", profile, &input]);
        assert!(
            direct.status.success(),
            "{fixture} native exit code: {:?}",
            direct.status
        );
        assert!(
            tree.status.success(),
            "{fixture} tree: {}",
            String::from_utf8_lossy(&tree.stderr)
        );
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
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
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
    std::fs::write(
        &source,
        "print(0)\nprint(9223372036854775807)\nprint(-9223372036854775807)\n",
    )
    .unwrap();
    let result = compile("ai", &source, &binary);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let direct = Command::new(&binary).output().unwrap();
    assert!(direct.status.success());
    assert_eq!(
        direct.stdout,
        b"0\n9223372036854775807\n-9223372036854775807\n"
    );
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn native_overflow_and_infinite_loop_end_with_finite_nonzero_exit() {
    for (fixture, expected_exit) in [("native_overflow_ai.nbd", 3), ("native_infinite_ai.nbd", 5)] {
        let source = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let tmp = temporary();
        let binary = tmp.join("program");
        let compiled = compile("ai", Path::new(&source), &binary);
        assert!(
            compiled.status.success(),
            "{fixture}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
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
    std::fs::write(&source, "fn f(x) { print(x) }\nprint(f(3))").unwrap();
    let result = compile("ai", &source, &output);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("N4004"));
    assert!(!output.exists());
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn zero_step_in_even_an_empty_native_range_is_a_bounded_exit() {
    let source = format!(
        "{}/examples/native_range_zero_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let tmp = temporary();
    let binary = tmp.join("range-zero");
    let compiled = compile("ai", Path::new(&source), &binary);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let native = Command::new(&binary).output().unwrap();
    assert_eq!(native.status.code(), Some(6));
    assert!(native.stdout.is_empty());
    assert!(native.stderr.is_empty());
    let tree = nobody(&["run", "ai", &source]);
    assert!(!tree.status.success());
    assert!(String::from_utf8_lossy(&tree.stderr).contains("N4006"));
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn native_range_signed_boundary_terminates_without_overflow_error() {
    let input = format!(
        "{}/examples/native_range_overflow_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let tmp = temporary();
    let binary = tmp.join("range-overflow");
    let compiled = compile("ai", Path::new(&input), &binary);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{:?}", native.status);
    assert_eq!(
        native.stdout,
        b"9223372036854775806\n-9223372036854775806\n"
    );
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn native_range_preserves_shadowed_outer_iteration_variable() {
    let input = format!(
        "{}/examples/native_range_shadow_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let tmp = temporary();
    let binary = tmp.join("range-shadow");
    let compiled = compile("ai", Path::new(&input), &binary);
    assert!(compiled.status.success());
    let native = Command::new(&binary).output().unwrap();
    assert_eq!(native.stdout, b"0\n1\n2\n99\n");
    assert!(native.status.success());
    std::fs::remove_dir_all(tmp).unwrap();
}

#[test]
fn native_function_calls_preserve_arg_order_and_stack_frames() {
    let source = format!(
        "{}/examples/native_function_args_ai.nbd",
        env!("CARGO_MANIFEST_DIR")
    );
    let temp = temporary();
    let binary = temp.join("functions-args");
    let compiled = compile("ai", Path::new(&source), &binary);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let direct = Command::new(&binary).output().unwrap();
    assert!(direct.status.success());
    assert_eq!(direct.stdout, b"123\n474\n");
    std::fs::remove_dir_all(temp).unwrap();
}

#[test]
fn native_functions_reject_global_capture_forward_calls_and_missing_return() {
    for (fixture, error_code) in [
        ("native_function_bad_implicit_ai.nbd", "N4004"),
        ("native_function_bad_forward_ai.nbd", "N4001"),
        ("native_function_bad_global_ai.nbd", "N4001"),
        ("bytecode_function_bad_arity_ai.nbd", "N4007"),
        ("bytecode_function_recursive_ai.nbd", "N4001"),
    ] {
        let source = format!("{}/examples/{fixture}", env!("CARGO_MANIFEST_DIR"));
        let temp = temporary();
        let output = temp.join("should-not-exist");
        let compiled = compile("ai", Path::new(&source), &output);
        assert!(!compiled.status.success(), "{fixture} should be rejected");
        assert!(
            String::from_utf8_lossy(&compiled.stderr).contains(error_code),
            "{fixture}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        assert!(!output.exists());
        std::fs::remove_dir_all(temp).unwrap();
    }
}

#[test]
fn standalone_native_function_binary_runs_after_sources_are_removed() {
    let temp = temporary();
    let source = temp.join("function-source.nbd");
    let binary = temp.join("independent-function-elf");
    std::fs::write(
        &source,
        include_str!("../examples/bytecode_functions_ai.nbd"),
    )
    .unwrap();
    let compiled = compile("ai", &source, &binary);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    std::fs::remove_file(&source).unwrap();
    let output = Command::new(&binary).output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"13\n6\n");
    std::fs::remove_dir_all(temp).unwrap();
}
