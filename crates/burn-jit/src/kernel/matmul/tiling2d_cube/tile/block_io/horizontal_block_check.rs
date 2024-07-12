use burn_cube::prelude::*;

use crate::kernel::matmul::{
    config::CubeTiling2dConfig,
    tiling2d_cube::{
        tile::{
            loader::{CheckBounds, ReadTileInfo},
            memory_access::{
                ContiguousAccess, StridedAccess, UnmatchingVectorization, WritePositions,
                WritePositionsExpand,
            },
        },
        write_output::WriteTileInfo,
    },
};

use super::base::{all_zeros_comptime, all_zeros_runtime, BlockLoader, BlockWriter};

pub(crate) struct HorizontalCheckBlockIO;

#[cube]
impl<F: Float> BlockLoader<F> for HorizontalCheckBlockIO {
    fn load_tile_plain<A: ContiguousAccess<F>>(
        tensor: &Tensor<F>,
        shared_memory: &mut SharedMemory<F>,
        info: ReadTileInfo,
        config: Comptime<CubeTiling2dConfig>,
        check_bounds: CheckBounds,
    ) {
        let tile_size = Comptime::map(config, |c| c.tile_size);
        let vectorization = Comptime::vectorization(&tensor);
        let unroll = Comptime::map(config, |c| c.unroll_tile);

        let col = check_bounds.skip_col + info.read_col;
        if check_bounds.dim_horizontal > col {
            for i in range(0u32, Comptime::get(tile_size), unroll) {
                let gm_position =
                    (info.gm_position_base + i * info.gm_stride) / Comptime::runtime(vectorization);
                let sm_position =
                    (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

                shared_memory[sm_position] =
                    A::read_contiguous_checked(tensor, gm_position, check_bounds, info, config);
            }
        } else {
            all_zeros_comptime(shared_memory, info.sm_position_base, info.sm_stride, config);
        }
    }

    fn load_tile_transposed(
        tensor: &Tensor<F>,
        shared_memory: &mut SharedMemory<F>,
        info: ReadTileInfo,
        config: Comptime<CubeTiling2dConfig>,
        check_bounds: CheckBounds,
    ) {
        let tile_size = Comptime::map(config, |c| c.tile_size);

        let mut num_reads = UInt::new(0);
        let col = check_bounds.skip_col + info.read_col;
        let dim_horizontal = check_bounds.dim_horizontal;
        if dim_horizontal > col {
            num_reads = UInt::min(dim_horizontal - col, Comptime::runtime(tile_size));
        }

        for i in range(0u32, num_reads, Comptime::new(false)) {
            let gm_position = info.gm_position_base + i;
            let sm_position =
                (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

            shared_memory[sm_position] = UnmatchingVectorization::read_strided_unchecked(
                tensor,
                gm_position,
                info.gm_stride,
                config,
            );
        }

        all_zeros_runtime(
            shared_memory,
            num_reads,
            info.sm_position_base,
            info.sm_stride,
            config,
        );
    }
}

#[cube]
impl<F: Float> BlockWriter<F> for HorizontalCheckBlockIO {
    fn write_output<A: ContiguousAccess<F>>(
        out: &mut Tensor<F>,
        results: &Array<F>,
        info: WriteTileInfo,
        config: Comptime<CubeTiling2dConfig>,
        check_bounds: CheckBounds,
    ) {
        let tile_size = Comptime::map(config, |c| c.tile_size);
        let unroll = Comptime::map(config, |c| c.unroll_tile);
        let coordinates = info.coordinates;

        let col = coordinates.skip_col + coordinates.unit_col;

        if check_bounds.dim_horizontal > col {
            let row = coordinates.skip_row + coordinates.unit_row;
            let out_position_base = row * info.out_stride + col + info.offset_output;

            for result_index in range(0u32, Comptime::get(tile_size), unroll) {
                let positions = WritePositions {
                    result: result_index * Comptime::runtime(tile_size),
                    out: out_position_base + result_index * info.out_stride,
                };

                A::write_contiguous_checked(out, results, positions, check_bounds, col, config);
            }
        }
    }
}
