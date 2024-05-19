use burn_cube::{cube, Float};

#[cube]
pub fn if_then_else<F: Float>(lhs: F) {
    if lhs < F::from_int(0) {
        let _ = lhs + F::from_int(4);
    } else {
        let _ = lhs - F::from_int(5);
    }
}

mod tests {
    use burn_cube::{
        cpa,
        dialect::{Elem, Item, Variable},
        CubeContext, PrimitiveVariable, F32,
    };

    use super::if_then_else_expand;

    type ElemType = F32;

    #[test]
    fn cube_if_else_test() {
        let mut context = CubeContext::root();

        let lhs = context.create_local(Item::new(ElemType::as_elem()));

        if_then_else_expand::<ElemType>(&mut context, lhs);
        let scope = context.into_scope();

        assert_eq!(format!("{:?}", scope.operations), inline_macro_ref());
    }

    fn inline_macro_ref() -> String {
        let mut context = CubeContext::root();
        let item = Item::new(ElemType::as_elem());
        let lhs = context.create_local(item);

        let mut scope = context.into_scope();
        let cond = scope.create_local(Item::new(Elem::Bool));
        let lhs: Variable = lhs.into();
        let y = scope.create_local(item);

        cpa!(scope, cond = lhs < 0f32);
        cpa!(&mut scope, if(cond).then(|scope| {
            cpa!(scope, y = lhs + 4.0f32);
        }).else(|scope|{
            cpa!(scope, y = lhs - 5.0f32);
        }));

        format!("{:?}", scope.operations)
    }
}
