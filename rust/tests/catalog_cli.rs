//! Python-free end-to-end coverage for the historical 249-entry catalogue CLI.
use std::process::Command;

#[test]
fn rust_catalog_lists_all_reference_variants_without_enabling_unimplemented_syntax() {
    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    let text = Command::new(executable)
        .arg("catalog")
        .output()
        .expect("run catalog");
    assert!(
        text.status.success(),
        "{}",
        String::from_utf8_lossy(&text.stderr)
    );
    let lines = String::from_utf8(text.stdout).expect("UTF-8 output");
    assert_eq!(lines.lines().count(), 249);
    assert!(lines.contains("Slovak: experimental sk"));
    assert!(lines.contains("Chinese (Traditional): planned"));

    let json = Command::new(executable)
        .args(["catalog", "--json"])
        .output()
        .expect("run JSON catalog");
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let stdout = String::from_utf8(json.stdout).expect("UTF-8 JSON");
    assert!(stdout.starts_with("{\n"));
    assert!(stdout.contains("\"reference_variants\": 249"));
    assert!(stdout.contains("\"active_experimental_profiles\": 40"));
    assert!(stdout.contains("\"planned_reference_variants\": 210"));
    assert_eq!(stdout.matches("\"syntax_code\":").count(), 249);
}

#[test]
fn rust_catalog_rejects_unrecognized_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(["catalog", "--unknown"])
        .output()
        .expect("run CLI");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn python_languages_command_is_served_by_rust_without_python_runtime() {
    let executable = env!("CARGO_BIN_EXE_nobody-rs");
    for args in [vec!["languages"], vec!["languages", "--json"]] {
        let legacy = Command::new(executable)
            .args(&args)
            .output()
            .expect("run Python-compatible languages command");
        let canonical = Command::new(executable)
            .arg("catalog")
            .args(&args[1..])
            .output()
            .expect("run Rust catalog command");
        assert!(legacy.status.success(), "{:?}", legacy.stderr);
        assert_eq!(legacy.stdout, canonical.stdout);
    }

    let active = Command::new(executable)
        .arg("list-languages")
        .output()
        .expect("run active profile listing");
    assert!(active.status.success());
    assert_eq!(String::from_utf8(active.stdout).unwrap().lines().count(), 40);
}
