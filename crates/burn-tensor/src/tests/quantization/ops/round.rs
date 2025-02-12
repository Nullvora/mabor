#[burn_tensor_testgen::testgen(q_round)]
mod tests {
    use super::*;
    use burn_tensor::TensorData;

    #[test]
    fn should_support_round_ops() {
        let tensor = QTensor::<TestBackend, 2>::int8([
            [24.0423, 87.9478, 76.1838],
            [59.6929, 43.8169, 94.8826],
        ]);

        let output = tensor.round();
        let expected = TensorData::from([[24., 88., 76.], [60., 44., 95.]]);

        output.into_data().assert_approx_eq(&expected, 3);
    }

    #[test]
    fn should_round_ties_even() {
        // NOTE: round ties to even only affects values that are exact halfway from ceil/floor, so quantization
        // errors can impact this. This basically only guarantees the values for the max value in the range since
        // it is always represented correctly.
        let tensor = QTensor::<TestBackend, 1>::int8([5.5]);

        let output = tensor.round();
        let expected = TensorData::from([6.]);

        output.into_data().assert_approx_eq(&expected, 3);
    }
}
