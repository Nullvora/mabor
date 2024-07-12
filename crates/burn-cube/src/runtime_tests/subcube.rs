use crate::{self as burn_cube, Feature};
use burn_cube::prelude::*;

#[cube(launch)]
pub fn kernel_sum<F: Float>(output: &mut Tensor<F>) {
    let val = output[UNIT_POS];
    let val2 = subcube_sum::<F>(val);

    if UNIT_POS == UInt::new(0) {
        output[0] = val2;
    }
}

#[cube(launch)]
pub fn kernel_prod<F: Float>(output: &mut Tensor<F>) {
    let val = output[UNIT_POS];
    let val2 = subcube_prod::<F>(val);

    if UNIT_POS == UInt::new(0) {
        output[0] = val2;
    }
}

#[cube(launch)]
pub fn kernel_max<F: Float>(output: &mut Tensor<F>) {
    let val = output[UNIT_POS];
    let val2 = subcube_max::<F>(val);

    if UNIT_POS == UInt::new(0) {
        output[0] = val2;
    }
}

#[cube(launch)]
pub fn kernel_min<F: Float>(output: &mut Tensor<F>) {
    let val = output[UNIT_POS];
    let val2 = subcube_min::<F>(val);

    if UNIT_POS == UInt::new(0) {
        output[0] = val2;
    }
}

pub fn test_subcube_sum<TestRuntime: Runtime>(
    client: ComputeClient<TestRuntime::Server, TestRuntime::Channel>,
) {
    test_subcube_operation::<TestRuntime, _>(
        &[4.0, 5.0, 7.0, 1.0],
        &[17.0, 5.0, 7.0, 1.0],
        client.clone(),
        |cube_count, cube_dim, handle| {
            kernel_sum::launch::<F32, TestRuntime>(client.clone(), cube_count, cube_dim, handle)
        },
    );
}

pub fn test_subcube_prod<TestRuntime: Runtime>(
    client: ComputeClient<TestRuntime::Server, TestRuntime::Channel>,
) {
    test_subcube_operation::<TestRuntime, _>(
        &[4.0, 5.0, 7.0, 1.0],
        &[140.0, 5.0, 7.0, 1.0],
        client.clone(),
        |cube_dim, settings, handle| {
            kernel_prod::launch::<F32, TestRuntime>(client.clone(), cube_dim, settings, handle)
        },
    );
}
pub fn test_subcube_max<TestRuntime: Runtime>(
    client: ComputeClient<TestRuntime::Server, TestRuntime::Channel>,
) {
    test_subcube_operation::<TestRuntime, _>(
        &[4.0, 5.0, 7.0, 1.0],
        &[7.0, 5.0, 7.0, 1.0],
        client.clone(),
        |cube_dim, settings, handle| {
            kernel_max::launch::<F32, TestRuntime>(client.clone(), cube_dim, settings, handle)
        },
    );
}

pub fn test_subcube_min<TestRuntime: Runtime>(
    client: ComputeClient<TestRuntime::Server, TestRuntime::Channel>,
) {
    test_subcube_operation::<TestRuntime, _>(
        &[4.0, 5.0, 7.0, 1.0],
        &[1.0, 5.0, 7.0, 1.0],
        client.clone(),
        |cube_dim, settings, handle| {
            kernel_min::launch::<F32, TestRuntime>(client.clone(), cube_dim, settings, handle)
        },
    );
}

fn test_subcube_operation<TestRuntime: Runtime, Launch>(
    input: &[f32],
    expected: &[f32],
    client: ComputeClient<TestRuntime::Server, TestRuntime::Channel>,
    launch: Launch,
) where
    Launch: Fn(CubeCount<TestRuntime::Server>, CubeDim, TensorArg<'_, TestRuntime>),
{
    if !client.features().enabled(Feature::Subcube) {
        // Can't execute the test.
        return;
    }

    let handle = client.create(f32::as_bytes(input));
    let (shape, strides) = ([input.len()], [1]);

    launch(
        CubeCount::Static(1, 1, 1),
        CubeDim::new(input.len() as u32, 1, 1),
        TensorArg::new(&handle, &strides, &shape),
    );

    let actual = client.read(handle.binding());
    let actual = f32::from_bytes(&actual);

    assert_eq!(actual, expected);
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! testgen_subcube {
    () => {
        use super::*;

        #[test]
        fn test_subcube_sum() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::subcube::test_subcube_sum::<TestRuntime>(client);
        }

        #[test]
        fn test_subcube_prod() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::subcube::test_subcube_prod::<TestRuntime>(client);
        }

        #[test]
        fn test_subcube_max() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::subcube::test_subcube_max::<TestRuntime>(client);
        }

        #[test]
        fn test_subcube_min() {
            let client = TestRuntime::client(&Default::default());
            burn_cube::runtime_tests::subcube::test_subcube_max::<TestRuntime>(client);
        }
    };
}
