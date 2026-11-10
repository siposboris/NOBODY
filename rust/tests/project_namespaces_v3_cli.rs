//! End-to-end compiler acceptance: v3 module namespaces and imports.
use nobody::{evaluate_source, load_project};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn scratch() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nobody-v3-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}
fn setup(root: &Path) -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/project_namespaces_v3");
    for path in ["nobody.nbproj", "math.nbd", "extra.nbd", "main.nbd"] {
        fs::copy(base.join(path), root.join(path)).unwrap();
    }
    root.join("nobody.nbproj")
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn real_v3_namespaces_allow_duplicate_function_names_and_execute_stored_bytecode() {
    let root = scratch();
    let manifest = setup(&root);
    let project = load_project(&manifest).unwrap();
    assert_eq!(project.modules, ["math", "extra", "main"]);
    assert!(!project.source.contains("math.step"));
    assert!(!project.source.contains("extra.step"));
    assert!(!project.source.contains("import math"));
    let expected = "12 9 12 Život ☸\n";
    assert_eq!(evaluate_source(&project.source, "ai").unwrap(), expected);
    let name = manifest.to_str().unwrap();
    let verified = run(&["check-project", name]);
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let live = run(&["run-project", name]);
    assert!(
        live.status.success(),
        "{}",
        String::from_utf8_lossy(&live.stderr)
    );
    assert_eq!(live.stdout, expected.as_bytes());

    let target = root.join("linked.nbc");
    let dest = target.to_str().unwrap();
    let built = run(&["compile-project-bytecode", name, dest]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(run(&["verify-bytecode", dest]).status.success());
    let stored = run(&["run-compiled", dest]);
    assert!(stored.status.success());
    assert_eq!(stored.stdout, live.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn two_modules_exporting_step_compile_into_real_independent_elf() {
    let root = scratch();
    let manifest = setup(&root);
    let target = root.join("namespaced-native");
    let built = run(&[
        "compile-project-native",
        manifest.to_str().unwrap(),
        target.to_str().unwrap(),
    ]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(fs::read(&target).unwrap().starts_with(b"\x7fELF"));
    let native = Command::new(&target).output().unwrap();
    let reference = run(&["run-project", manifest.to_str().unwrap()]);
    assert!(native.status.success(), "{:?}", native.status);
    assert_eq!(native.stdout, reference.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn v3_requires_qualified_imported_symbols_and_real_exports() {
    let root = scratch();
    let manifest = setup(&root);
    let main = root.join("main.nbd");
    for source in [
        "import math\nimport extra\nprint(step(3))\n",
        "import math\nimport extra\nprint(math.missing(3))\n",
        "import math\nimport extra\nprint(unlisted.step(3))\n",
        "import extra\nprint(math.step(3))\n",
        "import math\nimport extra\nprint(nobody_ns_math_x73746570(3))\n",
        "import math\nimport extra\nprint(math.step(3), extra.unknown(4))\n",
    ] {
        fs::write(&main, source).unwrap();
        let err = load_project(&manifest).unwrap_err();
        assert_eq!(err.code, 5003, "{err}");
        assert!(!run(&["check-project", manifest.to_str().unwrap()])
            .status
            .success());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn all_40_profiles_can_import_qualified_modules_in_v3_build() {
    let root = scratch();
    let manifest = setup(&root);
    let original = fs::read_to_string(&manifest).unwrap();
    for profile in nobody::profiles::PROFILES {
        let syntax_import = nobody::profiles::word(profile.code, "import").unwrap();
        let syntax_print = nobody::profiles::word(profile.code, "print").unwrap();
        let source = format!(
            "{syntax_import} math\n{syntax_import} extra\n{syntax_print}(math.step(2), extra.step(2))\n"
        );
        fs::write(root.join("main.nbd"), source).unwrap();
        fs::write(
            &manifest,
            original.replace(
                "module main ai main.nbd",
                &format!("module main {} main.nbd", profile.code),
            ),
        )
        .unwrap();
        let project =
            load_project(&manifest).unwrap_or_else(|err| panic!("{}: {err}", profile.code));
        let actual = nobody::execute_bytecode_source(&project.source, "ai")
            .unwrap_or_else(|err| panic!("{}: {err}", profile.code));
        assert_eq!(actual, "6 7\n", "{}", profile.code);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn original_v1_and_v2_compilers_remain_compatible() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    for name in ["project_multifile", "project_imports_v2"] {
        let manifest = base.join(name).join("nobody.nbproj");
        let project = load_project(&manifest).unwrap();
        assert_eq!(
            evaluate_source(&project.source, "ai").unwrap(),
            "13 Život ☸\n"
        );
    }
}
