//! Conservative, overflow-checked integer constant expressions for N1008.
use nobody::check_source;

#[test]
fn catches_zero_from_exact_integer_arithmetic() {
    for source in [
        "let x = 10 / (5 - 5)",
        "let x = 10 // (2 * 0)",
        "let x = 10 % (4 - (2 + 2))",
        "let x = 10 / (3 + -3)",
        "let x = 10 / (1_000 - 1000)",
        "let x = 10 / (0 * 999)",
    ] {
        let err = check_source(source, "ai").unwrap_err();
        assert_eq!(err.code, 1008, "{source}");
    }
}

#[test]
fn nonzero_exact_values_remain_valid() {
    for source in [
        "let x = 10 / (5 - 4)",
        "let x = 10 / (2 * 3)",
        "let x = 10 / (3 + -2)",
    ] {
        check_source(source, "ai").unwrap();
    }
}

#[test]
fn unknown_dynamic_or_unmodeled_constants_are_deferred() {
    for source in [
        "let x = 10 / (a - a)",
        "let x = 10 / (2 / 2 - 1)",
        "let x = 10 / (f(1) - 1)",
        "let x = 10 / (1e-99999 - 1e-99999)",
        "let x = 10 / (0.1 - 0.1)",
    ] {
        check_source(source, "ai").unwrap();
    }
}

#[test]
fn overflowed_integer_constant_never_wraps_to_zero() {
    check_source(
        "let x = 10 / (170141183460469231731687303715884105727 + 1)",
        "ai",
    )
    .unwrap();
}

#[test]
fn original_literal_zero_checks_still_apply() {
    for source in ["let x = 3 / 0e20", "let x = 3 / -0.0"] {
        assert_eq!(check_source(source, "ai").unwrap_err().code, 1008);
    }
}

#[test]
fn detects_zero_in_slovak_profile_too() {
    assert_eq!(
        check_source("nech vysledok = 10 / (5 - 5)", "sk")
            .unwrap_err()
            .code,
        1008
    );
}
