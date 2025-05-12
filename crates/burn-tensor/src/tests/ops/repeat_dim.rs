#[burn_tensor_testgen::testgen(repeat_dim)]
mod tests {
    use super::*;
    use burn_tensor::{Bool, Int, Tensor, TensorData, backend::Backend};

    #[test]
    fn should_support_repeat_ops() {
        let data = TensorData::from([[0.0f64, 1.0f64, 2.0f64]]);
        let tensor = TestTensor::<2>::from_data(data.clone(), &Default::default());

        let output = tensor.repeat_dim(0, 4);
        let expected = TensorData::from([
            [0.0f32, 1.0f32, 2.0f32],
            [0.0f32, 1.0f32, 2.0f32],
            [0.0f32, 1.0f32, 2.0f32],
            [0.0f32, 1.0f32, 2.0f32],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_bool_repeat_ops() {
        let data = TensorData::from([[true, false, false]]);
        let tensor = TestTensorBool::<2>::from_data(data, &Default::default());

        let output = tensor.repeat_dim(0, 4);
        let expected = TensorData::from([
            [true, false, false],
            [true, false, false],
            [true, false, false],
            [true, false, false],
        ]);
        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_int_repeat_ops() {
        let data = TensorData::from([[0, 1, 2]]);
        let tensor = TestTensorInt::<2>::from_data(data, &Default::default());

        let output = tensor.repeat_dim(0, 4);
        let expected = TensorData::from([[0, 1, 2], [0, 1, 2], [0, 1, 2], [0, 1, 2]]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_float_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[1.0f32, 2.0f32], [3.0f32, 4.0f32]],
            [[5.0f32, 6.0f32], [7.0f32, 8.0f32]],
            [[9.0f32, 10.0f32], [11.0f32, 12.0f32]],
            [[13.0f32, 14.0f32], [15.0f32, 16.0f32]],
        ]);
        let tensor = TestTensor::<3>::from_data(data, &Default::default());

        let output = tensor.repeat_dim(2, 2);
        let expected = TensorData::from([
            [
                [1.0f32, 2.0f32, 1.0f32, 2.0f32],
                [3.0f32, 4.0f32, 3.0f32, 4.0f32],
            ],
            [
                [5.0f32, 6.0f32, 5.0f32, 6.0f32],
                [7.0f32, 8.0f32, 7.0f32, 8.0f32],
            ],
            [
                [9.0f32, 10.0f32, 9.0f32, 10.0f32],
                [11.0f32, 12.0f32, 11.0f32, 12.0f32],
            ],
            [
                [13.0f32, 14.0f32, 13.0f32, 14.0f32],
                [15.0f32, 16.0f32, 15.0f32, 16.0f32],
            ],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_int_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[1i32, 2i32], [3i32, 4i32]],
            [[5i32, 6i32], [7i32, 8i32]],
            [[9i32, 10i32], [11i32, 12i32]],
            [[13i32, 14i32], [15i32, 16i32]],
        ]);
        let tensor = TestTensorInt::<3>::from_data(data, &Default::default());

        let output = tensor.repeat_dim(2, 3);
        let expected = TensorData::from([
            [
                [1i32, 2i32, 1i32, 2i32, 1i32, 2i32],
                [3i32, 4i32, 3i32, 4i32, 3i32, 4i32],
            ],
            [
                [5i32, 6i32, 5i32, 6i32, 5i32, 6i32],
                [7i32, 8i32, 7i32, 8i32, 7i32, 8i32],
            ],
            [
                [9i32, 10i32, 9i32, 10i32, 9i32, 10i32],
                [11i32, 12i32, 11i32, 12i32, 11i32, 12i32],
            ],
            [
                [13i32, 14i32, 13i32, 14i32, 13i32, 14i32],
                [15i32, 16i32, 15i32, 16i32, 15i32, 16i32],
            ],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn should_support_bool_repeat_on_dims_larger_than_1() {
        let data = TensorData::from([
            [[false, true], [true, false]],
            [[true, true], [false, false]],
        ]);
        let tensor = TestTensorBool::<3>::from_data(data, &Default::default());

        let output = tensor.repeat_dim(1, 2);
        let expected = TensorData::from([
            [[false, true], [true, false], [false, true], [true, false]],
            [[true, true], [false, false], [true, true], [false, false]],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn repeat_dim_swap_dims_1() {
        let tensor = TestTensorInt::arange(0..16, &Default::default()).float();

        let tensor = tensor.reshape([4, 1, 4]);
        let tensor = tensor.swap_dims(0, 2);
        let output = tensor.repeat_dim(1, 4);

        let expected = TensorData::from([
            [
                [0.0, 4.0, 8.0, 12.0],
                [0.0, 4.0, 8.0, 12.0],
                [0.0, 4.0, 8.0, 12.0],
                [0.0, 4.0, 8.0, 12.0],
            ],
            [
                [1.0, 5.0, 9.0, 13.0],
                [1.0, 5.0, 9.0, 13.0],
                [1.0, 5.0, 9.0, 13.0],
                [1.0, 5.0, 9.0, 13.0],
            ],
            [
                [2.0, 6.0, 10.0, 14.0],
                [2.0, 6.0, 10.0, 14.0],
                [2.0, 6.0, 10.0, 14.0],
                [2.0, 6.0, 10.0, 14.0],
            ],
            [
                [3.0, 7.0, 11.0, 15.0],
                [3.0, 7.0, 11.0, 15.0],
                [3.0, 7.0, 11.0, 15.0],
                [3.0, 7.0, 11.0, 15.0],
            ],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn repeat_dim_swap_dims_2() {
        let tensor = TestTensorInt::arange(0..16, &Default::default()).float();

        let tensor = tensor.reshape([2, 2, 1, 4]);
        let tensor = tensor.swap_dims(0, 1);
        let output = tensor.repeat_dim(2, 4);

        let expected = TensorData::from([
            [
                [
                    [0.0, 1.0, 2.0, 3.0],
                    [0.0, 1.0, 2.0, 3.0],
                    [0.0, 1.0, 2.0, 3.0],
                    [0.0, 1.0, 2.0, 3.0],
                ],
                [
                    [8.0, 9.0, 10.0, 11.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [8.0, 9.0, 10.0, 11.0],
                ],
            ],
            [
                [
                    [4.0, 5.0, 6.0, 7.0],
                    [4.0, 5.0, 6.0, 7.0],
                    [4.0, 5.0, 6.0, 7.0],
                    [4.0, 5.0, 6.0, 7.0],
                ],
                [
                    [12.0, 13.0, 14.0, 15.0],
                    [12.0, 13.0, 14.0, 15.0],
                    [12.0, 13.0, 14.0, 15.0],
                    [12.0, 13.0, 14.0, 15.0],
                ],
            ],
        ]);

        output.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn repeat_dim_swap_dims_3() {
        let tensor = TestTensorInt::arange(0..16, &Default::default()).float();

        let tensor = tensor.reshape([1, 2, 2, 4]);
        let tensor = tensor.swap_dims(0, 2);
        let tensor = tensor.swap_dims(1, 3);
        let output = tensor.repeat_dim(2, 4);

        let expected = TensorData::from([
            [
                [[0.0, 8.0], [0.0, 8.0], [0.0, 8.0], [0.0, 8.0]],
                [[1.0, 9.0], [1.0, 9.0], [1.0, 9.0], [1.0, 9.0]],
                [[2.0, 10.0], [2.0, 10.0], [2.0, 10.0], [2.0, 10.0]],
                [[3.0, 11.0], [3.0, 11.0], [3.0, 11.0], [3.0, 11.0]],
            ],
            [
                [[4.0, 12.0], [4.0, 12.0], [4.0, 12.0], [4.0, 12.0]],
                [[5.0, 13.0], [5.0, 13.0], [5.0, 13.0], [5.0, 13.0]],
                [[6.0, 14.0], [6.0, 14.0], [6.0, 14.0], [6.0, 14.0]],
                [[7.0, 15.0], [7.0, 15.0], [7.0, 15.0], [7.0, 15.0]],
            ],
        ]);
        output.into_data().assert_eq(&expected, false);
    }
}
