#![forbid(unsafe_code)]

mod expression;

pub use expression::{evaluate_expression, ExpressionError};

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArithmeticError {
    DivisionByZero,
    Overflow,
}

impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DivisionByZero => "division by zero",
            Self::Overflow => "integer overflow",
        })
    }
}

impl std::error::Error for ArithmeticError {}

pub fn evaluate(left: i64, operator: Operator, right: i64) -> Result<i64, ArithmeticError> {
    let value = match operator {
        Operator::Add => left.checked_add(right),
        Operator::Subtract => left.checked_sub(right),
        Operator::Multiply => left.checked_mul(right),
        Operator::Divide if right == 0 => return Err(ArithmeticError::DivisionByZero),
        Operator::Divide => left.checked_div(right),
    };
    value.ok_or(ArithmeticError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_checked_integer_results() {
        use Operator::*;
        for (left, operator, right, expected) in [
            (1, Add, 1, 2),
            (1, Subtract, 2, -1),
            (-2, Multiply, 3, -6),
            (-7, Divide, 3, -2),
            (7, Divide, -3, -2),
            (i64::MIN, Add, 0, i64::MIN),
            (i64::MAX, Add, 0, i64::MAX),
        ] {
            assert_eq!(evaluate(left, operator, right), Ok(expected));
        }
    }

    #[test]
    fn distinguishes_zero_division_from_overflow() {
        use Operator::*;
        for (left, operator, right) in [
            (i64::MAX, Add, 1),
            (i64::MIN, Subtract, 1),
            (i64::MAX, Multiply, 2),
            (i64::MIN, Divide, -1),
        ] {
            assert_eq!(
                evaluate(left, operator, right),
                Err(ArithmeticError::Overflow)
            );
        }
        assert_eq!(evaluate(0, Divide, 0), Err(ArithmeticError::DivisionByZero));
    }
}
