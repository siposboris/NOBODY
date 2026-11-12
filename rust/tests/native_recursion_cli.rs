//! End-to-end native recursion: real ELF, no embedded VM or Python.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::process::Command;

fn build(source_text: &str, name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("nobody-recursion-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("source.nbd");
    let binary = dir.join("program");
    std::fs::write(&source, source_text).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args([
            "compile-native",
            "ai",
            source.to_str().unwrap(),
            binary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    (dir, binary)
}

#[test]
fn typed_factorial_runs_in_standalone_elf() {
    let source = "fn factorial(n: int): int { if n <= 1 { return 1 } else { return n * factorial(n - 1) } }\nprint(factorial(5))\n";
    let (dir, binary) = build(source, "factorial");
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{native:?}");
    assert_eq!(native.stdout, b"120\n");

    let source_path = dir.join("source.nbd");
    let evaluated = Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(["run", "ai", source_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(evaluated.status.success(), "{evaluated:?}");
    assert_eq!(native.stdout, evaluated.stdout);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unbounded_native_self_call_exits_on_depth_budget() {
    let source = "fn recurse(n: int): int { return recurse(n) }\nprint(recurse(1))\n";
    let (dir, binary) = build(source, "limit");
    let run = Command::new(&binary).output().unwrap();
    assert_eq!(run.status.code(), Some(5));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn branching_fibonacci_matches_evaluator_in_standalone_elf() {
    // Two recursive calls in one expression exercise independent native
    // call frames and preservation of the first result across the second call.
    let source = "fn fib(n: int): int { if n <= 1 { return n } else { return fib(n - 1) + fib(n - 2) } }\nprint(fib(6))\n";
    let (dir, binary) = build(source, "fibonacci");
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{native:?}");
    assert_eq!(native.stdout, b"8\n");

    let source_path = dir.join("source.nbd");
    let evaluated = Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(["run", "ai", source_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(evaluated.status.success(), "{evaluated:?}");
    assert_eq!(native.stdout, evaluated.stdout);
    std::fs::remove_dir_all(dir).unwrap();
}
