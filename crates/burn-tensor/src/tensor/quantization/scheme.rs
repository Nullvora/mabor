use serde::{Deserialize, Serialize};

use crate::{Tensor, TensorPrimitive, backend::Backend};

use super::{
    Calibration, CalibrationRange, QuantizationParameters, QuantizationParametersPrimitive,
};

/// Describes a quantization scheme/configuration.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuantScheme {
    /// Granularity level of quantization (e.g., per-tensor).
    pub level: QuantLevel,
    /// Quantization mode (e.g., symmetric).
    pub mode: QuantMode,
    /// Data type used for storing quantized values (e.g., QInt8).
    pub q_type: QuantInputType,
    /// Precision used for accumulating intermediate values (e.g., during matmul).
    pub acc_precision: QuantAccPrecision,
    /// Whether to propagate quantization to outputs or return unquantized results.
    pub propagation: QuantPropagation,
}

impl Default for QuantScheme {
    fn default() -> Self {
        Self {
            level: QuantLevel::Tensor,
            mode: QuantMode::Symmetric,
            q_type: QuantInputType::QInt8,
            acc_precision: QuantAccPrecision::Full,
            propagation: QuantPropagation::Inhibit,
        }
    }
}

impl QuantScheme {
    /// Set the quantization level.
    pub fn set_level(mut self, level: QuantLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the quantization mode.
    pub fn set_mode(mut self, mode: QuantMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the data type used for quantized values.
    pub fn set_q_type(mut self, q_type: QuantInputType) -> Self {
        self.q_type = q_type;
        self
    }

    /// Set the accumulation precision used during computations.
    pub fn set_acc_precision(mut self, acc_precision: QuantAccPrecision) -> Self {
        self.acc_precision = acc_precision;
        self
    }

    /// Set whether quantization is propagated through operations.
    pub fn set_propagation(mut self, propagation: QuantPropagation) -> Self {
        self.propagation = propagation;
        self
    }
}
/// Level or granularity of quantization.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuantLevel {
    /// Quantize the whole tensor using a single tensor.
    Tensor,
}

/// Data type used to represent quantized values.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuantInputType {
    /// 8-bit signed integer.
    QInt8,
}

/// Strategy used to quantize values.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuantMode {
    /// Symmetric or scale quantization.
    Symmetric,
}

/// Quantization accumulator precision. This is the precision to used when accumulating values
/// while executing algorithms such as matmul.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuantAccPrecision {
    /// Full precision accumulation (f32).
    Full,
    /// Half precision accumulation (f16).
    Half,
}

/// Specify if the output of an operation is quantized using the scheme of the input
/// or returned unquantized.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QuantPropagation {
    /// The output is quantized using the scheme of the input.
    Propagate,
    /// The output is not quantized.
    Inhibit,
}

impl QuantScheme {
    /// Compute the quantization range mapping.
    pub fn compute_range<B: Backend, const D: usize>(
        &self,
        tensor: &Tensor<B, D>,
        calibration: &Calibration,
    ) -> CalibrationRange<B> {
        let (min, max) = match &tensor.primitive {
            TensorPrimitive::Float(tensor) => {
                self.compute_range_primitive::<B>(tensor.clone(), calibration)
            }
            TensorPrimitive::QFloat(_) => unreachable!(),
        };

        CalibrationRange {
            min: Tensor::from_primitive(TensorPrimitive::Float(min)),
            max: Tensor::from_primitive(TensorPrimitive::Float(max)),
        }
    }

    pub(crate) fn compute_range_primitive<B: Backend>(
        &self,
        tensor: B::FloatTensorPrimitive,
        calibration: &Calibration,
    ) -> (B::FloatTensorPrimitive, B::FloatTensorPrimitive) {
        match calibration {
            Calibration::MinMax => match self.level {
                QuantLevel::Tensor => (B::float_min(tensor.clone()), B::float_max(tensor)),
            },
        }
    }

    /// Compute the quantization parameters.
    pub fn compute_q_params<B: Backend>(
        &self,
        range: CalibrationRange<B>,
    ) -> QuantizationParameters<B> {
        match self {
            QuantScheme {
                level: QuantLevel::Tensor,
                mode: QuantMode::Symmetric,
                q_type: QuantInputType::QInt8,
                ..
            } => {
                // Quantized range `[a, b]`
                let b = i8::MAX as i32;
                let a = -b;

                // Compute scale to convert an input value in range `[-alpha, alpha]`
                let values_range = range.min.abs().max_pair(range.max.abs()).mul_scalar(2);

                QuantizationParameters {
                    scale: values_range.div_scalar(b - a),
                    offset: None,
                }
            }
        }
    }

    /// Compute the quantization parameters.
    pub(crate) fn compute_q_params_primitive<B: Backend>(
        &self,
        min: B::FloatTensorPrimitive,
        max: B::FloatTensorPrimitive,
    ) -> QuantizationParametersPrimitive<B> {
        let range = CalibrationRange {
            min: Tensor::from_primitive(TensorPrimitive::Float(min)),
            max: Tensor::from_primitive(TensorPrimitive::Float(max)),
        };
        self.compute_q_params(range).into()
    }
}
