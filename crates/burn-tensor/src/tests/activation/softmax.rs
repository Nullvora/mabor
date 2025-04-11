#[burn_tensor_testgen::testgen(softmax)]
mod tests {
    use super::*;
    use burn_tensor::{Tensor, TensorData, activation};
    use burn_tensor::{Tolerance, ops::FloatElem};
    type FT = FloatElem<TestBackend>;

    #[test]
    fn test_softmax_d2() {
        let tensor = TestTensor::<2>::from([[1.0, 7.0], [13.0, -3.0]]);

        let output = activation::softmax(tensor, 1);
        let expected = TensorData::from([[2.472623e-03, 9.975274e-01], [1.0, 1.125352e-07]]);

        output
            .into_data()
            .assert_approx_eq::<FT>(&expected, Tolerance::default());
    }
}
