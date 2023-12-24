use crate::{
    codegen::{
        ComputeShader, Elem, ElemWiseKernelCodegen, Input, Operator, Output, ReadingStrategy,
        Visibility,
    },
    fusion::{
        cache::{FusedKernelSource, KernelCompilationCache},
        kernel,
    },
    FloatElement, GraphicsApi, IntElement, Wgpu,
};
use burn_fusion::{graph::Context, Optimization, TensorDescription};
use burn_tensor::Device;

pub(crate) struct ElementWise<G, F, I>
where
    G: GraphicsApi,
    F: FloatElement,
    I: IntElement,
{
    pub(crate) id: String,
    pub(crate) inputs: Vec<(TensorDescription, Elem)>,
    pub(crate) outputs: Vec<(TensorDescription, Elem)>,
    pub(crate) locals: Vec<u16>,
    pub(crate) operators: Vec<Operator>,
    pub(crate) scalars_f32: usize,
    pub(crate) scalars_u32: usize,
    pub(crate) scalars_i32: usize,
    pub(crate) device: Device<Wgpu<G, F, I>>,
    pub(crate) cache: KernelCompilationCache,
}

impl<G, F, I> ElementWise<G, F, I>
where
    G: GraphicsApi,
    F: FloatElement,
    I: IntElement,
{
    pub fn compile(&mut self) -> ComputeShader {
        let mut inputs = self
            .inputs
            .iter()
            .map(|(_tensor, elem)| Input::Array {
                elem: *elem,
                visibility: Visibility::Read,
                strategy: ReadingStrategy::OutputLayout,
            })
            .collect::<Vec<_>>();

        let outputs = self
            .outputs
            .iter()
            .zip(self.locals.iter())
            .map(|((_tensor, elem), local)| Output::Array {
                elem: *elem,
                local: *local,
            })
            .collect::<Vec<_>>();

        if self.scalars_f32 > 0 {
            inputs.push(Input::Scalar {
                elem: Elem::F32,
                size: self.scalars_f32,
            })
        }

        if self.scalars_u32 > 0 {
            inputs.push(Input::Scalar {
                elem: Elem::U32,
                size: self.scalars_u32,
            })
        }

        if self.scalars_i32 > 0 {
            inputs.push(Input::Scalar {
                elem: Elem::I32,
                size: self.scalars_i32,
            })
        }

        ElemWiseKernelCodegen::new()
            .inputs(&inputs)
            .body(&self.operators)
            .outputs(&outputs)
            .compile()
    }
}

impl<G, F, I> Optimization<Wgpu<G, F, I>> for ElementWise<G, F, I>
where
    G: GraphicsApi,
    F: FloatElement,
    I: IntElement,
{
    fn execute(&mut self, context: &mut Context<'_, Wgpu<G, F, I>>) {
        if let Some(kernel) = self.cache.get(&self.id) {
            kernel::execute_fusion(
                &self.inputs.iter().map(|a| &a.0).collect::<Vec<_>>(),
                &self.outputs.iter().map(|a| &a.0).collect::<Vec<_>>(),
                self.scalars_f32,
                self.scalars_i32,
                kernel,
                context,
                self.device.clone(),
            );
        } else {
            let shader = self.compile();

            kernel::execute_fusion(
                &self.inputs.iter().map(|a| &a.0).collect::<Vec<_>>(),
                &self.outputs.iter().map(|a| &a.0).collect::<Vec<_>>(),
                self.scalars_f32,
                self.scalars_i32,
                FusedKernelSource::NewKernel {
                    id: self.id.to_string(),
                    shader,
                },
                context,
                self.device.clone(),
            );

            self.cache.insert(self.id.clone());
        }
    }

    fn len(&self) -> usize {
        self.operators.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn_fusion::graph::Ops;
    use burn_fusion::{Fusion, FusionBackend};
    use burn_tensor::Int;
    use burn_tensor::{backend::Backend, Data, Tensor};

    #[test]
    fn test_fusion_same_behavior() {
        type Backend = Wgpu;
        type FusedBackend = Fusion<Wgpu>;

        let data_1 =
            Tensor::<FusedBackend, 2>::random_devauto([1, 32], burn_tensor::Distribution::Default)
                .into_data();
        let data_2 =
            Tensor::<Backend, 2>::random_devauto([32, 32], burn_tensor::Distribution::Default)
                .into_data();

        let result_ref = execute::<Backend>(
            data_1.clone(),
            data_2.clone(),
            ImplementationDetails::Variant1,
        );
        let result_fused = execute::<FusedBackend>(
            data_1.clone(),
            data_2.clone(),
            ImplementationDetails::Variant1,
        );

        result_ref.assert_approx_eq(&result_fused, 3);
    }

    #[test]
    fn test_fusion_same_behavior_int() {
        let data_1 = Tensor::<FusedBackend, 2>::random(
            [32, 32],
            burn_tensor::Distribution::Default,
            &Default::default(),
        )
        .into_data();
        let data_2 = Tensor::<Backend, 2>::random(
            [32, 32],
            burn_tensor::Distribution::Default,
            &Default::default(),
        )
        .into_data()
        .convert();

        fn func<B: burn_tensor::backend::Backend>(
            data1: Data<f32, 2>,
            data2: Data<i32, 2>,
        ) -> Data<f32, 2> {
            let x = Tensor::<B, 2>::from_data(data1.convert(), &Default::default());
            let y = Tensor::<B, 2, Int>::from_data(data2.convert(), &Default::default());

            let x_1 = x.clone().powf(2.0);
            let x_1 = x_1 + x;
            let y_1 = y * 6;
            let y_1 = y_1 + 4;

            let z = x_1 * y_1.float();

            z.into_data().convert()
        }

        type Backend = Wgpu;
        type FusedBackend = Fusion<Wgpu>;

        let result_fused = func::<FusedBackend>(data_1.clone(), data_2.clone());
        let result_ref = func::<Backend>(data_1.clone(), data_2.clone());

        result_ref.assert_approx_eq(&result_fused, 3);
    }

    #[test]
    fn test_fusion_same_behavior_different_variant() {
        type Backend = Wgpu;
        type FusedBackend = Fusion<Wgpu>;

        let data_1 =
            Tensor::<FusedBackend, 2>::random_devauto([1, 32], burn_tensor::Distribution::Default)
                .into_data();
        let data_2 =
            Tensor::<Backend, 2>::random_devauto([32, 32], burn_tensor::Distribution::Default)
                .into_data();

        let result_ref = execute::<Backend>(
            data_1.clone(),
            data_2.clone(),
            ImplementationDetails::Variant2,
        );
        let result_fused_variant1 = execute::<FusedBackend>(
            data_1.clone(),
            data_2.clone(),
            ImplementationDetails::Variant1,
        );
        let result_fused_variant2 = execute::<FusedBackend>(
            data_1.clone(),
            data_2.clone(),
            ImplementationDetails::Variant2,
        );

        result_ref.assert_approx_eq(&result_fused_variant1, 3);
        result_ref.assert_approx_eq(&result_fused_variant2, 3);
    }

    #[test]
    fn test_end_condition_scalar_ops() {
        type Backend = Fusion<Wgpu>;
        let tensor1 = Tensor::<Backend, 2>::ones_devauto([32, 32]);
        let tensor2 = Tensor::<Backend, 2>::ones_devauto([32, 42]);
        let output = tensor1.exp().log();

        // This will add a scalar to the context, even if the actual operation can't be fused with
        // the preceding ones because of the shape difference.
        let _ = tensor2 + 2;

        // When we try to execute the operations, the number of bindings can be different if we are
        // not careful.
        Backend::sync(&output.device());
    }

    struct FakeAddOps;

    impl<B: FusionBackend> Ops<B> for FakeAddOps {
        fn execute(self: Box<Self>, _: &mut burn_fusion::HandleContainer<B>) {
            panic!("Should always fused during tests.")
        }
    }

    enum ImplementationDetails {
        Variant1,
        Variant2,
    }

    fn execute<B: Backend>(
        data_1: Data<f32, 2>,
        data_2: Data<f32, 2>,
        variant: ImplementationDetails,
    ) -> Data<f32, 2> {
        let tensor_1 = Tensor::<B, 2>::from_data_devauto(data_1.convert());
        let tensor_2 = Tensor::<B, 2>::from_data_devauto(data_2.convert());
        let tensor_3 = tensor_1.clone() + tensor_2;
        let tensor_4 = tensor_3.clone() - tensor_1;
        let mut tensor_5 = tensor_4.clone() + 5.0;
        match variant {
            ImplementationDetails::Variant1 => {}
            ImplementationDetails::Variant2 => {
                tensor_5 = tensor_5 + 1;
                tensor_5 = tensor_5 - 1;
            }
        }
        let tensor_6 = burn_tensor::activation::gelu(tensor_5 + tensor_3.clone());
        let mask = tensor_4.lower_equal(tensor_3);
        let tmp = tensor_6.mask_fill(mask, 0.3);

        tmp.into_data().convert()
    }
}
