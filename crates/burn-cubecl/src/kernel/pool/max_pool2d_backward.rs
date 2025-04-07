use crate::{
    CubeRuntime, IntElement,
    element::CubeElement,
    kernel::conv::permute_nchw_to_nhwc,
    ops::{max_vectorization, numeric::empty_device, permute},
    tensor::CubeTensor,
};
use burn_tensor::Shape;
use cubecl::{calculate_cube_count_elemwise, prelude::*};

use super::{PoolBackwardArgs, PoolBackwardArgsLaunch};

#[cube(launch_unchecked)]
fn max_pool2d_with_indices_backward_kernel<E: Numeric, I: Int>(
    grad: &Tensor<Line<E>>,
    indices: &Tensor<Line<I>>,
    output: &mut Tensor<Line<E>>,
    args: &PoolBackwardArgs,
    #[comptime] kernel_size_0: i32,
    #[comptime] kernel_size_1: i32,
) {
    if ABSOLUTE_POS >= output.len() {
        terminate!();
    }

    let line_size = grad.line_size();

    let channels = output.shape(3) / line_size;
    let channel = (ABSOLUTE_POS % channels) * output.line_size();
    let pos = ABSOLUTE_POS / channels;
    let iw = pos % output.shape(2);
    let pos = pos / output.shape(2);
    let ih = pos % output.shape(1);
    let batch = pos / output.shape(1);

    let index_current = ih * output.shape(2) + iw;

    let (oh_start, oh_end, ow_start, ow_end) = loop_ranges(
        ih as i32,
        iw as i32,
        grad.shape(1),
        grad.shape(2),
        args,
        kernel_size_0,
        kernel_size_1,
    );

    let mut grad_acc = Line::empty(grad.line_size()).fill(E::from_int(0));

    let index_base = batch * grad.stride(0) + channel * grad.stride(3);

    for oh in oh_start..oh_end {
        for ow in ow_start..ow_end {
            let index = index_base + oh * grad.stride(1) + ow * grad.stride(2);
            let index_max = Line::<u32>::cast_from(indices[index / line_size]);

            grad_acc += select_many(
                index_max.equal(Line::cast_from(index_current)),
                grad[index / line_size],
                Line::new(E::from_int(0)),
            );
        }
    }

    output[ABSOLUTE_POS] = grad_acc;
}

#[cube]
fn loop_ranges(
    ih: i32,
    iw: i32,
    grad_h: u32,
    grad_w: u32,
    args: &PoolBackwardArgs,
    #[comptime] kernel_size_0: i32,
    #[comptime] kernel_size_1: i32,
) -> (u32, u32, u32, u32) {
    let kms_0 = args.dilation_0 * kernel_size_0 - args.stride_0;
    let kms_1 = args.dilation_1 * kernel_size_1 - args.stride_1;

    let oh_start = Max::max((ih + args.padding_0 - kms_0) / args.stride_0, 0) as u32;
    let ow_start = Max::max((iw + args.padding_1 - kms_1) / args.stride_1, 0) as u32;
    let oh_end = Min::min(Max::max(kms_0, 0) as u32 + oh_start, grad_h - 1) + 1;
    let ow_end = Min::min(Max::max(kms_1, 0) as u32 + ow_start, grad_w - 1) + 1;

    (oh_start, oh_end, ow_start, ow_end)
}

pub(crate) fn max_pool2d_with_indices_backward<R: CubeRuntime, E: CubeElement, I: IntElement>(
    x: CubeTensor<R>,
    grad: CubeTensor<R>,
    indices: CubeTensor<R>,
    kernel_size: [usize; 2],
    stride: [usize; 2],
    padding: [usize; 2],
    dilation: [usize; 2],
) -> CubeTensor<R> {
    let [batches, channels, height, width] = x.shape.dims();

    let grad = permute_nchw_to_nhwc::<R, E>(grad);
    let indices = permute_nchw_to_nhwc::<R, I>(indices);

    let line_size = if grad.strides[3] == indices.strides[3] {
        max_vectorization(&grad)
    } else {
        1
    };

    let out_shape = Shape::new([batches, height, width, channels]);
    let output = empty_device::<R, E>(x.client.clone(), x.device.clone(), out_shape);
    let cube_dim = CubeDim::default();
    let cube_count =
        calculate_cube_count_elemwise(output.shape.num_elements() / line_size as usize, cube_dim);

    unsafe {
        max_pool2d_with_indices_backward_kernel::launch_unchecked::<E, I, R>(
            &x.client,
            cube_count,
            cube_dim,
            grad.as_tensor_arg::<E>(line_size),
            indices.as_tensor_arg::<I>(line_size),
            output.as_tensor_arg::<E>(line_size),
            PoolBackwardArgsLaunch::new(
                ScalarArg::new(stride[0] as i32),
                ScalarArg::new(stride[1] as i32),
                ScalarArg::new(dilation[0] as i32),
                ScalarArg::new(dilation[0] as i32),
                ScalarArg::new(padding[0] as i32),
                ScalarArg::new(padding[1] as i32),
            ),
            kernel_size[0] as i32,
            kernel_size[1] as i32,
        )
    };

    permute(output, &[0, 3, 1, 2])
}
