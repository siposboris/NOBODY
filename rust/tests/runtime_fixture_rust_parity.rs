//! Python-free runtime regression coverage for the six historical differential fixtures.
//! The legacy Python-vs-Rust comparison remains in CI until its reference semantics
//! are independently accounted for; this test exercises both Rust execution engines.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn historical_runtime_fixtures_agree_between_rust_engines() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixtures_dir = root.join("tests/fixtures/runtime_parity");
    let mut fixtures: Vec<_> = fs::read_dir(&fixtures_dir)
        .expect("runtime parity fixtures directory must exist")
        .map(|entry| entry.expect("fixture directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "nbd"))
        .collect();
    fixtures.sort();
    assert_eq!(
        fixtures.len(),
        6,
        "historical runtime parity fixture count changed"
    );

    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    for fixture in fixtures {
        let filename = fixture.file_name().unwrap().to_string_lossy();
        let profile = filename.split('_').next().expect("profile prefix");
        let execute = |mode: &str| {
            Command::new(executable)
                .current_dir(&root)
                .arg(mode)
                .arg(profile)
                .arg(&fixture)
                .output()
                .unwrap_or_else(|error| panic!("{mode} failed to launch for {filename}: {error}"))
        };
        let evaluator = execute("run");
        let bytecode = execute("run-bytecode");
        assert!(
            evaluator.status.success(),
            "Rust evaluator failed on {filename}: {}",
            String::from_utf8_lossy(&evaluator.stderr)
        );
        assert!(
            bytecode.status.success(),
            "Rust bytecode VM failed on {filename}: {}",
            String::from_utf8_lossy(&bytecode.stderr)
        );
        assert_eq!(
            evaluator.stdout, bytecode.stdout,
            "Rust evaluator/bytecode output differs for {filename}"
        );
    }
}
