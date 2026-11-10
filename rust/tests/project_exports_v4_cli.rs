//! End-to-end project v4 export visibility and private-name isolation.
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
    let root = std::env::temp_dir().join(format!("nobody-v4-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn setup(root: &Path) -> PathBuf {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/project_namespaces_v3");
    for file in ["nobody.nbproj", "math.nbd", "extra.nbd", "main.nbd"] {
        fs::copy(base.join(file), root.join(file)).unwrap();
    }
    // v4 has no implicit public symbols. Declare the intended API explicitly.
    let manifest = root.join("nobody.nbproj");
    let old = fs::read_to_string(&manifest).unwrap();
    let updated = old
        .replace("nobody.project.v3", "nobody.project.v4")
        .replace("entry main", "export math step,doubled\nexport extra step\nentry main");
    fs::write(&manifest, updated).unwrap();

    // The same non-exported name exists in two different modules, and a
    // public function calls its own private helper.
    let math = root.join("math.nbd");
    let source = fs::read_to_string(&math).unwrap().replace(
        "vrat step(n) + step(n)",
        "vrat step(secret(n)) + step(n)",
    );
    fs::write(&math, format!("{source}\nfunkcia secret(n: cele): cele {{ vrat n + 1 }}\n"))
        .unwrap();
    let extra = root.join("extra.nbd");
    let source = fs::read_to_string(&extra).unwrap();
    fs::write(
        &extra,
        format!("{source}\nfunction secret(n: int): int {{ return n + 100 }}\n"),
    )
    .unwrap();
    manifest
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn project_v4_private_symbols_do_not_collide_and_self_calls_work_in_all_backends() {
    let root = scratch();
    let manifest = setup(&root);
    let project = load_project(&manifest).unwrap();
    assert_eq!(project.modules, ["math", "extra", "main"]);
    let expected = "12 9 15 Život ☸\n";
    assert_eq!(evaluate_source(&project.source, "ai").unwrap(), expected);
    assert!(!project.source.contains("math.secret"));
    assert!(!project.source.contains("extra.secret"));
    assert!(project.source.contains("nobody_ns_math_x736563726574"));
    assert!(project.source.contains("nobody_ns_extra_x736563726574"));
    let manifest = manifest.to_str().unwrap();
    assert!(cli(&["check-project", manifest]).status.success());
    let live = cli(&["run-project", manifest]);
    assert!(live.status.success(), "{}", String::from_utf8_lossy(&live.stderr));
    assert_eq!(live.stdout, expected.as_bytes());
    let path = root.join("public-api.nbc");
    let output = path.to_str().unwrap();
    let built = cli(&["compile-project-bytecode", manifest, output]);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    assert!(cli(&["verify-bytecode", output]).status.success());
    let stored = cli(&["run-compiled", output]);
    assert!(stored.status.success());
    assert_eq!(stored.stdout, expected.as_bytes());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn project_v4_exports_compile_to_real_standalone_elf_with_output_parity() {
    let root = scratch();
    let manifest = setup(&root);
    let binary = root.join("v4-native");
    let built = cli(&[
        "compile-project-native",
        manifest.to_str().unwrap(),
        binary.to_str().unwrap(),
    ]);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    assert!(fs::read(&binary).unwrap().starts_with(b"\x7fELF"));
    let native = Command::new(&binary).output().unwrap();
    assert!(native.status.success(), "{}", String::from_utf8_lossy(&native.stderr));
    let reference = cli(&["run-project", manifest.to_str().unwrap()]);
    assert_eq!(native.stdout, reference.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn v4_rejects_private_member_access_even_when_its_module_is_imported() {
    let root = scratch();
    let manifest = setup(&root);
    let file = root.join("main.nbd");
    let original = fs::read_to_string(&file).unwrap();
    for invalid in [
        original.replace("math.step(4)", "math.secret(4)"),
        original.replace("extra.step(4)", "extra.secret(4)"),
    ] {
        fs::write(&file, invalid).unwrap();
        let err = load_project(&manifest).unwrap_err();
        assert_eq!(err.code, 5003, "{err}");
        assert!(err.detail.contains("inaccessible or missing export"), "{err}");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn v4_export_contracts_reject_unknown_duplicate_and_unlisted_public_members() {
    let root = scratch();
    let manifest = setup(&root);
    let original = fs::read_to_string(&manifest).unwrap();
    for invalid in [
        original.replace("export math step,doubled", "export math step,missing"),
        original.replace("export math step,doubled", "export unlisted step,doubled"),
        original.replace("export math step,doubled", "export math step,step"),
        original.replace("export math step,doubled", "export math step,"),
        original.replace("entry main", "export math step\nentry main"),
        original.replace("export math step,doubled", "export math secret"),
    ] {
        fs::write(&manifest, invalid).unwrap();
        let err = load_project(&manifest).unwrap_err();
        assert_eq!(err.code, 5003, "{err}");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn all_40_profiles_can_use_v4_explicit_exports_without_syntax_mixing() {
    let root = scratch();
    let manifest = setup(&root);
    let original = fs::read_to_string(&manifest).unwrap();
    for profile in nobody::profiles::PROFILES {
        let import = nobody::profiles::word(profile.code, "import").unwrap();
        let print = nobody::profiles::word(profile.code, "print").unwrap();
        fs::write(
            root.join("main.nbd"),
            format!("{import} math\n{import} extra\n{print}(math.step(2), extra.step(2))\n"),
        )
        .unwrap();
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
        let result = nobody::execute_bytecode_source(&project.source, "ai")
            .unwrap_or_else(|err| panic!("{}: {err}", profile.code));
        assert_eq!(result, "6 7\n", "{}", profile.code);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn version_3_preserves_implicit_exports_for_existing_projects() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/project_namespaces_v3/nobody.nbproj");
    let project = load_project(&manifest).unwrap();
    assert_eq!(
        evaluate_source(&project.source, "ai").unwrap(),
        "12 9 12 Život ☸\n"
    );
}
