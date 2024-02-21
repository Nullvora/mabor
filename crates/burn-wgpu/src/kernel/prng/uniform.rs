use burn_compute::client::ComputeClient;
use burn_tensor::Shape;

use crate::{
    compute::StaticKernel,
    element::JitElement,
    kernel::{
        prng::base::{make_args_buffer, make_info_buffer},
        prng_workgroup, KernelSettings, SourceTemplate, StaticKernelSource, WORKGROUP_DEFAULT,
    },
    ops::numeric::empty_device,
    tensor::JitTensor,
    Runtime,
};

use super::base::Prng;

struct UniformPrng;

impl StaticKernelSource for UniformPrng {
    fn source() -> SourceTemplate {
        Prng::source().register("num_args", "2").register(
            "prng_loop",
            include_str!("../../template/prng/uniform_inner_loop.wgsl"),
        )
    }
}

/// Pseudo-random generator for the uniform distribution.
pub fn random_uniform<R: Runtime, E: JitElement, const D: usize>(
    shape: Shape<D>,
    device: &R::Device,
    low: E,
    high: E,
) -> JitTensor<R, E, D> {
    let client = R::client(device);
    uniform_kernel(client, device, &shape, low, high)
}

/// Pseudo-random generator for uniform distribution, based on
/// another tensor.
pub fn random_like_uniform<R: Runtime, E: JitElement, const D: usize>(
    tensor: &JitTensor<R, E, D>,
    low: E,
    high: E,
) -> JitTensor<R, E, D> {
    uniform_kernel(
        tensor.client.clone(),
        &tensor.device,
        &tensor.shape,
        low,
        high,
    )
}

fn uniform_kernel<R: Runtime, E: JitElement, const D: usize>(
    client: ComputeClient<R::Server, R::Channel>,
    device: &R::Device,
    shape: &Shape<D>,
    low: E,
    high: E,
) -> JitTensor<R, E, D> {
    const N_VALUES_PER_THREAD: usize = 128;

    let output = empty_device(client.clone(), device.clone(), shape.clone());
    let info_handle = make_info_buffer::<R>(client.clone(), N_VALUES_PER_THREAD);
    let args_handle = make_args_buffer::<R, E>(client.clone(), &[low, high]);
    let workgroup = prng_workgroup(shape.num_elements(), WORKGROUP_DEFAULT, N_VALUES_PER_THREAD);
    let kernel = StaticKernel::<
        KernelSettings<UniformPrng, E, i32, WORKGROUP_DEFAULT, WORKGROUP_DEFAULT, 1>,
    >::new(workgroup);

    client.execute(
        Box::new(kernel),
        &[&output.handle, &info_handle, &args_handle],
    );

    output
}

#[cfg(test)]
mod tests {
    use core::f32;

    use burn_tensor::{backend::Backend, Distribution, Shape, Tensor};
    use serial_test::serial;

    use crate::{kernel::prng::base::tests::calculate_bin_stats, tests::TestBackend, WgpuDevice};

    #[test]
    #[serial]
    fn subsequent_calls_give_different_tensors() {
        TestBackend::seed(0);
        let shape = [4, 5];
        let device = WgpuDevice::default();

        let tensor_1 = Tensor::<TestBackend, 2>::random(shape, Distribution::Default, &device);
        let tensor_2 = Tensor::<TestBackend, 2>::random(shape, Distribution::Default, &device);
        for i in 0..20 {
            assert!(tensor_1.to_data().value[i] != tensor_2.to_data().value[i]);
        }
    }

    #[test]
    #[serial]
    fn values_all_within_interval_default() {
        TestBackend::seed(0);
        let shape = [24, 24];
        let device = WgpuDevice::default();

        let tensor = Tensor::<TestBackend, 2>::random(shape, Distribution::Default, &device);
        tensor.to_data().assert_within_range(0..1);
    }

    #[test]
    #[serial]
    fn values_all_within_interval_uniform() {
        TestBackend::seed(0);
        let shape = [24, 24];
        let device = WgpuDevice::default();

        let tensor =
            Tensor::<TestBackend, 2>::random(shape, Distribution::Uniform(5., 17.), &device);
        tensor.to_data().assert_within_range(5..17);
    }

    #[test]
    #[serial]
    fn at_least_one_value_per_bin_uniform() {
        TestBackend::seed(0);
        let shape = [64, 64];
        let device = WgpuDevice::default();

        let tensor =
            Tensor::<TestBackend, 2>::random(shape, Distribution::Uniform(-5., 10.), &device);
        let numbers = tensor.into_data().value;
        let stats = calculate_bin_stats(numbers, 3, -5., 10.);
        assert!(stats[0].count >= 1);
        assert!(stats[1].count >= 1);
        assert!(stats[2].count >= 1);
    }

    #[test]
    #[serial]
    fn runs_test() {
        TestBackend::seed(0);
        let shape = Shape::new([512, 512]);
        let device = WgpuDevice::default();
        let tensor = Tensor::<TestBackend, 2>::random(shape, Distribution::Default, &device);

        let numbers = tensor.into_data().value;
        let stats = calculate_bin_stats(numbers, 2, 0., 1.);
        let n_0 = stats[0].count as f32;
        let n_1 = stats[1].count as f32;
        let n_runs = (stats[0].n_runs + stats[1].n_runs) as f32;

        let expectation = (2. * n_0 * n_1) / (n_0 + n_1) + 1.0;
        let variance = ((2. * n_0 * n_1) * (2. * n_0 * n_1 - n_0 - n_1))
            / ((n_0 + n_1).powf(2.) * (n_0 + n_1 - 1.));
        let z = (n_runs - expectation) / variance.sqrt();

        // below 2 means we can have good confidence in the randomness
        // we put 2.5 to make sure it passes even when very unlucky
        assert!(z.abs() < 2.5);
    }
}
