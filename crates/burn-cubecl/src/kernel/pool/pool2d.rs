use core::hash::Hash;
use cubecl::prelude::*;

pub trait Pool2dDirectStrategyFamily: Send + Sync + 'static {
    type Indices: LaunchArg;
    type Config: CubeType + Clone + Send + Sync + core::fmt::Debug + Hash + core::cmp::Eq;
    type Pool2d<N: Numeric>: Pool2dDirectStrategy<N, Config = Self::Config, Indices = Self::Indices>;
}

#[cube]
pub(crate) trait Pool2dDirectStrategy<N: Numeric>: Send + Sync + 'static {
    type Accumulator: CubeType;
    type Config: CubeType + Clone + Send + Sync + core::fmt::Debug + Hash + core::cmp::Eq;

    type Indices: LaunchArg;

    fn initialize(
        #[comptime] config: &Self::Config,
        #[comptime] line_size: u32,
    ) -> Self::Accumulator;

    fn accumulate(
        #[comptime] config: &Self::Config,
        accumulator: &mut Self::Accumulator,
        index: u32,
        result: Line<N>,
    );

    fn store(
        #[comptime] config: &Self::Config,
        position: u32,
        output: &mut Tensor<Line<N>>,
        output_indices: &mut Self::Indices,
        accumulator: Self::Accumulator,
    );
}

#[derive(CubeLaunch, CubeType)]
pub struct Pool2dDirectArgs {
    pub strides_0: u32,
    pub strides_1: u32,
    pub dilation_0: u32,
    pub dilation_1: u32,
    pub padding_0: u32,
    pub padding_1: u32,
}

#[cube(launch)]
pub fn pool2d_direct<E: Numeric, S: Pool2dDirectStrategyFamily>(
    input: &Tensor<Line<E>>,
    output: &mut Tensor<Line<E>>,
    indices: &mut S::Indices,
    args: &Pool2dDirectArgs,
    #[comptime] kernel_size: (u32, u32),
    #[comptime] config: &S::Config,
) {
    if ABSOLUTE_POS >= output.len() {
        terminate!();
    }

    let (out_h, out_w, channels) = (output.shape(1), output.shape(2), output.shape(3));
    let channel_lines = channels / input.line_size();
    let (in_stride_b, in_stride_h, in_stride_w, in_stride_c) = (
        input.stride(0),
        input.stride(1),
        input.stride(2),
        input.stride(3),
    );
    let (in_h, in_w) = (input.shape(1), input.shape(2));

    let c = (ABSOLUTE_POS % channel_lines) * input.line_size();
    let pos = ABSOLUTE_POS / channel_lines;
    let ow = pos % out_w;
    let pos = pos / out_w;
    let oh = pos % out_h;
    let b = pos / out_h;

    let mut accumulator = S::Pool2d::<E>::initialize(config, input.line_size());

    let in_b_off = b * in_stride_b;
    let in_c_off = c * in_stride_c;

    let border_bottom = in_h + args.padding_0;
    let border_right = in_w + args.padding_1;

    for kh in 0..kernel_size.0 {
        let ih = oh * args.strides_0 + kh * args.dilation_0;
        let within_padding_h = ih >= args.padding_0 && ih < border_bottom;

        for kw in 0..kernel_size.1 {
            let iw = ow * args.strides_1 + kw * args.dilation_1;
            let within_padding_w = iw >= args.padding_1 && iw < border_right;

            if within_padding_h && within_padding_w {
                let ih_pad = ih - args.padding_0;
                let iw_pad = iw - args.padding_1;

                let in_h_off = ih_pad * in_stride_h;
                let in_w_off = iw_pad * in_stride_w;

                let index_input = in_b_off + in_c_off + in_h_off + in_w_off;

                S::Pool2d::<E>::accumulate(
                    config,
                    &mut accumulator,
                    ih_pad * in_w + iw_pad,
                    input[index_input / input.line_size()],
                );
            }
        }
    }

    S::Pool2d::<E>::store(config, ABSOLUTE_POS, output, indices, accumulator);
}
