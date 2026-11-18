use super::{
    evaluate,
    ir::{LinearIr, Value, ValueId, VerifiedIr},
};

pub(super) fn fold_constants(ir: VerifiedIr) -> LinearIr {
    let mut ir = ir.into_inner();
    for index in 0..ir.definitions.len() {
        let constant = |id: ValueId| match ir.definitions[id.0].value {
            Value::Constant(value) => Some(value),
            _ => None,
        };
        let folded = match ir.definitions[index].value {
            Value::Constant(_) => None,
            Value::Negate(id) => constant(id).and_then(i64::checked_neg),
            Value::Binary(operator, left, right) => constant(left)
                .zip(constant(right))
                .and_then(|(left, right)| evaluate(left, operator, right).ok()),
        };
        // Keep failing and unknown operations at their original positions.
        if let Some(value) = folded {
            ir.definitions[index].value = Value::Constant(value);
        }
    }
    remove_unused_constants(ir)
}

fn remove_unused_constants(mut ir: LinearIr) -> LinearIr {
    let mut used = vec![false; ir.definitions.len()];
    used[ir.result.0] = true;
    for definition in &ir.definitions {
        definition.value.for_each_operand(|id| used[id.0] = true);
    }

    let mut remap = vec![0; ir.definitions.len()];
    let mut old_index = 0;
    let mut new_index = 0;
    ir.definitions.retain_mut(|definition| {
        let keep = used[old_index] || !matches!(definition.value, Value::Constant(_));
        if keep {
            remap[old_index] = new_index;
            new_index += 1;
            definition.value = match definition.value {
                Value::Constant(value) => Value::Constant(value),
                Value::Negate(id) => Value::Negate(ValueId(remap[id.0])),
                Value::Binary(operator, left, right) => {
                    Value::Binary(operator, ValueId(remap[left.0]), ValueId(remap[right.0]))
                }
            };
        }
        old_index += 1;
        keep
    });
    ir.result = ValueId(remap[ir.result.0]);
    ir
}
