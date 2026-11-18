use super::{
    evaluate,
    ir::{Value, VerifiedIr},
    ArithmeticError, ErrorKind, ExpressionError, Operator, Span,
};

#[derive(Clone, Copy)]
struct Register(usize);

#[derive(Clone, Copy)]
enum Instruction {
    LoadConst(Register, i64),
    Negate(Register, Register),
    Add(Register, Register, Register),
    Subtract(Register, Register, Register),
    Multiply(Register, Register, Register),
    Divide(Register, Register, Register),
}

/// In-memory register bytecode emitted by the compiler. Each definition writes
/// a distinct register. The result register is read after the last instruction.
pub struct Program {
    instructions: Vec<Instruction>,
    spans: Vec<Span>,
    result: Register,
}

pub(super) fn emit(ir: VerifiedIr) -> Program {
    let ir = ir.into_inner();
    let (instructions, spans) = ir
        .definitions
        .into_iter()
        .enumerate()
        .map(|(index, definition)| {
            let destination = Register(index);
            let instruction = match definition.value {
                Value::Constant(value) => Instruction::LoadConst(destination, value),
                Value::Negate(source) => Instruction::Negate(destination, Register(source.0)),
                Value::Binary(operator, left, right) => {
                    let left = Register(left.0);
                    let right = Register(right.0);
                    match operator {
                        Operator::Add => Instruction::Add(destination, left, right),
                        Operator::Subtract => Instruction::Subtract(destination, left, right),
                        Operator::Multiply => Instruction::Multiply(destination, left, right),
                        Operator::Divide => Instruction::Divide(destination, left, right),
                    }
                }
            };
            (instruction, definition.span)
        })
        .unzip();
    Program {
        instructions,
        spans,
        result: Register(ir.result.0),
    }
}

impl Program {
    /// Number of instructions emitted for this program.
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    /// Run the compiled program with a fresh register file.
    pub fn execute(&self) -> Result<i64, ExpressionError> {
        let mut registers = vec![0_i64; self.instructions.len()];
        for (instruction, span) in self.instructions.iter().zip(&self.spans) {
            use Instruction::*;
            let (destination, value) = match *instruction {
                LoadConst(dst, value) => (dst, Ok(value)),
                Negate(dst, source) => (
                    dst,
                    registers[source.0]
                        .checked_neg()
                        .ok_or(ArithmeticError::Overflow),
                ),
                Add(dst, left, right) => (
                    dst,
                    evaluate(registers[left.0], Operator::Add, registers[right.0]),
                ),
                Subtract(dst, left, right) => (
                    dst,
                    evaluate(registers[left.0], Operator::Subtract, registers[right.0]),
                ),
                Multiply(dst, left, right) => (
                    dst,
                    evaluate(registers[left.0], Operator::Multiply, registers[right.0]),
                ),
                Divide(dst, left, right) => (
                    dst,
                    evaluate(registers[left.0], Operator::Divide, registers[right.0]),
                ),
            };
            registers[destination.0] = value.map_err(|error| ExpressionError {
                kind: ErrorKind::Arithmetic(error),
                span: span.clone(),
            })?;
        }
        Ok(registers[self.result.0])
    }
}
