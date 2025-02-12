#[burn_tensor_testgen::testgen(q_expand)]
mod tests {
    use super::*;
    use burn_tensor::TensorData;

    #[test]
    fn expand_2d() {
        let tensor = QTensor::<TestBackend, 1>::int8([1.0, 2.0, 3.0]);
        let output = tensor.expand([3, 3]);

        // Precision 1 to approximate de/quantization errors
        output.dequantize().into_data().assert_approx_eq(
            &TensorData::from([[1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]),
            1,
        );

        // Quantized [4.0, 7.0, 2.0, 3.0]
        let tensor = QTensor::<TestBackend, 1>::int8([4.0, 7.0, 2.0, 3.0]);
        let output = tensor.expand([2, 4]);

        // Precision 1 to approximate de/quantization errors
        output.dequantize().into_data().assert_approx_eq(
            &TensorData::from([[4.0, 7.0, 2.0, 3.0], [4.0, 7.0, 2.0, 3.0]]),
            1,
        );
    }

    #[test]
    fn expand_3d() {
        let tensor = QTensor::<TestBackend, 2>::int8([[1.0, 2.0], [3.0, 4.0]]);

        let output = tensor.expand([3, 2, 2]);
        let expected = TensorData::from([
            [[1.0, 2.0], [3.0, 4.0]],
            [[1.0, 2.0], [3.0, 4.0]],
            [[1.0, 2.0], [3.0, 4.0]],
        ]);

        // Precision 1 to approximate de/quantization errors
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&expected, 1);
    }

    #[test]
    fn expand_higher_dimensions() {
        let tensor = QTensor::<TestBackend, 2>::int8([[1.0, 2.0, 3.0, 4.0]]);

        let output = tensor.expand([2, 3, 4]);
        let expected = TensorData::from([
            [
                [1.0, 2.0, 3.0, 4.0],
                [1.0, 2.0, 3.0, 4.0],
                [1.0, 2.0, 3.0, 4.0],
            ],
            [
                [1.0, 2.0, 3.0, 4.0],
                [1.0, 2.0, 3.0, 4.0],
                [1.0, 2.0, 3.0, 4.0],
            ],
        ]);

        // Precision 1 to approximate de/quantization errors
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&expected, 1);
    }

    #[test]
    fn broadcast_single() {
        let tensor = QTensor::<TestBackend, 1>::int8([1.0]);

        let output = tensor.expand([2, 3]);

        output
            .dequantize()
            .into_data()
            .assert_eq(&TensorData::from([[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]), false);
    }

    #[test]
    #[should_panic]
    fn should_fail_expand_incompatible_shapes() {
        let tensor = QTensor::<TestBackend, 1>::int8([1.0, 2.0, 3.0]);
        let _expanded_tensor = tensor.expand([2, 2]);
    }

    #[test]
    fn should_all_negative_one() {
        let tensor = QTensor::<TestBackend, 1>::int8([1.0, 2.0, 3.0]);

        let output = tensor.expand([2, -1]);

        // Precision 1 to approximate de/quantization errors
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&TensorData::from([[1., 2., 3.], [1., 2., 3.]]), 1);
    }

    #[test]
    #[should_panic]
    fn should_panic_negative_one_on_non_existing_dim() {
        let tensor = QTensor::<TestBackend, 1>::int8([1.0, 2.0, 3.0]);
        let _expanded_tensor = tensor.expand([-1, 3]);
    }
}
