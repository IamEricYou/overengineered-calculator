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
