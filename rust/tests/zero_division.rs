//! Narrow N1008 guard against provably zero constant divisors.
//! This is not general constant folding or a runtime execution guarantee.
use nobody::check_source;

#[test]
fn rejects_zero_division_in_all_three_operators() {
    for source in ["let x = 7 / 0", "let x = 7 // 0", "let x = 7 % 0"] {
        let err = check_source(source, "ai").unwrap_err();
        assert_eq!(err.code, 1008, "{source}");
        assert!(err.detail.contains("zero constant divisor"));
    }
}

#[test]
fn rejects_zero_exponents_and_grouping_in_divisor() {
    for source in [
        "let x = 2 / 0.0",
        "let x = 2 / 0e20",
        "let x = 2 / 0.00E-9",
        "let x = 2 / 0_0",
        "let x = 2 / (0)",
        "let x = 2 / -0",
        "let x = 2 / +0.0",
    ] {
        assert_eq!(
            check_source(source, "ai").unwrap_err().code,
            1008,
            "{source}"
        );
    }
}

#[test]
fn preserves_nonzero_mantissa_even_with_extreme_exponent() {
    for source in [
        "let x = 2 / 1e-9999",
        "let x = 2 / 0.001",
        "let x = 2 / 1_000",
        "let x = 2 / -2",
    ] {
        check_source(source, "ai").unwrap();
    }
}

#[test]
fn defers_dynamic_and_computed_denominators() {
    for source in [
        "let x = 2 / mystery",
        "let x = 2 / f(0)",
    ] {
        check_source(source, "ai").unwrap();
    }
}

#[test]
fn checks_zero_divisor_inside_nested_call_and_function() {
    for source in [
        "print(2 / 0)",
        "fn f(x) { return x % 0 }",
        "if true { let x = 2 // 0 }",
    ] {
        assert_eq!(check_source(source, "ai").unwrap_err().code, 1008);
    }
}

#[test]
fn mixed_invalid_types_still_take_n1002_priority() {
    assert_eq!(
        check_source("let x = \"text\" / 0", "ai").unwrap_err().code,
        1002
    );
}

#[test]
fn slovak_profile_uses_the_same_zero_division_guard() {
    let err = check_source("nech pomer = 2 / 0", "sk").unwrap_err();
    assert_eq!(err.code, 1008);
}

#[test]
fn zero_elsewhere_does_not_trigger() {
    check_source("let x = 0 / 2\nlet y = 0 * 8\nlet z = 2 ** 0", "ai").unwrap();
}
