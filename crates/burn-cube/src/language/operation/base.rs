use crate::dialect::{BinaryOperator, Elem, Item, Operator, Variable, Vectorization};
use crate::language::{CubeContext, ExpandElement};

pub(crate) fn binary_expand<F>(
    context: &mut CubeContext,
    lhs: ExpandElement,
    rhs: ExpandElement,
    func: F,
) -> ExpandElement
where
    F: Fn(BinaryOperator) -> Operator,
{
    let lhs: Variable = *lhs;
    let rhs: Variable = *rhs;

    let item = lhs.item();
    check_vectorization(item.vectorization, rhs.item().vectorization);

    let out = context.create_local(item);
    let out_var = *out;

    let op = func(BinaryOperator {
        lhs,
        rhs,
        out: out_var,
    });

    context.register(op);

    out
}

pub(crate) fn cmp_expand<F>(
    context: &mut CubeContext,
    lhs: ExpandElement,
    rhs: ExpandElement,
    func: F,
) -> ExpandElement
where
    F: Fn(BinaryOperator) -> Operator,
{
    let lhs: Variable = *lhs;
    let rhs: Variable = *rhs;
    let item = lhs.item();

    check_vectorization(item.vectorization, rhs.item().vectorization);

    let out_item = Item {
        elem: Elem::Bool,
        vectorization: item.vectorization,
    };

    let out = context.create_local(out_item);
    let out_var = *out;

    let op = func(BinaryOperator {
        lhs,
        rhs,
        out: out_var,
    });

    context.register(op);

    out
}

pub(crate) fn assign_op_expand<F>(
    context: &mut CubeContext,
    lhs: ExpandElement,
    rhs: ExpandElement,
    func: F,
) -> ExpandElement
where
    F: Fn(BinaryOperator) -> Operator,
{
    let lhs_var: Variable = *lhs;
    let rhs: Variable = *rhs;

    let op = func(BinaryOperator {
        lhs: lhs_var,
        rhs,
        out: lhs_var,
    });

    context.register(op);

    lhs
}

fn check_vectorization(lhs: Vectorization, rhs: Vectorization) {
    if lhs == 1 || rhs == 1 {
        return;
    }
    assert!(
        lhs == rhs,
        "Tried to perform binary operation on different vectorization schemes."
    );
}
