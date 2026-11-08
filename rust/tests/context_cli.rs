use std::process::Command;
use std::path::Path;

fn invoke(args:&[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nobody-rs"))
        .args(args)
        .output()
        .expect("nobody-rs integration binary must exist")
}
fn fixture(name:&str)->String {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(name)
        .to_string_lossy().into_owned()
}

#[test]
fn valid_sk_context_is_accepted() {
    let file=fixture("basic_sk.nbd");
    let output=invoke(&["validate","sk",&file]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("validation passed"));
}
#[test]
fn top_level_return_reports_n3008() {
    let file=fixture("invalid_return_sk.nbd");
    let output=invoke(&["validate","sk",&file]);
    assert!(!output.status.success());
    let err=String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("N3008"),"stderr: {err}");
    assert!(err.contains("Move the return"),"stderr: {err}");
    assert!(!err.contains("riadok 0"),"no fabricated source location");
}
#[test]
fn top_level_break_reports_n3009() {
    let file=fixture("invalid_break_ai.nbd");
    let output=invoke(&["validate","ai",&file]);
    assert!(!output.status.success());
    let err=String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("N3009"),"stderr: {err}");
    assert!(err.contains("while/for loop"),"stderr: {err}");
}
#[test]
fn parse_does_not_claim_to_run_or_validate() {
    let file=fixture("invalid_return_sk.nbd");
    let output=invoke(&["parse","sk",&file]);
    assert!(output.status.success(),"parse should build an AST even for contextually invalid return");
}

#[test]
fn translate_sk_to_readable_ai_cli() {
    let file=fixture("basic_sk.nbd");
    let output=invoke(&["translate","sk","ai",&file]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    let text=String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("let vek: int = 30"),"translated: {text}");
    assert!(text.contains("print(vek)"),"translated: {text}");
}
#[test]
fn migration_cli_does_not_modify_text_literals() {
    use std::fs;
    let repo=Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let file=repo.join("examples/hello_ai_legacy.nbd");
    let before=fs::read_to_string(&file).unwrap();
    let output=invoke(&["migrate-ai",&file.to_string_lossy()]);
    assert!(output.status.success(),"stderr: {}",String::from_utf8_lossy(&output.stderr));
    let rewritten=String::from_utf8_lossy(&output.stdout);
    assert!(rewritten.contains("let pocet"),"result: {rewritten}");
    assert!(rewritten.contains("if pocet"),"result: {rewritten}");
    assert_eq!(fs::read_to_string(file).unwrap(),before,"the command must not overwrite source");
}
