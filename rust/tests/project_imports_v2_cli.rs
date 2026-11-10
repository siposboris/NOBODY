//! Real source-level imports in Nobody's dependency-aware v2 project compiler.
use nobody::load_project;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn scratch() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("nobody-v2-imports-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("run Nobody compiler")
}
fn setup(root: &Path) -> PathBuf {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/project_imports_v2");
    for file in ["nobody.nbproj", "math.nbd", "bridge.nbd", "main.nbd"] {
        fs::copy(reference.join(file), root.join(file)).unwrap();
    }
    root.join("nobody.nbproj")
}

#[test]
fn actual_import_directives_compile_and_run_vm_and_stored_v3() {
    let root = scratch();
    let manifest = setup(&root);
    let program = load_project(&manifest).unwrap();
    assert_eq!(program.modules, ["math", "bridge", "main"]);
    assert!(!program.source.contains("import bridge"));
    assert!(!program.source.contains("import math"));
    let name = manifest.to_str().unwrap();
    let verify = cli(&["check-project", name]);
    assert!(verify.status.success(), "{}", String::from_utf8_lossy(&verify.stderr));
    let ast = nobody::evaluate_source(&program.source, "ai").unwrap();
    let run = cli(&["run-project", name]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(run.stdout, ast.as_bytes());
    assert_eq!(run.stdout, "13 Život ☸\n".as_bytes());

    let output = root.join("linked.nbc");
    let result = cli(&["compile-project-bytecode", name, output.to_str().unwrap()]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let check = cli(&["verify-bytecode", output.to_str().unwrap()]);
    assert!(check.status.success());
    let disk = cli(&["run-compiled", output.to_str().unwrap()]);
    assert!(disk.status.success());
    assert_eq!(disk.stdout, run.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn real_source_imports_link_to_independent_native_linux_elf() {
    let root = scratch();
    let manifest = setup(&root);
    let target = root.join("native");
    let compile = cli(&[
        "compile-project-native",
        manifest.to_str().unwrap(),
        target.to_str().unwrap(),
    ]);
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    assert!(fs::read(&target).unwrap().starts_with(b"\x7fELF"));
    let executable = Command::new(&target).output().unwrap();
    let interpreted = cli(&["run-project", manifest.to_str().unwrap()]);
    assert!(executable.status.success());
    assert_eq!(executable.stdout, interpreted.stdout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn import_directives_must_match_manifest_graph_and_precede_executable_code() {
    let root = scratch();
    let manifest = setup(&root);
    let path = root.join("main.nbd");
    let normal = fs::read_to_string(&path).unwrap();
    for source in [
        normal.replace("import bridge\n", ""),
        normal.replace("import bridge\n", "import math\nimport bridge\n"),
        normal.replace("import bridge\n", "import bridge\nimport bridge\n"),
        normal.replace("import bridge\n", "import bridge,math\n"),
        normal.replace("import bridge\n", "let x = 3\nimport bridge\n"),
        normal.replace("import bridge\n", "# import bridge\n"),
        normal.replace("import bridge\n", "importuj bridge\n"),
    ] {
        fs::write(&path, source).unwrap();
        assert!(
            load_project(&manifest).is_err(),
            "Invalid module import escaped project validation"
        );
    }
    fs::write(&path, normal).unwrap();
    let old = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, old.replace("nobody.project.v2", "nobody.project.v1")).unwrap();
    assert_eq!(load_project(&manifest).unwrap_err().code, 5003);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn transitive_dependencies_cannot_be_used_without_direct_import() {
    let root = scratch();
    let manifest = setup(&root);
    let path = root.join("main.nbd");
    let valid = fs::read_to_string(&path).unwrap();
    fs::write(&path, "import bridge\nprint(krat(3))\n").unwrap();
    let error = load_project(&manifest).unwrap_err();
    assert_eq!(error.code, 5003);
    assert!(error.detail.contains("without a direct import"), "{error}");
    fs::write(&path, valid).unwrap();
    assert!(load_project(&manifest).is_ok());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn import_mentions_inside_strings_or_comments_do_not_create_dependencies() {
    let root = scratch();
    let manifest = setup(&root);
    let path = root.join("main.nbd");
    fs::write(
        &path,
        "import bridge # math is a dependency of bridge\nlet result = next_number(4)\nprint(result, \"import math\")\n",
    )
    .unwrap();
    assert!(load_project(&manifest).is_ok());
    fs::remove_dir_all(root).unwrap();
}
