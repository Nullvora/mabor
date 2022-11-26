use crate as burn;

use crate::config::Config;
use crate::module::Module;
use crate::module::Param;
use crate::tensor::backend::Backend;
use crate::tensor::{Distribution, ElementConversion, Shape, Tensor};
use std::ops::Deref;

/// Configuration to create a [Linear](Linear) layer.
#[derive(Config)]
pub struct LinearConfig {
    /// The size of the input features.
    pub d_input: usize,
    /// The size of the output features.
    pub d_output: usize,
    /// If a bias should be applied during the linear transformation.
    #[config(default = true)]
    pub bias: bool,
}

/// Applies a linear transformation to the input tensor:
///
/// `O = IW + b`
#[derive(Module, Debug)]
pub struct Linear<B: Backend> {
    weight: Param<Tensor<B, 2>>,
    bias: Param<Option<Tensor<B, 1>>>,
}

impl<B: Backend> Linear<B> {
    /// Create the module from the given configuration.
    pub fn new(config: &LinearConfig) -> Self {
        // Glorot init
        let start = -1.0 / f64::sqrt(config.d_input as f64);
        let end = 1.0 / f64::sqrt(config.d_input as f64);
        let distribution = Distribution::Uniform(start.to_elem(), end.to_elem());

        let weight = Tensor::random(Shape::new([config.d_input, config.d_output]), distribution);
        let bias = match config.bias {
            true => Some(Tensor::zeros(Shape::new([config.d_output]))),
            false => None,
        };

        Self {
            weight: Param::new(weight),
            bias: Param::new(bias),
        }
    }

    /// Applies the forward pass on the input tensor.
    ///
    /// # Shapes
    ///
    /// - input: [..., any, d_input]
    /// - output: [..., any, d_output]
    pub fn forward<const D: usize>(&self, input: Tensor<B, D>) -> Tensor<B, D> {
        let output = input.matmul(&self.weight.unsqueeze());

        match self.bias.deref() {
            Some(bias) => output + bias.unsqueeze(),
            None => output,
        }
    }
}
