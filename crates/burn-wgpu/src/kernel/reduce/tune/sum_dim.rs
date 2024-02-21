use burn_compute::tune::{AutotuneOperation, AutotuneOperationSet};
use burn_tensor::{Element, ElementConversion};

use crate::{
    compute::JitAutotuneKey,
    element::JitElement,
    kernel::{
        prng::random_like_uniform,
        reduce::{init_reduce_output, sum_dim, sum_dim_shared_memory},
    },
    ops::numeric::empty_device,
    reduce_tune_ops,
    tensor::JitTensor,
    Runtime,
};

use super::ReduceAutotuneKey;

/// Set of sum_dim implementations available for autotune
/// Autotune key is given by concatenating the closest upper power of 2 of
/// dim to reduce, and product of others
pub struct SumDimAutotuneOperationSet<R: Runtime, E: JitElement, const D: usize> {
    key: JitAutotuneKey,
    input: JitTensor<R, E, D>,
    output: JitTensor<R, E, D>,
    reduce_dim: usize,
}
impl<R: Runtime, E: JitElement, const D: usize> SumDimAutotuneOperationSet<R, E, D> {
    fn new(input: JitTensor<R, E, D>, output: JitTensor<R, E, D>, reduce_dim: usize) -> Self {
        Self {
            key: JitAutotuneKey::SumDim(ReduceAutotuneKey::new(
                &input.shape,
                &input.strides,
                reduce_dim,
            )),
            input,
            output,
            reduce_dim,
        }
    }
}

impl<R: Runtime, E: JitElement + Element, const D: usize> AutotuneOperationSet<JitAutotuneKey>
    for SumDimAutotuneOperationSet<R, E, D>
{
    fn key(&self) -> JitAutotuneKey {
        self.key.clone()
    }

    fn autotunables(&self) -> Vec<Box<dyn AutotuneOperation>> {
        let random_bounds: (E, E) = ((-10.0).elem::<E>(), (10.0).elem::<E>());
        let input = random_like_uniform(&self.input, random_bounds.0, random_bounds.1);

        let output = empty_device(
            self.output.client.clone(),
            self.output.device.clone(),
            self.output.shape.clone(),
        );

        vec![
            Box::new(SumDimAutotune::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
            Box::new(SumDimSharedMemoryAutotune::new(
                input.clone(),
                output.clone(),
                self.reduce_dim,
            )),
        ]
    }

    fn fastest(self: Box<Self>, fastest_index: usize) -> Box<dyn AutotuneOperation> {
        // Warning: since AutotuneOperationSet shares his key with MeanDimAutotuneOperationSet
        // we must make sure the order here is correlated with MeanDim
        match fastest_index {
            0 => Box::new(SumDimAutotune::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            1 => Box::new(SumDimSharedMemoryAutotune::new(
                self.input,
                self.output,
                self.reduce_dim,
            )),
            _ => panic!("Fastest index is out of bound"),
        }
    }
}

/// Executes autotune on sum_dim operation
pub fn sum_dim_autotune<R: Runtime, E: JitElement + Element, const D: usize>(
    input: JitTensor<R, E, D>,
    reduce_dim: usize,
) -> JitTensor<R, E, D> {
    let client = input.client.clone();

    let output = init_reduce_output(&input, reduce_dim);

    let operation_set = Box::new(SumDimAutotuneOperationSet::new(
        input,
        output.clone(),
        reduce_dim,
    ));

    client.autotune_execute(operation_set);

    output
}

// Probably better on balanced tensor shapes
reduce_tune_ops!(SumDimAutotune, sum_dim);

// Probably better on tensors large along reduce dim
reduce_tune_ops!(SumDimSharedMemoryAutotune, sum_dim_shared_memory);
