//! Transitional differential harness: Rust owns fixture discovery, invocation and comparison.
//! The Python reference remains test-only until checked-in golden outputs replace it.
use std::{env,fs,path::{Path,PathBuf},process::{Command,Output},time::Duration};
use std::process::Stdio;
use std::thread;

fn run(mut command: Command) -> Result<Output,String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child=command.spawn().map_err(|e|e.to_string())?;
    let start=std::time::Instant::now();
    loop {
        match child.try_wait().map_err(|e|e.to_string())? {
            Some(_) => return child.wait_with_output().map_err(|e|e.to_string()),
            None if start.elapsed()>Duration::from_secs(15) => {
                let _=child.kill();
                let _=child.wait();
                return Err("process timed out after 15 seconds".into());
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}
fn equal(reference:&Output,rust:&Output)->bool {
    reference.status.success() && rust.status.success() && reference.stdout==rust.stdout
}
fn fixtures(dir:&Path,expected:usize)->Result<Vec<PathBuf>,String>{
    let mut files=fs::read_dir(dir).map_err(|e|e.to_string())?
        .map(|entry|entry.map(|e|e.path()).map_err(|e|e.to_string()))
        .collect::<Result<Vec<_>,_>>()?;
    files.retain(|p|p.extension().is_some_and(|ext|ext=="nbd"));
    files.sort();
    if files.len()!=expected {return Err(format!("expected {expected} fixtures, found {}",files.len()));}
    Ok(files)
}
fn main()->Result<(),String>{
    let mode=env::args().nth(1).ok_or("usage: cargo run --bin parity-harness -- runtime|ast")?;
    let (folder,expected,bin,python_action,rust_action)=match mode.as_str(){
        "runtime"=>("runtime_parity",6,"nobody-rs","ast-run","run"),
        "ast"=>("ast_json_parity",13,"nobody","ast-json","ast-json"),
        _=>return Err("expected runtime or ast".into()),
    };
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().ok_or("missing repo root")?.to_path_buf();
    let executable=root.join("rust/target/debug").join(bin);
    if !executable.is_file(){return Err(format!("missing Rust executable: {}",executable.display()));}
    let mut failures=0;
    for fixture in fixtures(&root.join("tests/fixtures").join(folder),expected)? {
        let name=fixture.file_name().unwrap().to_string_lossy();
        let profile=name.split('_').next().ok_or("missing language profile")?;
        let mut python=Command::new("python3");
        python.current_dir(&root).env("PYTHONPATH",&root)
            .args(["-m","nobody",python_action])
            .arg(&fixture).args(["--lang",profile]);
        let mut rust=Command::new(&executable);
        rust.current_dir(&root).arg(rust_action).arg(profile).arg(&fixture);
        let reference=run(python)?;
        let native=run(rust)?;
        if equal(&reference,&native){println!("PASS {name}");}
        else {
            failures+=1;
            eprintln!("FAIL {name}\n  Python: status={} stdout={:?} stderr={:?}\n  Rust: status={} stdout={:?} stderr={:?}",
                reference.status,reference.stdout,reference.stderr,native.status,native.stdout,native.stderr);
        }
    }
    if failures>0 {return Err(format!("{failures}/{expected} fixtures disagree"));}
    println!("{mode} differential parity passed for {expected} fixtures");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_wrong_fixture_count(){
        let dir=env::temp_dir().join(format!("nobody-parity-{}",std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert!(fixtures(&dir,1).unwrap_err().contains("found 0"));
        fs::remove_dir(&dir).unwrap();
    }
    #[test] fn rejects_failed_reference(){
        let failed=Command::new("false").output().unwrap();
        let succeeded=Command::new("true").output().unwrap();
        assert!(!equal(&failed,&succeeded));
    }
}
