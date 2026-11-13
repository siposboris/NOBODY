//! Python-free regression gate for the legacy AST JSON fixture corpus.
//! This is a Rust exporter smoke/regression test, not proof of Python parity.
use nobody::export_ast_json;
use std::{fs, path::PathBuf};

#[test]
fn legacy_ast_json_fixtures_are_supported_by_native_rust_exporter() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/ast_json_parity");
    let mut fixtures = fs::read_dir(&root)
        .expect("AST JSON fixtures directory must exist")
        .map(|entry| entry.expect("read fixture directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "nbd"))
        .collect::<Vec<_>>();
    fixtures.sort();
    assert_eq!(fixtures.len(), 13, "AST fixture corpus changed: review expected count");

    for fixture in fixtures {
        let filename = fixture.file_name().unwrap().to_string_lossy();
        let profile = filename.split('_').next().expect("profile prefix");
        assert!(matches!(profile, "en" | "sk"), "unexpected fixture profile: {filename}");
        let source = fs::read_to_string(&fixture).expect("read UTF-8 Nobody fixture");
        let first = export_ast_json(&source, profile)
            .unwrap_or_else(|error| panic!("{filename}: Rust AST export failed: {error}"));
        let second = export_ast_json(&source, profile)
            .unwrap_or_else(|error| panic!("{filename}: repeated AST export failed: {error}"));
        assert_eq!(first, second, "{filename}: AST JSON export is not deterministic");
        assert!(
            first.starts_with(r#"{"ast":{"children":["#)
                && first.ends_with(r#","schema":"nobody.ast.v1"}"#),
            "{filename}: malformed or unexpected AST JSON envelope"
        );
    }
}
