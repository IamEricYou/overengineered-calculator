#![forbid(unsafe_code)]

use calc_core::{compile_expression, evaluate, evaluate_expression, Operator};
use std::{ffi::OsString, process::ExitCode};

fn calculate(args: &[OsString]) -> Result<i64, String> {
    const USAGE: &str =
        "usage: overcalc <expression> [--engine <naive|enterprise>] | <left> <operator> <right>";
    let expression_and_engine = match args {
        [expression] => Some((expression, false)),
        [_, flag] if flag == "--engine" => {
            return Err(format!("missing value for --engine; {USAGE}"));
        }
        [expression, flag, engine] if flag == "--engine" => match engine.to_str() {
            Some("naive") => Some((expression, true)),
            Some("enterprise") => Some((expression, false)),
            _ => {
                return Err(format!(
                    "available expression engines: naive, enterprise; {USAGE}"
                ))
            }
        },
        [_, flag, ..] if flag.as_encoded_bytes().starts_with(b"--") && flag != "--engine" => {
            return Err(format!(
                "unknown option: {}; {USAGE}",
                flag.to_string_lossy().escape_default()
            ));
        }
        _ => None,
    };
    if let Some((expression, naive)) = expression_and_engine {
        let expression = expression
            .to_str()
            .ok_or("expression must be valid UTF-8")?;
        let result = if naive {
            evaluate_expression(expression)
        } else {
            compile_expression(expression).and_then(|program| program.execute())
        };
        return result.map_err(|error| error.to_string());
    }
    let [left, operator, right] = args else {
        return Err(USAGE.into());
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
