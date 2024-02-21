use super::{
    gpu, Elem, Item, Metadata, Operator, ReadGlobalAlgo, ReadGlobalWithLayoutAlgo, Scope, Variable,
};
use crate::codegen::dialect::gpu::BinaryOperator;

impl ReadGlobalAlgo {
    pub fn expand(self, scope: &mut Scope) {
        scope.register(Operator::Index(BinaryOperator {
            lhs: self.global,
            rhs: Variable::Id,
            out: self.out,
        }));
    }
}

impl ReadGlobalWithLayoutAlgo {
    pub fn expand(self, scope: &mut Scope) {
        let out = self.out;
        let tensor = self.global;
        let layout = self.layout;
        let index_item_ty = Item::Scalar(Elem::UInt);
        let index_local = scope.create_local(index_item_ty);
        let zero: Variable = 0u32.into();
        let id = Variable::Id;
        let offset: Variable = match self.global.item() {
            Item::Vec4(_) => 4u32,
            Item::Vec3(_) => 3u32,
            Item::Vec2(_) => 2u32,
            Item::Scalar(_) => 1u32,
        }
        .into();

        gpu!(scope, index_local = zero);
        gpu!(
            scope,
            range(zero, Variable::Rank).for_each(|i, scope| {
                let stride = scope.create_local(index_item_ty);
                let stride_layout = scope.create_local(index_item_ty);
                let shape = scope.create_local(index_item_ty);
                let tmp = scope.create_local(index_item_ty);

                gpu!(scope, stride = stride(tensor, i));
                gpu!(scope, shape = shape(tensor, i));
                gpu!(scope, stride_layout = stride(layout, i));

                gpu!(scope, tmp = id * offset);
                gpu!(scope, tmp = tmp / stride_layout);
                gpu!(scope, tmp = tmp % shape);
                gpu!(scope, tmp = tmp * stride);
                gpu!(scope, index_local = index_local + tmp);
            })
        );

        gpu!(scope, index_local = index_local / offset);
        gpu!(scope, out = tensor[index_local]);
    }
}
