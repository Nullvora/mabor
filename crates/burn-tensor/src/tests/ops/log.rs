#[burn_tensor_testgen::testgen(log)]
mod tests {
    use super::*;
    use burn_tensor::{Tensor, TensorData};
    use burn_tensor::{Tolerance, ops::FloatElem};
    type FT = FloatElem<TestBackend>;

    #[test]
    fn should_support_log_ops() {
        let data = TensorData::from([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);
        let tensor = TestTensor::<2>::from_data(data, &Default::default());

        let output = tensor.log();
        let expected = TensorData::from([
            [-f32::INFINITY, 0.0, core::f32::consts::LN_2],
            [1.09861, 1.38629, 1.60944],
        ]);

        output.into_data().assert_approx_eq::<FT>(
            &expected,
            Tolerance::default().set_half_precision_relative(1e-3),
        );
    }
}
