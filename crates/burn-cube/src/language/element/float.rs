use crate::dialect::{Elem, FloatKind, Variable, Vectorization};
use crate::language::{CubeContext, CubeType, ExpandElement, Numeric, PrimitiveVariable};
use std::rc::Rc;

/// Floating point numbers. Used as input in float kernels
pub trait Float: Numeric {
    fn new(val: f64) -> Self;
    fn new_expand(context: &mut CubeContext, val: f64) -> <Self as CubeType>::ExpandType;
}

macro_rules! impl_float {
    ($type:ident) => {
        #[derive(Clone, Copy)]
        pub struct $type {
            pub val: <Self as PrimitiveVariable>::Primitive,
            pub vectorization: u8,
        }

        impl CubeType for $type {
            type ExpandType = ExpandElement;
        }

        impl PrimitiveVariable for $type {
            type Primitive = f64;

            /// Return the element type to use on GPU
            fn as_elem() -> Elem {
                Elem::Float(FloatKind::$type)
            }

            fn vectorization(&self) -> Vectorization {
                self.vectorization.into()
            }

            fn to_f64(&self) -> f64 {
                self.val
            }

            fn from_f64(val: f64) -> Self {
                Self::new(val)
            }

            fn from_i64(val: i64) -> Self {
                Self::new(val as f64)
            }

            fn from_i64_vec(vec: &[i64]) -> Self {
                Self {
                    // We take only one value, because type implements copy and we can't copy an unknown sized vec
                    // When using CPU-side values for debugging kernels, prefer using unvectorized types
                    val: *vec.first().expect("Should be at least one value")
                        as <Self as PrimitiveVariable>::Primitive,
                    vectorization: vec.len() as u8,
                }
            }
        }

        impl Numeric for $type {}

        impl Float for $type {
            fn new(val: <Self as PrimitiveVariable>::Primitive) -> Self {
                Self {
                    val,
                    vectorization: 1,
                }
            }

            fn new_expand(
                _context: &mut CubeContext,
                val: <Self as PrimitiveVariable>::Primitive,
            ) -> <Self as CubeType>::ExpandType {
                let new_var = Variable::ConstantScalar(val as f64, Self::as_elem());
                ExpandElement::new(Rc::new(new_var))
            }
        }
    };
}

impl_float!(F16);
impl_float!(BF16);
impl_float!(F32);
impl_float!(F64);
