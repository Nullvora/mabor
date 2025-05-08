#![allow(missing_docs)] // cube derive macros

use burn_tensor::quantization::{QuantInputType, QuantLevel, QuantMode, QuantScheme};
use cubecl::prelude::*;

/// Quantization parameters.
#[derive(CubeLaunch, CubeType)]
pub struct QParams {
    #[cube(comptime)]
    scheme: QuantScheme,
}

/// Quantized tensor representation.
pub type QTensor = Array<Line<u32>>;

#[cube]
impl QParams {
    /// Create a new quantization parameters instance.
    pub fn new(#[comptime] scheme: QuantScheme) -> Self {
        QParams { scheme }
    }

    /// Get the quantization parameters values.
    pub fn values(&self, tensor: &QTensor) -> (f32, i32) {
        let len = tensor.len();
        match comptime!(self.scheme) {
            // Symmetric quantization only contains the scaling factor as the last element
            QuantScheme {
                level: QuantLevel::Tensor,
                mode: QuantMode::Symmetric,
                q_type: QuantInputType::QInt8,
                ..
            } => (f32::reinterpret(tensor[len - 1][tensor.line_size() - 1]), 0),
        }
    }
}
