//! Execute a real standalone ELF for inferred UTF-8 function parameters.
//! This checks runtime semantics, not just generated assembly text.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn native_text_equality_parameters_execute_in_elf() {
    use nobody::native_x86_64::compile_native;
    use std::process::Command;

    let source = "fn matches(word) { return word == \"héllo\" }\nprint(matches(\"héllo\"))\nprint(matches(\"other\"))";
    let directory = std::env::temp_dir().join(format!(
        "nobody-native-text-equality-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir(&directory).expect("create isolated ELF test directory");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(directory.clone());
    let executable = directory.join("program");
    compile_native(source, "ai", &executable).expect("compile standalone ELF");
    let output = Command::new(&executable)
        .output()
        .expect("execute standalone ELF");
    assert!(output.status.success(), "ELF failed: {:?}", output.status);
    assert_eq!(output.stdout, b"true\nfalse\n");
    assert!(output.stderr.is_empty());
}
