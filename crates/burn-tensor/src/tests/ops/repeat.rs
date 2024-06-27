#[burn_tensor_testgen::testgen(repeat)]
mod tests {
    use super::*;
    use burn_tensor::{backend::Backend, Bool, Int, Tensor, TensorData};

    #[test]
    fn should_support_repeat_ops() {
        let data = TensorData::from([[0.0, 1.0, 2.0]]);
        let tensor = Tensor::<TestBackend, 2>::from_data(data, &Default::default());

        let output = tensor.repeat(0, 4);
        let expected = TensorData::from([
            [0.0, 1.0, 2.0],
            [0.0, 1.0, 2.0],
            [0.0, 1.0, 2.0],
            [0.0, 1.0, 2.0],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_bool_repeat_ops() {
        let data = TensorData::from([[true, false, false]]);
        let tensor = Tensor::<TestBackend, 2, Bool>::from_data(data, &Default::default());

        let output = tensor.repeat(0, 4);
        let expected = TensorData::from([
            [true, false, false],
            [true, false, false],
            [true, false, false],
            [true, false, false],
        ]);
        output.into_data().assert_eq(&expected, true);
    }

    #[test]
    fn should_support_int_repeat_ops() {
        let data = TensorData::from([[0, 1, 2]]);
        let tensor = Tensor::<TestBackend, 2, Int>::from_data(data, &Default::default());

        let output = tensor.repeat(0, 4);
        let expected = TensorData::from([[0, 1, 2], [0, 1, 2], [0, 1, 2], [0, 1, 2]]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_float_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[1.0, 2.0], [3.0, 4.0]],
            [[5.0, 6.0], [7.0, 8.0]],
            [[9.0, 10.0], [11.0, 12.0]],
            [[13.0, 14.0], [15.0, 16.0]],
        ]);
        let tensor = Tensor::<TestBackend, 3>::from_data(data, &Default::default());

        let output = tensor.repeat(2, 2);
        let expected = TensorData::from([
            [[1.0, 2.0, 1.0, 2.0], [3.0, 4.0, 3.0, 4.0]],
            [[5.0, 6.0, 5.0, 6.0], [7.0, 8.0, 7.0, 8.0]],
            [[9.0, 10.0, 9.0, 10.0], [11.0, 12.0, 11.0, 12.0]],
            [[13.0, 14.0, 13.0, 14.0], [15.0, 16.0, 15.0, 16.0]],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_int_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[1, 2], [3, 4]],
            [[5, 6], [7, 8]],
            [[9, 10], [11, 12]],
            [[13, 14], [15, 16]],
        ]);
        let tensor = Tensor::<TestBackend, 3, Int>::from_data(data, &Default::default());

        let output = tensor.repeat(2, 3);
        let expected = TensorData::from([
            [[1, 2, 1, 2, 1, 2], [3, 4, 3, 4, 3, 4]],
            [[5, 6, 5, 6, 5, 6], [7, 8, 7, 8, 7, 8]],
            [[9, 10, 9, 10, 9, 10], [11, 12, 11, 12, 11, 12]],
            [[13, 14, 13, 14, 13, 14], [15, 16, 15, 16, 15, 16]],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_bool_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[false, true], [true, false]],
            [[true, true], [false, false]],
        ]);
        let tensor = Tensor::<TestBackend, 3, Bool>::from_data(data, &Default::default());

        let output = tensor.repeat(1, 2);
        let expected = TensorData::from([
            [[false, true], [true, false], [false, true], [true, false]],
            [[true, true], [false, false], [true, true], [false, false]],
        ]);

        output.into_data().assert_eq(&expected, true);
    }
}
