use crate::compute::{KernelBuilder, KernelLauncher};
use crate::frontend::{CubeContext, CubePrimitive, CubeType, ExpandElement, Numeric};
use crate::ir::{Elem, IntKind, Item, Variable, Vectorization};
use crate::prelude::index_assign;
use crate::Runtime;

use super::{ArgSettings, LaunchArg, LaunchArgExpand, UInt, Vectorized};

/// Signed integer. Used as input in int kernels
pub trait Int: Numeric + std::ops::Rem<Output = Self> {
    fn new(val: i64) -> Self;
    fn new_expand(context: &mut CubeContext, val: i64) -> <Self as CubeType>::ExpandType;
    fn vectorized(val: i64, vectorization: UInt) -> Self;
    fn vectorized_expand(
        context: &mut CubeContext,
        val: i64,
        vectorization: UInt,
    ) -> <Self as CubeType>::ExpandType;
}

macro_rules! impl_int {
    ($type:ident, $primitive:ty) => {
        #[derive(Clone, Copy)]
        pub struct $type {
            pub val: $primitive,
            pub vectorization: u8,
        }

        impl CubeType for $type {
            type ExpandType = ExpandElement;
        }

        impl CubeType for &$type {
            type ExpandType = ExpandElement;
        }

        impl CubeType for &mut $type {
            type ExpandType = ExpandElement;
        }

        impl CubePrimitive for $type {
            fn as_elem() -> Elem {
                Elem::Int(IntKind::$type)
            }
        }

        impl Numeric for $type {}

        impl Int for $type {
            fn new(val: i64) -> Self {
                Self {
                    val: val as $primitive,
                    vectorization: 1,
                }
            }

            fn new_expand(_context: &mut CubeContext, val: i64) -> <Self as CubeType>::ExpandType {
                let new_var = Variable::ConstantScalar(val as f64, Self::as_elem());
                ExpandElement::Plain(new_var)
            }

            fn vectorized(val: i64, vectorization: UInt) -> Self {
                if vectorization.val == 1 {
                    Self::new(val)
                } else {
                    Self {
                        val: val as $primitive,
                        vectorization: vectorization.val as u8,
                    }
                }
            }

            fn vectorized_expand(
                context: &mut CubeContext,
                val: i64,
                vectorization: UInt,
            ) -> <Self as CubeType>::ExpandType {
                if vectorization.val == 1 {
                    Self::new_expand(context, val)
                } else {
                    let mut new_var = context
                        .create_local(Item::vectorized(Self::as_elem(), vectorization.val as u8));
                    for (i, element) in vec![val; vectorization.val as usize].iter().enumerate() {
                        new_var = index_assign::expand(context, new_var, i, *element);
                    }

                    new_var
                }
            }
        }

        impl LaunchArgExpand for &$type {
            fn expand(builder: &mut KernelBuilder, vectorization: Vectorization) -> ExpandElement {
                assert_eq!(vectorization, 1, "Attempted to vectorize a scalar");
                builder.scalar($type::as_elem())
            }
        }

        impl LaunchArgExpand for &mut $type {
            fn expand(builder: &mut KernelBuilder, vectorization: Vectorization) -> ExpandElement {
                assert_eq!(vectorization, 1, "Attempted to vectorize a scalar");
                builder.scalar($type::as_elem())
            }
        }

        impl LaunchArg for $type {
            type RuntimeArg<'a, R: Runtime> = $primitive;
        }

        impl Vectorized for $type {
            fn vectorization_factor(&self) -> UInt {
                UInt {
                    val: self.vectorization as u32,
                    vectorization: 1,
                }
            }

            fn vectorize(mut self, factor: UInt) -> Self {
                self.vectorization = factor.vectorization;
                self
            }
        }
    };
}

impl_int!(I32, i32);
impl_int!(I64, i64);

impl From<i64> for I64 {
    fn from(value: i64) -> Self {
        Self {
            val: value,
            vectorization: 1,
        }
    }
}

impl From<i32> for I32 {
    fn from(value: i32) -> Self {
        Self {
            val: value,
            vectorization: 1,
        }
    }
}

impl<R: Runtime> ArgSettings<R> for i32 {
    fn register(&self, settings: &mut KernelLauncher<R>) {
        settings.register_i32(*self);
    }
}

impl<R: Runtime> ArgSettings<R> for i64 {
    fn register(&self, settings: &mut KernelLauncher<R>) {
        settings.register_i64(*self);
    }
}
