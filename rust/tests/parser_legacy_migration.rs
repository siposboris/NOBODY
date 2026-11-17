//! Legacy Python parser regression cases ported to native Rust.
 //! Keep tests/test_parser.py until all consumers are migrated and CI is green.
use nobody::{parse, Expr, Statement};

#[test]
fn legacy_slovak_function_and_condition() {
    let src = "nech cislo = 10\nfunkcia dvoj(x) { vrat x * 2 }\nak cislo > 5 { vypis(dvoj(cislo)) } inak { vypis(0) }";
    let program = parse(src, "sk").unwrap();
    assert_eq!(program.statements.len(), 3);
    assert!(matches!(&program.statements[0], Statement::Let { name, .. } if name == "cislo"));
    assert!(matches!(&program.statements[1], Statement::Function { name, parameters, .. } if name == "dvoj" && parameters == &["x"]));
    assert!(matches!(&program.statements[2], Statement::If { branches, otherwise } if branches.len() == 1 && otherwise.is_some()));
}

#[test]
fn legacy_english_and_czech_conditionals() {
    assert!(matches!(&parse("let n = 1\nif n < 2 { print(n) }", "en").unwrap().statements[1], Statement::If { .. }));
    assert!(matches!(&parse("necht n = 1\npokud pravda { vypis(n) }", "cs").unwrap().statements[1], Statement::If { .. }));
}

#[test]
fn legacy_loop_constructs() {
    let program = parse("pre n v rozsah(4) { vypis(n) }\npokial nepravda { prerus }", "sk").unwrap();
    assert!(matches!(&program.statements[0], Statement::For { .. }));
    assert!(matches!(&program.statements[1], Statement::While { .. }));
}

#[test]
fn legacy_operator_precedence() {
    let program = parse("nech x = 1 + 2 * 3", "sk").unwrap();
    assert!(matches!(&program.statements[0], Statement::Let { expr: Expr::Binary { op, rhs, .. }, .. } if op == "+" && matches!(rhs.as_ref(), Expr::Binary { op, .. } if op == "*")));
}

#[test]
fn legacy_mixed_profile_and_uppercase_rejected() {
    assert!(parse("if pravda { vypis(1) }", "sk").is_err());
    assert!(parse("Vypis(1)", "sk").is_err());
}

#[test]
fn legacy_unbalanced_delimiters_and_invalid_binding() {
    for source in ["ak pravda { vypis(1)", "nech x = (1 + 2", "}", "nech = 2"] {
        assert!(parse(source, "sk").is_err(), "{source}");
    }
}

#[test]
fn legacy_typed_declaration() {
    let program = parse("nech n: cele = 1", "sk").unwrap();
    assert!(matches!(&program.statements[0], Statement::Let { type_name: Some(ty), .. } if ty == "int"));
}
