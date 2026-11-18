use super::{CheckedExpr, NodeKind, Operation, Operator, Span};

/// Each definition produces one value. Its position is the value's ID.
pub(super) struct LinearIr {
    pub(super) definitions: Vec<Definition>,
    pub(super) result: ValueId,
}

#[derive(Clone, Copy)]
pub(super) struct ValueId(pub(super) usize);

pub(super) struct Definition {
    pub(super) value: Value,
    pub(super) span: Span,
}

pub(super) enum Value {
    Constant(i64),
    Negate(ValueId),
    Binary(Operator, ValueId, ValueId),
}

pub(super) fn lower(tree: &CheckedExpr) -> LinearIr {
    // Validation appends each node after its children, visiting left before right.
    // Preserve that order so potentially failing arithmetic runs in AST order.
    let definitions = tree
        .nodes
        .iter()
        .map(|node| {
            let value = match node.kind {
                NodeKind::Literal(value) => Value::Constant(value),
                NodeKind::Operation(Operation::Negate(child)) => Value::Negate(ValueId(child.0)),
                NodeKind::Operation(Operation::Binary(operator, left, right)) => {
                    Value::Binary(operator, ValueId(left.0), ValueId(right.0))
                }
            };
            Definition {
                value,
                span: node.span.clone(),
            }
        })
        .collect();
    LinearIr {
        definitions,
        result: ValueId(tree.root.0),
    }
}

impl Value {
    pub(super) fn for_each_operand(&self, mut visit: impl FnMut(ValueId)) {
        match *self {
            Self::Constant(_) => {}
            Self::Negate(id) => visit(id),
            Self::Binary(_, left, right) => {
                visit(left);
                visit(right);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum InvalidIr {
    OperandNotDefined { definition: usize, operand: usize },
    ResultNotDefined(usize),
}

/// Only the verifier can construct this wrapper.
pub(super) struct VerifiedIr(LinearIr);

impl VerifiedIr {
    pub(super) fn verify(ir: LinearIr) -> Result<Self, InvalidIr> {
        if ir.result.0 >= ir.definitions.len() {
            return Err(InvalidIr::ResultNotDefined(ir.result.0));
        }
        for (index, definition) in ir.definitions.iter().enumerate() {
            let mut invalid = None;
            definition.value.for_each_operand(|id| {
                if id.0 >= index && invalid.is_none() {
                    invalid = Some(id.0);
                }
            });
            if let Some(operand) = invalid {
                return Err(InvalidIr::OperandNotDefined {
                    definition: index,
                    operand,
                });
            }
        }
        Ok(Self(ir))
    }

    pub(super) fn into_inner(self) -> LinearIr {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_self_future_and_out_of_bounds_operands() {
        for operand in [0, 1, 2] {
            let ir = LinearIr {
                definitions: vec![
                    Definition {
                        value: Value::Negate(ValueId(operand)),
                        span: 0..1,
                    },
                    Definition {
                        value: Value::Constant(1),
                        span: 1..2,
                    },
                ],
                result: ValueId(1),
            };
            assert_eq!(
                VerifiedIr::verify(ir).err(),
                Some(InvalidIr::OperandNotDefined {
                    definition: 0,
                    operand
                }),
            );
        }
    }

    #[test]
    fn rejects_missing_result() {
        let ir = LinearIr {
            definitions: vec![],
            result: ValueId(0),
        };
        assert_eq!(
            VerifiedIr::verify(ir).err(),
            Some(InvalidIr::ResultNotDefined(0))
        );
    }
}
