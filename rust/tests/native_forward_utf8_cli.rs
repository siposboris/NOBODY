//! Forward-declared UTF-8 functions must agree across native ELF and Rust evaluation.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::process::Command;

#[test]
fn forward_typed_utf8_return_matches_evaluator_in_standalone_elf() {
    let dir = std::env::temp_dir().join(format!(
        "nobody-forward-utf8-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("source.nbd");
    let binary = dir.join("program");
    // The callee is deliberately declared after its caller. Non-ASCII bytes
    // also exercise the native string pointer/length return ABI.
    std::fs::write(
        &source,
        "fn greeting(): str { return later() }\nfn later(): str { return \"ahoj 🌍\" }\nprint(greeting())\n",
    )
    .unwrap();
    let cli = env!("CARGO_BIN_EXE_nobody");
    let compile = Command::new(cli)
        .args([
            "compile-native",
            "ai",
            source.to_str().unwrap(),
            binary.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "native compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{native:?}");
    assert_eq!(native.stdout, "ahoj 🌍\n".as_bytes());
    let evaluated = Command::new(cli)
        .args(["run", "ai", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(evaluated.status.success(), "{evaluated:?}");
    assert_eq!(native.stdout, evaluated.stdout);
    std::fs::remove_dir_all(dir).unwrap();
}
