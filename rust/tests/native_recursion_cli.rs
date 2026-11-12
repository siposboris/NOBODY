//! End-to-end native recursion: real ELF, no embedded VM or Python.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::process::Command;

#[test]
fn typed_factorial_runs_in_standalone_elf() {
    let dir = std::env::temp_dir().join(format!("nobody-recursion-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("factorial.nbd");
    let binary = dir.join("factorial");
    std::fs::write(&source, "fn factorial(n: int): int { if n <= 1 { return 1 } else { return n * factorial(n - 1) } }\nprint(factorial(5))\n").unwrap();
    let compiler = env!("CARGO_BIN_EXE_nobody");
    let built = Command::new(compiler).args(["compile-native", "ai", source.to_str().unwrap(), binary.to_str().unwrap()]).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{native:?}");
    assert_eq!(native.stdout, b"120\n");
    let evaluated = Command::new(compiler).args(["run", "ai", source.to_str().unwrap()]).output().unwrap();
    assert!(evaluated.status.success(), "{evaluated:?}");
    assert_eq!(native.stdout, evaluated.stdout);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unbounded_native_self_call_exits_on_depth_budget() {
    let dir = std::env::temp_dir().join(format!("nobody-recursion-limit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("loop.nbd");
    let binary = dir.join("loop");
    std::fs::write(&source, "fn loop(n: int): int { return loop(n) }\nprint(loop(1))\n").unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_nobody")).args(["compile-native", "ai", source.to_str().unwrap(), binary.to_str().unwrap()]).output().unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let run = Command::new(&binary).output().unwrap();
    assert_eq!(run.status.code(), Some(5));
    std::fs::remove_dir_all(dir).unwrap();
}
