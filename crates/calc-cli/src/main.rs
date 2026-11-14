#![forbid(unsafe_code)]

use calc_core::{evaluate, evaluate_expression, Operator};
use std::{ffi::OsString, process::ExitCode};

fn calculate(args: &[OsString]) -> Result<i64, String> {
    let expression = match args {
        [expression] => Some(expression),
        [expression, flag, engine] if flag == "--engine" => {
            if engine != "naive" {
                return Err("available expression engine: naive".into());
            }
            Some(expression)
        }
        _ => None,
    };
    if let Some(expression) = expression {
        let expression = expression
            .to_str()
            .ok_or("expression must be valid UTF-8")?;
        return evaluate_expression(expression).map_err(|error| error.to_string());
    }
    let [left, operator, right] = args else {
        return Err("usage: overcalc <expression> | <left> <operator> <right>".into());
    };

    let operator = match operator.to_str() {
        Some("+") => Operator::Add,
        Some("-") => Operator::Subtract,
        Some("*") => Operator::Multiply,
        Some("/") => Operator::Divide,
        _ => return Err("operator must be one of +, -, *, /".into()),
    };

    let left = left.to_str().and_then(|value| value.parse::<i64>().ok());
    let right = right.to_str().and_then(|value| value.parse::<i64>().ok());
    let left = left.ok_or("left operand must be a signed 64-bit integer")?;
    let right = right.ok_or("right operand must be a signed 64-bit integer")?;
    evaluate(left, operator, right).map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match calculate(&args) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
