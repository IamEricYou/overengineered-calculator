use std::process::Command;

#[test]
fn success_writes_only_the_result() {
    let output = Command::new(env!("CARGO_BIN_EXE_overcalc"))
        .args(["-7", "/", "3"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"-2\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn failures_write_only_a_diagnostic() {
    for (args, expected) in [
        (vec!["1", "/", "0"], "division by zero"),
        (vec!["-9223372036854775808", "/", "-1"], "integer overflow"),
        (
            vec!["9223372036854775808", "+", "1"],
            "left operand must be a signed 64-bit integer",
        ),
        (
            vec!["1", "+", "oops"],
            "right operand must be a signed 64-bit integer",
        ),
        (vec!["1", "%", "2"], "operator must be one of +, -, *, /"),
        (
            vec![],
            "usage: overcalc <expression> | <left> <operator> <right>",
        ),
        (vec!["1 / 0"], "division by zero at bytes 2..3"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_overcalc"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, format!("error: {expected}\n").as_bytes());
    }
}

#[test]
fn evaluates_an_expression_with_the_naive_engine() {
    for args in [
        vec!["2 + 3 * (4 - 1)"],
        vec!["2 + 3 * (4 - 1)", "--engine", "naive"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_overcalc"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"11\n");
        assert!(output.stderr.is_empty());
    }
}
