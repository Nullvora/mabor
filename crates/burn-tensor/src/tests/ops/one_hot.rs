#[burn_tensor_testgen::testgen(one_hot)]
mod tests {
    use super::*;
    use burn_tensor::{
        Float, Int, Numeric, Shape, Tensor, TensorData, as_type,
        backend::Backend,
        tests::{Float as _, Int as _},
    };

    #[test]
    fn float_should_support_one_hot() {
        let tensor = TestTensor::<1>::from([0.0, 1.0, 4.0]);
        let one_hot_tensor: Tensor<TestBackend, 2, Float> = tensor.one_hot(5);
        let expected = TensorData::from([
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
        ]);
        one_hot_tensor.into_data().assert_eq(&expected, false);
    }

    #[test]
    fn float_should_support_one_hot_index() {
        let tensor = TestTensor::<1>::from([2.0]);
        let one_hot_tensor: Tensor<TestBackend, 2> = tensor.one_hot::<2>(10);
        let expected = TensorData::from([[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]);
        one_hot_tensor.into_data().assert_eq(&expected, false);
    }

    #[test]
    #[should_panic]
    fn float_one_hot_should_panic_when_index_exceeds_number_of_classes() {
        let tensor = TestTensor::<1>::from([5.0]);
        let result: Tensor<TestBackend, 2> = tensor.one_hot(5);
    }

    #[test]
    #[should_panic]
    fn float_one_hot_should_panic_when_number_of_classes_is_zero() {
        let tensor = TestTensor::<1>::from([0.0]);
        let result: Tensor<TestBackend, 2> = tensor.one_hot(0);
    }

    #[test]
    fn int_should_support_one_hot() {
        let tensor = TestTensorInt::<1>::from([0, 1, 4]);
        let one_hot_tensor: Tensor<TestBackend, 2, Int> = tensor.one_hot(5);
        let expected = TensorData::from([[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 0, 0, 1]]);
        one_hot_tensor.into_data().assert_eq(&expected, false);
    }

    #[test]
    #[should_panic]
    fn int_one_hot_should_panic_when_index_exceeds_number_of_classes() {
        let tensor = TestTensorInt::<1>::from([5]);
        let result: Tensor<TestBackend, 2, Int> = tensor.one_hot(5);
    }

    #[test]
    #[should_panic]
    fn int_one_hot_should_panic_when_number_of_classes_is_zero() {
        let tensor = TestTensorInt::<1>::from([2]);
        let result: Tensor<TestBackend, 2, Int> = tensor.one_hot(0);
    }

    #[test]
    fn one_hot_fill_with_positive_axis_and_indices() {
        let tensor = TestTensorInt::<2>::from([[1, 9], [2, 4]]);
        let expected = TensorData::from(as_type!(IntType: [
            [[1, 1], [3, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1], [1, 3]],
            [[1, 1], [1, 1], [3, 1], [1, 1], [1, 3], [1, 1], [1, 1], [1, 1], [1, 1], [1, 1]]
        ]));

        let one_hot_tensor: Tensor<TestBackend, 3, Int> = tensor.one_hot_fill(10, 3.0, 1.0, 1);

        one_hot_tensor.into_data().assert_eq(&expected, true);
    }

    #[test]
    fn one_hot_fill_with_negative_axis_and_indices() {
        let tensor = TestTensor::<2>::from([[0, 2], [1, -1]]);
        let expected = TensorData::from(as_type!(FloatType: [
            [[5.0, 0.0, 0.0], [0.0, 0.0, 5.0]],
            [[0.0, 5.0, 0.0], [0.0, 0.0, 5.0]]
        ]));

        let one_hot_tensor: Tensor<TestBackend, 3> = tensor.one_hot_fill(3, 5.0, 0.0, -1);

        one_hot_tensor.into_data().assert_eq(&expected, true);
    }

    #[test]
    fn one_hot_fill_with_negative_indices() {
        let tensor = TestTensor::<1>::from([0.0, -7.0, -8.0]);
        let expected = TensorData::from(as_type!(FloatType: [
            [3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            [1.0, 1.0, 3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
        ]));

        let one_hot_tensor: Tensor<TestBackend, 2> = tensor.one_hot_fill(10, 3.0, 1.0, 1);

        one_hot_tensor.into_data().assert_eq(&expected, true);
    }

    #[should_panic]
    #[test]
    fn one_hot_fill_should_panic_when_axis_out_range_of_rank() {
        let tensor = TestTensor::<2>::from([[0.0, 2.0], [1.0, -1.0]]);

        let one_hot_tensor: Tensor<TestBackend, 3, Float> = tensor.one_hot_fill(2, 5.0, 0.0, 3);
    }
}
