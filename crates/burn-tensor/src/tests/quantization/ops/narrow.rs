#[burn_tensor_testgen::testgen(q_narrow)]
mod tests {
    use super::*;
    use burn_tensor::{Shape, TensorData};

    // NOTE: we use affine quantization to reduce quantization errors for range of input values
    #[test]
    fn test_narrow() {
        let tensor =
            QTensor::<TestBackend, 2>::int8_affine([[1., 2., 3.], [7., 8., 9.], [13., 14., 15.]]);

        let output = tensor.clone().narrow(0, 0, 2);
        let expected = TensorData::from([[1., 2., 3.], [7., 8., 9.]]);

        assert_eq!(output.shape(), Shape::from([2, 3]));
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&expected, 3);

        let output = tensor.narrow(1, 1, 2);
        let expected = TensorData::from([[2., 3.], [8., 9.], [14., 15.]]);
        assert_eq!(output.shape(), Shape::from([3, 2]));
        output
            .dequantize()
            .into_data()
            .assert_approx_eq(&expected, 3);
    }

    #[test]
    #[should_panic]
    fn test_narrow_invalid_dim() {
        let tensor =
            QTensor::<TestBackend, 2>::int8_affine([[1., 2., 3.], [7., 8., 9.], [13., 14., 15.]]);

        let output = tensor.narrow(2, 0, 2);
    }

    #[test]
    #[should_panic]
    fn test_narrow_invalid_start() {
        let tensor =
            QTensor::<TestBackend, 2>::int8_affine([[1., 2., 3.], [7., 8., 9.], [13., 14., 15.]]);

        let output = tensor.narrow(0, 3, 2);
    }

    #[test]
    #[should_panic]
    fn test_narrow_invalid_zero_length() {
        let tensor =
            QTensor::<TestBackend, 2>::int8_affine([[1., 2., 3.], [7., 8., 9.], [13., 14., 15.]]);

        let output = tensor.narrow(0, 1, 0);
    }

    #[test]
    #[should_panic]
    fn test_narrow_invalid_length() {
        let tensor =
            QTensor::<TestBackend, 2>::int8_affine([[1., 2., 3.], [7., 8., 9.], [13., 14., 15.]]);

        let output = tensor.narrow(0, 0, 4);
    }
}
