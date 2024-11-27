#[burn_tensor_testgen::testgen(q_log1p)]
mod tests {
    use super::*;
    use burn_tensor::TensorData;

    #[test]
    fn should_support_exp_log1p() {
        let tensor = QTensor::<TestBackend, 2>::int8([[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]);

        let output = tensor.log1p();
        let expected = TensorData::from([
            [0.0, core::f32::consts::LN_2, 1.0986],
            [1.3862, 1.6094, 1.7917],
        ]);

        // Precision 1 to approximate de/quantization errors
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&expected, 1);
    }
}
