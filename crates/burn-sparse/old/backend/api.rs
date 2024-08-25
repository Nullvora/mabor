use crate::backend::{Sparse, SparseBackend};
use burn_tensor::{Int, Tensor, TensorPrimitive};

pub enum CoalesceReduction {
    Sum,
}

pub trait ToSparse<const D: usize, B>
where
    B: SparseBackend,
{
    fn into_sparse(self) -> Tensor<B, D, Sparse>;
}

pub trait SparseTensorApi<const D: usize, B>
where
    B: SparseBackend,
{
    fn sddmm(self, lhs: Tensor<B, D>, rhs: Tensor<B, D>) -> Self;
    fn dense_int(self) -> Tensor<B, D, Int>;
    fn spmm(self, rhs: Tensor<B, D>) -> Tensor<B, D>;
    fn dense(self) -> Tensor<B, D>;
    fn coalesce(self, reduce: CoalesceReduction) -> Tensor<B, D, Sparse>;
    fn number_nonzero(self) -> usize;
    fn density(self) -> f32;
    fn add_dense(self, rhs: Tensor<B, D>) -> Tensor<B, D>;
    fn mul_dense(self, rhs: Tensor<B, D>) -> Tensor<B, D>;
}

impl<const D: usize, B> ToSparse<D, B> for Tensor<B, D>
where
    B: SparseBackend,
{
    fn into_sparse(self) -> Tensor<B, D, Sparse> {
        Tensor::new(B::sparse_to_sparse(self.into_primitive().tensor()))
    }
}

impl<const D: usize, B> SparseTensorApi<D, B> for Tensor<B, D, Sparse>
where
    B: SparseBackend,
{
    fn dense(self) -> Tensor<B, D> {
        Tensor::new(TensorPrimitive::Float(B::sparse_to_dense(
            self.into_primitive(),
        )))
    }

    fn dense_int(self) -> Tensor<B, D, Int> {
        self.dense().int()
    }

    fn spmm(self, rhs: Tensor<B, D>) -> Tensor<B, D> {
        Tensor::new(TensorPrimitive::Float(B::sparse_spmm(
            self.into_primitive(),
            rhs.into_primitive().tensor(),
        )))
    }

    fn sddmm(self, lhs: Tensor<B, D>, rhs: Tensor<B, D>) -> Self {
        Tensor::new(B::sparse_sddmm(
            lhs.into_primitive().tensor(),
            rhs.into_primitive().tensor(),
            self.into_primitive(),
        ))
    }

    fn coalesce(self, reduction: CoalesceReduction) -> Tensor<B, D, Sparse> {
        match reduction {
            CoalesceReduction::Sum => Tensor::new(B::sparse_coalesce_sum(self.into_primitive())),
        }
    }

    fn number_nonzero(self) -> usize {
        B::sparse_nonzero(self.into_primitive())
    }

    fn density(self) -> f32 {
        B::sparse_density(self.into_primitive())
    }

    fn add_dense(self, rhs: Tensor<B, D>) -> Tensor<B, D> {
        Tensor::new(TensorPrimitive::Float(B::sparse_add_dense(
            self.into_primitive(),
            rhs.into_primitive().tensor(),
        )))
    }

    fn mul_dense(self, rhs: Tensor<B, D>) -> Tensor<B, D> {
        Tensor::new(TensorPrimitive::Float(B::sparse_mul_dense(
            self.into_primitive(),
            rhs.into_primitive().tensor(),
        )))
    }
}
