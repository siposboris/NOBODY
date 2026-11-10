//! End-to-end multi-file Nobody compilation: three source profiles,
//! dependency graph, portable VM, and actual standalone Linux ELF.
use nobody::{evaluate_source, load_project};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn sandbox() -> PathBuf {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "nobody-project-{}-{id}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("start Nobody Rust compiler")
}

fn fixture(root: &Path) -> PathBuf {
    fs::write(
        root.join("math.nbd"),
        "funkcia krat(n: cele): cele { vrat n * 3 }\n",
    )
    .unwrap();
    fs::write(
        root.join("bridge.nbd"),
        "fn next_number(n: int): int { return krat(n) + 1 }\n",
    )
    .unwrap();
    fs::write(
        root.join("main.nbd"),
        "let result = next_number(4)\nprint(result, \"Život ☸\")\n",
    )
    .unwrap();
    let manifest = root.join("project.nbproj");
    fs::write(
        &manifest,
        "nobody.project.v1\n\
module main en main.nbd bridge\n\
module math sk math.nbd -\n\
module bridge ai bridge.nbd math\n\
entry main\n",
    )
    .unwrap();
    manifest
}

#[test]
fn three_files_in_three_profiles_link_in_dependency_order_and_run_bytecode() {
    let dir = sandbox();
    let manifest = fixture(&dir);
    let project = load_project(&manifest).unwrap();
    assert_eq!(project.modules, vec!["math", "bridge", "main"]);
    assert_eq!(evaluate_source(&project.source, "ai").unwrap(), "13 Život ☸\n");
    let manifest = manifest.to_str().unwrap();
    let check = cli(&["check-project", manifest]);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let run = cli(&["run-project", manifest]);
    assert!(run.status.success());
    assert_eq!(run.stdout, b"13 \xc5\xbdivot \xe2\x98\xb8\n");
    let target = dir.join("project.nbc");
    let path = target.to_str().unwrap();
    let compile = cli(&["compile-project-bytecode", manifest, path]);
    assert!(compile.status.success(), "{:?}", compile.stderr);
    let data = fs::read(&target).unwrap();
    assert_eq!(&data[0..8], b"NOBBC001");
    assert!(cli(&["verify-bytecode", path]).status.success());
    let disk = cli(&["run-compiled", path]);
    assert!(disk.status.success(), "{:?}", disk.stderr);
    assert_eq!(disk.stdout, run.stdout);
    // Never silently clobber an existing user artifact.
    assert!(!cli(&["compile-project-bytecode", manifest, path])
        .status
        .success());
    assert_eq!(fs::read(&target).unwrap(), data);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn linked_project_compiles_to_real_standalone_linux_elf() {
    let dir = sandbox();
    let manifest = fixture(&dir);
    let target = dir.join("linked-program");
    let result = cli(&[
        "compile-project-native",
        manifest.to_str().unwrap(),
        target.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(fs::read(&target).unwrap().starts_with(b"\x7fELF"));
    let native = Command::new(&target).output().unwrap();
    let vm = cli(&["run-project", manifest.to_str().unwrap()]);
    assert!(native.status.success(), "{:?}", native.status);
    assert_eq!(native.stdout, vm.stdout);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn graph_errors_unsafe_files_and_cross_module_collisions_fail_closed() {
    let dir = sandbox();
    let manifest = fixture(&dir);
    let base = fs::read_to_string(&manifest).unwrap();
    let invalid = [
        base.replace("module bridge ai bridge.nbd math", "module bridge ai bridge.nbd main"),
        base.replace("module bridge ai bridge.nbd math", "module bridge ai bridge.nbd missing"),
        base.replace("main.nbd bridge", "../escape.nbd bridge"),
        base.replace("entry main", "entry unknown"),
        base.replace("module bridge ai bridge.nbd math", "module bridge ai math.nbd math"),
        format!("{base}module spare ai spare.nbd -\n"),
    ];
    for value in invalid {
        fs::write(&manifest, value).unwrap();
        let err = load_project(&manifest).unwrap_err();
        assert_eq!(err.code, 5003, "{err}");
        assert!(!cli(&["check-project", manifest.to_str().unwrap()])
            .status
            .success());
    }
    fs::write(&manifest, base.replace("module bridge ai bridge.nbd math", "module bridge en bridge.nbd math")).unwrap();
    assert!(load_project(&manifest).is_err(), "wrong syntax profile must fail");
    fs::write(&manifest, &base).unwrap();
    fs::write(
        dir.join("bridge.nbd"),
        "fn krat(n: int): int { return n + 1 }\n",
    )
    .unwrap();
    assert_eq!(load_project(&manifest).unwrap_err().code, 5003);
    fs::write(dir.join("bridge.nbd"), "fn next_number(n: int): int { return krat(n) + 1 }\n").unwrap();
    fs::write(dir.join("main.nbd"), "let result = next_number(4)\nif pravda { vypis(result) }\n").unwrap();
    assert!(load_project(&manifest).is_err(), "mixed English and Slovak source should fail");
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn symlink_outside_project_root_is_forbidden() {
    use std::os::unix::fs::symlink;
    let dir = sandbox();
    let outside = sandbox();
    let manifest = fixture(&dir);
    fs::write(outside.join("hidden.nbd"), "print(999)\n").unwrap();
    fs::remove_file(dir.join("math.nbd")).unwrap();
    symlink(outside.join("hidden.nbd"), dir.join("math.nbd")).unwrap();
    assert_eq!(load_project(&manifest).unwrap_err().code, 5003);
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn project_size_is_bounded_and_bad_manifest_is_never_executed() {
    let dir = sandbox();
    let manifest = fixture(&dir);
    fs::write(dir.join("main.nbd"), format!("let data = \"{}\"\n", "x".repeat(1_048_576))).unwrap();
    assert_eq!(load_project(&manifest).unwrap_err().code, 5003);
    fs::write(&manifest, "not a real project\n").unwrap();
    assert_eq!(load_project(&manifest).unwrap_err().code, 5003);
    fs::remove_dir_all(dir).unwrap();
}
