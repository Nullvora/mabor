//! This module exposes cooperative matrix-multiply and accumulate operations.
//!
//! Most of the functions are actually unsafe, since they mutate their input, even if they are
//! passed as reference.
//!
//! # Example
//!
//! This is a basic 16x16x16 matrix multiplication example.
//!
//! ```rust, ignore
//! #[cube(launch)]
//! pub fn example(lhs: &Array<F16>, rhs: &Array<F16>, out: &mut Array<F32>) {
//!     let a = cmma::Matrix::<F16>::new(
//!         cmma::MatrixIdent::A,
//!         16,
//!         16,
//!         16,
//!         cmma::MatrixLayout::ColMajor,
//!     );
//!     let b = cmma::Matrix::<F16>::new(
//!         cmma::MatrixIdent::B,
//!         16,
//!         16,
//!         16,
//!         cmma::MatrixLayout::RowMajor,
//!     );
//!     let c = cmma::Matrix::<F32>::new(
//!         cmma::MatrixIdent::Accumulator,
//!         16,
//!         16,
//!         16,
//!         cmma::MatrixLayout::Undefined,
//!     );
//!     cmma::fill::<F32>(&c, F32::new(0.0));
//!     cmma::load::<F16>(&a, lhs, UInt::new(16));
//!     cmma::load::<F16>(&b, rhs, UInt::new(16));
//!
//!     cmma::execute::<F16, F16, F32, F32>(&a, &b, &c, &c);
//!
//!     cmma::store::<F32>(out, &c, UInt::new(16), cmma::MatrixLayout::RowMajor);
//! }
//! ```

use std::marker::PhantomData;

use crate::{
    ir::{self, Operation},
    unexpanded,
};

use super::{
    Array, CubeContext, CubePrimitive, CubeType, ExpandElement, ExpandElementTyped, Init, UInt,
};

pub use ir::{MatrixIdent, MatrixLayout};

/// A matrix represent a 2D grid of numbers.
///
/// They can either be in a [row major](MatrixLayout::RowMajor) or a
/// [column major](MatrixLayout::ColMajor) format.
pub struct Matrix<C: CubeType> {
    _c: PhantomData<C>,
}

/// Expand type of [Matrix].
#[derive(Clone)]
pub struct MatrixExpand {
    elem: ExpandElement,
}

impl<C: CubeType> CubeType for Matrix<C> {
    type ExpandType = MatrixExpand;
}

impl Init for MatrixExpand {
    fn init(self, _context: &mut CubeContext) -> Self {
        self
    }
}

impl<C: CubePrimitive> Matrix<C> {
    /// Create a new matrix that is going to be used in the
    /// [matrix-multiply and accumulate](execute) function.
    ///
    /// You have to declare the shape used for the execution.
    /// The shape of the current matrix is determined using the [MatrixIdent].
    ///
    /// * [MatrixIdent::A] Shape => (M, K)
    /// * [MatrixIdent::B] Shape => (K, N)
    /// * [MatrixIdent::Accumulator] Shape => (M, N)
    ///
    /// Not all shapes are supported, and the permitted shapes depend on the element type.
    ///
    /// Refer to [nvidia documentation](https://docs.nvidia.com/cuda/cuda-c-programming-guide/index.html#element-types-and-matrix-sizes).
    #[allow(unused_variables)]
    pub fn new(ident: MatrixIdent, m: u8, n: u8, k: u8, layout: MatrixLayout) -> Self {
        Matrix { _c: PhantomData }
    }

    pub fn new_expand(
        context: &mut CubeContext,
        ident: MatrixIdent,
        m: u8,
        n: u8,
        k: u8,
        layout: MatrixLayout,
    ) -> MatrixExpand {
        let elem = context.create_matrix(ir::Matrix {
            ident,
            m,
            n,
            k,
            elem: C::as_elem(),
            layout,
        });
        MatrixExpand { elem }
    }
}

/// Fill the matrix with the provided value.
#[allow(unused_variables)]
pub fn fill<C: CubeType>(mat: &Matrix<C>, value: C) {
    unexpanded!()
}

/// Expand method of [fill].
pub fn fill_expand<C: CubeType>(
    context: &mut CubeContext,
    mat: MatrixExpand,
    value: ExpandElement,
) {
    context.register(Operation::CoopMma(ir::CoopMma::Fill {
        mat: *mat.elem,
        value: *value,
    }));
}

/// Load the matrix with the provided array using the stride.
#[allow(unused_variables)]
pub fn load<C: CubeType>(mat: &Matrix<C>, value: &Array<C>, stride: UInt) {
    unexpanded!()
}

/// Expand method of [load].
#[allow(unused_variables)]
pub fn load_expand<C: CubeType>(
    context: &mut CubeContext,
    mat: MatrixExpand,
    value: ExpandElementTyped<Array<C>>,
    stride: ExpandElement,
) {
    context.register(Operation::CoopMma(ir::CoopMma::Load {
        mat: *mat.elem,
        value: *value.expand,
        stride: *stride,
    }));
}

/// Store the matrix in the given array following the given stride and layout.
#[allow(unused_variables)]
pub fn store<C: CubePrimitive>(
    output: &Array<C>,
    mat: &Matrix<C>,
    stride: UInt,
    layout: MatrixLayout,
) {
    unexpanded!()
}

/// Expand method of [store].
#[allow(unused_variables)]
pub fn store_expand<C: CubePrimitive>(
    context: &mut CubeContext,
    output: ExpandElementTyped<Array<C>>,
    mat: MatrixExpand,
    stride: ExpandElement,
    layout: MatrixLayout,
) {
    context.register(Operation::CoopMma(ir::CoopMma::Store {
        output: *output.expand,
        mat: *mat.elem,
        stride: *stride,
        layout,
    }));
}

/// Execute the matrix-multiply and accumulate operation on the given [matrices](Matrix).
#[allow(unused_variables)]
pub fn execute<A: CubePrimitive, B: CubePrimitive, C: CubePrimitive, D: CubePrimitive>(
    mat_a: &Matrix<A>,
    mat_b: &Matrix<B>,
    mat_c: &Matrix<C>,
    mat_d: &Matrix<D>,
) {
    unexpanded!()
}

/// Expand method of [execute].
pub fn execute_expand<A: CubePrimitive, B: CubePrimitive, C: CubePrimitive, D: CubePrimitive>(
    context: &mut CubeContext,
    mat_a: MatrixExpand,
    mat_b: MatrixExpand,
    mat_c: MatrixExpand,
    mat_d: MatrixExpand,
) {
    context.register(Operation::CoopMma(ir::CoopMma::Execute {
        mat_a: *mat_a.elem,
        mat_b: *mat_b.elem,
        mat_c: *mat_c.elem,
        mat_d: *mat_d.elem,
    }));
}
