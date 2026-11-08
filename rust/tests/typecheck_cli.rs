use std::{path::Path,process::Command};
fn run(args:&[&str])->std::process::Output{
    Command::new(env!("CARGO_BIN_EXE_nobody-rs")).args(args).output().expect("Rust binary built")
}
fn fixture(name:&str)->String{
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(name).to_string_lossy().into_owned()
}
#[test]fn valid_ai_program_passes(){
    let file=fixture("typecheck_good_ai.nbd");
    let out=run(&["typecheck","ai",&file]);
    assert!(out.status.success(),"{}",String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("known types only"));
}
#[test]fn bad_ai_initializer_reports_n1001(){
    let file=fixture("typecheck_bad_ai.nbd");
    let out=run(&["typecheck","ai",&file]);
    assert!(!out.status.success());
    let stderr=String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("N1001"),"{stderr}");
    assert!(stderr.contains("count"),"{stderr}");
}
