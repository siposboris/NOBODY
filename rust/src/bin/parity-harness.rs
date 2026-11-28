//! Rust regression harness against frozen historical reference outputs.
use std::io::Read;
use std::process::Stdio;
use std::thread;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

fn drain<R: Read + Send + 'static>(mut stream: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn run(mut command: Command) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    // Drain both pipes concurrently while the process runs. Waiting before reading
    // deadlocks when either pipe fills its OS buffer.
    let stdout = drain(child.stdout.take().ok_or("missing stdout pipe")?);
    let stderr = drain(child.stderr.take().ok_or("missing stderr pipe")?);
    let start = std::time::Instant::now();
    let (status, timed_out) = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break (status, false),
            None if start.elapsed() >= Duration::from_secs(15) => {
                child.kill().map_err(|e| e.to_string())?;
                let status = child.wait().map_err(|e| e.to_string())?;
                break (status, true);
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    let stdout = stdout
        .join()
        .map_err(|_| "stdout reader panicked")?
        .map_err(|e| e.to_string())?;
    let stderr = stderr
        .join()
        .map_err(|_| "stderr reader panicked")?
        .map_err(|e| e.to_string())?;
    if timed_out {
        return Err("process timed out after 15 seconds".into());
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn equal(reference: &[u8], rust: &Output) -> bool {
    rust.status.success() && reference == rust.stdout.as_slice()
}

fn fixtures(dir: &Path, expected: usize) -> Result<Vec<PathBuf>, String> {
    let mut files = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .map(|entry| entry.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|p| p.extension().is_some_and(|ext| ext == "nbd"));
    files.sort();
    if files.len() != expected {
        return Err(format!(
            "expected {expected} fixtures, found {}",
            files.len()
        ));
    }
    Ok(files)
}

fn main() -> Result<(), String> {
    let mode = env::args()
        .nth(1)
        .ok_or("usage: cargo run --bin parity-harness -- runtime|ast")?;
    let (folder, expected, bin, rust_action) = match mode.as_str() {
        "runtime" => ("runtime_parity", 6, "nobody-rs", "run"),
        "ast" => ("ast_json_parity", 13, "nobody", "ast-json"),
        _ => return Err("expected runtime or ast".into()),
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repo root")?
        .to_path_buf();
    let executable = root.join("rust/target/debug").join(bin);
    if !executable.is_file() {
        return Err(format!("missing Rust executable: {}", executable.display()));
    }
    let mut failures = 0;
    for fixture in fixtures(&root.join("tests/fixtures").join(folder), expected)? {
        let name = fixture.file_name().unwrap().to_string_lossy();
        let profile = name.split('_').next().ok_or("missing language profile")?;
        let reference = fs::read(fixture.with_extension("expected"))
            .map_err(|e| format!("{name}: missing reference output: {e}"))?;
        let mut rust = Command::new(&executable);
        rust.current_dir(&root)
            .arg(rust_action)
            .arg(profile)
            .arg(&fixture);
        let native = run(rust)?;
        if equal(&reference, &native) {
            println!("PASS {name}");
        } else {
            failures += 1;
            eprintln!(
                "FAIL {name}\n  Expected: {:?}\n  Rust: status={} stdout={:?} stderr={:?}",
                reference, native.status, native.stdout, native.stderr
            );
        }
    }
    if failures > 0 {
        return Err(format!("{failures}/{expected} fixtures disagree"));
    }
    println!("{mode} reference regression passed for {expected} fixtures");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_fixture_count() {
        let dir = env::temp_dir().join(format!("nobody-parity-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert!(fixtures(&dir, 1).unwrap_err().contains("found 0"));
        fs::remove_dir(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn drains_large_stdout_and_stderr_without_deadlock() {
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "head -c 262144 /dev/zero; head -c 262144 /dev/zero >&2",
        ]);
        let output = run(command).unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 262144);
        assert_eq!(output.stderr.len(), 262144);
    }

    #[test]
    fn rejects_failed_execution() {
        let failed = Command::new("false").output().unwrap();
        assert!(!equal(b"", &failed));
    }
}
