use crate::{element::JitElement, ops::numeric::empty_device, tensor::JitTensor, Runtime};
use burn_tensor::Shape;

/// Creates an empty output tensor with matmul output shape
pub fn init_matmul_output<R: Runtime, E: JitElement, const D: usize>(
    lhs: &JitTensor<R, E, D>,
    rhs: &JitTensor<R, E, D>,
) -> JitTensor<R, E, D> {
    empty_device(lhs.client.clone(), lhs.device.clone(), shape_out(lhs, rhs))
}

pub(crate) fn shape_out<R: Runtime, E: JitElement, const D: usize>(
    lhs: &JitTensor<R, E, D>,
    rhs: &JitTensor<R, E, D>,
) -> Shape<D> {
    let mut shape_out = [0; D];
    lhs.shape
        .dims
        .iter()
        .zip(rhs.shape.dims.iter())
        .enumerate()
        .for_each(|(index, (dim_lhs, dim_rhs))| {
            shape_out[index] = usize::max(*dim_lhs, *dim_rhs);
        });
    shape_out[D - 2] = lhs.shape.dims[D - 2];
    shape_out[D - 1] = rhs.shape.dims[D - 1];
    Shape::new(shape_out)
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::tensor::JitTensor;
    use crate::tests::{ReferenceTensor, TestRuntime, TestTensor};
    use burn_tensor::Shape;

    use super::init_matmul_output;

    type TB = TestRuntime;

    pub(crate) fn same_as_reference<F, const D: usize, S>(func: F, shape_lhs: S, shape_rhs: S)
    where
        F: Fn(
            JitTensor<TB, f32, D>,
            JitTensor<TB, f32, D>,
            JitTensor<TB, f32, D>,
        ) -> JitTensor<TB, f32, D>,
        S: Into<Shape<D>>,
    {
        let ref_tensor_device = Default::default();
        let x = ReferenceTensor::random(
            shape_lhs,
            burn_tensor::Distribution::Uniform(-1.0, 1.0),
            &ref_tensor_device,
        );
        let y = ReferenceTensor::random(
            shape_rhs,
            burn_tensor::Distribution::Uniform(-1.0, 1.0),
            &ref_tensor_device,
        );

        let test_tensor_device = Default::default();
        let x_wgpu = TestTensor::from_data(x.to_data(), &test_tensor_device).into_primitive();
        let y_wgpu = TestTensor::from_data(y.to_data(), &test_tensor_device).into_primitive();

        let z_reference = x.matmul(y);

        let out = init_matmul_output(&x_wgpu, &y_wgpu);
        let z = func(x_wgpu, y_wgpu, out);
        let z = TestTensor::from_primitive(z);

        z_reference.into_data().assert_approx_eq(&z.into_data(), 3);
    }

    pub(crate) fn same_as_reference_swapped_dims<F, const D: usize, S>(
        func: F,
        swap_lhs: [usize; 2],
        swap_rhs: [usize; 2],
        shape_lhs: S,
        shape_rhs: S,
    ) where
        F: Fn(
            JitTensor<TB, f32, D>,
            JitTensor<TB, f32, D>,
            JitTensor<TB, f32, D>,
        ) -> JitTensor<TB, f32, D>,
        S: Into<Shape<D>>,
    {
        let x = ReferenceTensor::random(
            shape_lhs,
            burn_tensor::Distribution::Uniform(-1.0, 1.0),
            &Default::default(),
        );
        let y = ReferenceTensor::random(
            shape_rhs,
            burn_tensor::Distribution::Uniform(-1.0, 1.0),
            &Default::default(),
        );

        let x_wgpu = TestTensor::from_data(x.to_data(), &Default::default())
            .swap_dims(swap_lhs[0], swap_lhs[1]);
        let y_wgpu = TestTensor::from_data(y.to_data(), &Default::default())
            .swap_dims(swap_rhs[0], swap_rhs[1]);

        let z_reference = x
            .swap_dims(swap_lhs[0], swap_lhs[1])
            .matmul(y.swap_dims(swap_rhs[0], swap_rhs[1]));

        let out = init_matmul_output(
            &x_wgpu.clone().into_primitive(),
            &y_wgpu.clone().into_primitive(),
        );
        let z = func(x_wgpu.into_primitive(), y_wgpu.into_primitive(), out);
        let z = TestTensor::from_primitive(z);

        z_reference.into_data().assert_approx_eq(&z.into_data(), 3);
    }
}
