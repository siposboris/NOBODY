//! The Rust Nobody CLI replaces the Python direction subcommand.
use std::process::Command;

fn invoke(profile: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody"))
        .args(["direction", profile])
        .output()
        .expect("run native Nobody direction")
}

#[test]
fn all_four_rtl_profiles_have_rtl_editor_guidance() {
    for code in ["ar", "ur", "he", "fa"] {
        let output = invoke(code);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.starts_with("rtl\n"), "{code}");
        assert!(stdout.contains("logical character order"), "{code}");
    }
}

#[test]
fn other_profiles_remain_left_to_right() {
    for code in ["sk", "en", "ai", "ja", "zh"] {
        let output = invoke(code);
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.starts_with("ltr\n"), "{code}");
    }
}

#[test]
fn unknown_direction_is_numbered_error() {
    let output = invoke("nonexistent");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("N3007"));
}
