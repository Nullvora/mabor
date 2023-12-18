use crate::codegen::{
    Binding, Body, ComputeShader, Elem, Function, Location, Operator, Variable, Visibility,
    WorkgroupSize,
};
use crate::compute::{StaticKernel, WgpuComputeClient, WgpuHandle};
use crate::element::WgpuElement;
use crate::kernel::{elemwise_workgroup, StaticKernelSource, WORKGROUP_DEFAULT};
use std::marker::PhantomData;

/// Kernel creation input phase, see [kernel codegen](ElemWiseKernelCodegen) for more details.
pub struct InputPhase;
/// Kernel creation body phase, see [kernel codegen](ElemWiseKernelCodegen) for more details.
pub struct BodyPhase;
/// Kernel creation output phase, see [kernel codegen](ElemWiseKernelCodegen) for more details.
pub struct OutputPhase;
/// Kernel compilation phase, see [kernel codegen](ElemWiseKernelCodegen) for more details.
pub struct CompilationPhase;

/// Allows to create custom wgsl kernels based on configured inputs, body and outputs.
///
/// This type has 4 phases that must be executed in order, but no worry the type system won't allow
/// you to make mistakes.
///
///   1. [Input Phase](InputPhase)
///     This phase focuses on registering the input arrays and scalars that are going to be used by
///     the kernel.
///   2. [Body Phase](BodyPhase)
///     After the input phase is done, all the operations that happen in the body must be
///     registered.
///   3. [Output Phase](OutputPhase)
///     This step focuses on registering all output arrays or inputs that the kernel needs to write to.
///   4. [Compilation Phase](CompilationPhase)
///     Now that all other phases are completed, we can actually compile the kernel.
pub struct ElemWiseKernelCodegen<Phase = InputPhase> {
    operations: Vec<Operator>,
    input_bindings: Vec<Binding>,
    output_bindings: Vec<Binding>,
    named_bindings: Vec<(String, Binding)>,
    functions: Vec<Function>,
    _phase: PhantomData<Phase>,
}

pub enum Input {
    Array {
        elem: Elem,
        visibility: Visibility,
        strategy: ReadingStrategy,
    },
    Scalar {
        elem: Elem,
        size: usize,
    },
}

pub enum ReadingStrategy {
    IntoContiguous,
    Plain,
}

pub enum Output {
    Array { elem: Elem, local: u16 },
    Input { elem: Elem, input: u16, local: u16 },
}

impl ElemWiseKernelCodegen<InputPhase> {
    /// Create a new fusion kernel on the given device.
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            input_bindings: Vec::new(),
            output_bindings: Vec::new(),
            named_bindings: Vec::new(),
            functions: Vec::new(),
            _phase: PhantomData,
        }
    }

    /// Register the inputs used by the kernel.
    pub fn inputs(mut self, inputs: &[Input]) -> ElemWiseKernelCodegen<BodyPhase> {
        let mut index: u16 = 0;

        let first_output_index = inputs
            .iter()
            .filter(|input| match input {
                Input::Array {
                    elem: _,
                    visibility: _,
                    strategy: _,
                } => true,
                Input::Scalar { elem: _, size: _ } => false,
            })
            .count();

        for input in inputs {
            match input {
                Input::Array {
                    elem,
                    visibility,
                    strategy,
                } => {
                    self.input_bindings.push(Binding {
                        elem: bool_elem(*elem),
                        visibility: *visibility,
                        location: Location::Storage,
                        size: None,
                    });

                    match strategy {
                        ReadingStrategy::IntoContiguous => {
                            self.operations.push(Operator::ReadGlobalIntoContiguous {
                                variable: Variable::Input(index, *elem),
                                position: index as usize,
                                position_out: first_output_index, // First output
                            });
                        }
                        ReadingStrategy::Plain => {
                            self.operations.push(Operator::ReadGlobal {
                                variable: Variable::Input(index, *elem),
                            });
                        }
                    }

                    index += 1;
                }
                Input::Scalar { elem, size } => {
                    let elem = bool_elem(*elem);

                    self.named_bindings.push((
                        format!("scalars_{}", elem),
                        Binding {
                            elem,
                            visibility: Visibility::Read,
                            location: Location::Storage,
                            size: Some(*size),
                        },
                    ));
                }
            }
        }

        ElemWiseKernelCodegen {
            operations: self.operations,
            input_bindings: self.input_bindings,
            output_bindings: self.output_bindings,
            named_bindings: self.named_bindings,
            functions: self.functions,
            _phase: PhantomData,
        }
    }
}

impl ElemWiseKernelCodegen<BodyPhase> {
    /// Register the [operators](Operator) that the kernel must execute in the order provided.
    pub fn body(mut self, operators: &[Operator]) -> ElemWiseKernelCodegen<OutputPhase> {
        let mut register_function = |function: Function| {
            if !self.functions.contains(&function) {
                self.functions.push(function);
            }
        };

        // Since not all operators are native to WGSL, we need to add the custom ones.
        for ops in operators.iter() {
            match ops {
                Operator::Powf {
                    lhs: _,
                    rhs: _,
                    out: _,
                } => {
                    register_function(Function::Powf(Elem::F32));
                }
                Operator::Erf { input: _, out: _ } => {
                    register_function(Function::Erf(Elem::F32));
                }
                _ => {}
            }
            self.operations.push(ops.clone());
        }

        ElemWiseKernelCodegen {
            operations: self.operations,
            input_bindings: self.input_bindings,
            output_bindings: self.output_bindings,
            named_bindings: self.named_bindings,
            functions: self.functions,
            _phase: PhantomData,
        }
    }
}

impl ElemWiseKernelCodegen<OutputPhase> {
    /// Register the outputs with their local variable index.
    ///
    /// Note that the index corresponds to the registered [operator](Operator) number at the
    /// [body phase](BodyPhase).
    /// So the 4th operator registered creates the local variable 3 (N-1, since the 1th index is 0).
    pub fn outputs(mut self, outputs: &[Output]) -> ElemWiseKernelCodegen<CompilationPhase> {
        let mut index = 0;

        for array in outputs {
            match array {
                Output::Array { elem, local } => {
                    let elem_adapted = bool_elem(*elem);

                    self.output_bindings.push(Binding {
                        elem: elem_adapted,
                        visibility: Visibility::ReadWrite,
                        location: Location::Storage,
                        size: None,
                    });
                    self.operations.push(Operator::AssignGlobal {
                        input: Variable::Local(*local, *elem),
                        out: Variable::Output(index, elem_adapted),
                    });
                    index += 1;
                }
                Output::Input { elem, input, local } => {
                    self.operations.push(Operator::AssignGlobal {
                        input: Variable::Local(*local, *elem),
                        out: Variable::Input(*input, bool_elem(*elem)),
                    });
                }
            }
        }

        ElemWiseKernelCodegen {
            operations: self.operations,
            input_bindings: self.input_bindings,
            output_bindings: self.output_bindings,
            named_bindings: self.named_bindings,
            functions: self.functions,
            _phase: PhantomData,
        }
    }
}

impl ElemWiseKernelCodegen<CompilationPhase> {
    /// Compile the kernel into a [compute shader](ComputeShader).
    pub fn compile(self) -> ComputeShader {
        let inputs = self.input_bindings;
        let outputs = self.output_bindings;
        let mut named = Vec::with_capacity(2);

        named.push((
            "info".to_string(),
            Binding {
                elem: Elem::U32,
                visibility: Visibility::Read,
                location: Location::Storage,
                size: None, // We avoid putting the length here since it will force a new kernel
                            // for each tensor rank.
            },
        ));

        for (name, binding) in self.named_bindings.into_iter() {
            named.push((name, binding));
        }

        ComputeShader {
            inputs,
            outputs,
            named,
            workgroup_size: WorkgroupSize::default(),
            body: Body::new(self.operations),
            num_workgroups: true,
            global_invocation_id: true,
            functions: self.functions,
        }
    }
}

#[derive(new)]
pub struct StaticHandle<'a> {
    handle: &'a WgpuHandle,
    strides: &'a [usize],
    shape: &'a [usize],
}

pub enum GridLaunch {
    Input { pos: usize },
    Output { pos: usize },
}

/// Execute a static kernel.
///
///
/// The limitation from this method is that you can't launch a kernel with multiple types of
/// scalar.
pub fn execute_static<K, E: WgpuElement>(
    inputs: &[StaticHandle],
    outputs: &[StaticHandle],
    scalar_elems: Option<&[E]>,
    launch: GridLaunch,
    client: WgpuComputeClient,
) where
    K: StaticKernelSource + 'static,
{
    let mut info = Vec::new();
    let mut handles = Vec::with_capacity(inputs.len() + outputs.len() + 2);

    // Inner function to fill the info buffer.
    let mut register_info_tensor = |strides: &[usize], shape: &[usize]| {
        if info.is_empty() {
            info.push(strides.len() as u32);
        }

        for s in strides.iter() {
            info.push(*s as u32);
        }
        for s in shape.iter() {
            info.push(*s as u32);
        }
    };

    let mut num_elems_output = 0;

    // We start by registering the inputs.
    for (i, input) in inputs.iter().enumerate() {
        if let GridLaunch::Input { pos } = &launch {
            if i == *pos {
                num_elems_output = calculate_num_elems_dyn_rank(input.shape);
            }
        };
        register_info_tensor(input.strides, input.shape);
        handles.push(input.handle);
    }

    // Then we follow with the outputs.
    for (i, output) in outputs.iter().enumerate() {
        if let GridLaunch::Output { pos } = &launch {
            if i == *pos {
                num_elems_output = calculate_num_elems_dyn_rank(output.shape);
            }
        };
        register_info_tensor(output.strides, output.shape);
        handles.push(output.handle);
    }

    let info = &client.create(bytemuck::cast_slice(&info));
    handles.push(info);

    // Finally we finish with the named bindings.
    let mut scalars = None;
    if let Some(values) = &scalar_elems {
        scalars = Some(client.create(bytemuck::cast_slice(values)));
    }

    if let Some(scalars) = scalars.as_ref() {
        handles.push(scalars);
    }

    let workgroup = elemwise_workgroup(num_elems_output, WORKGROUP_DEFAULT);
    let kernel = Box::new(StaticKernel::<K>::new(workgroup));

    client.execute(kernel, &handles);
}

pub(crate) fn calculate_num_elems_dyn_rank(shape: &[usize]) -> usize {
    let mut num_elems = 1;
    for i in shape.iter() {
        num_elems *= i;
    }
    num_elems
}

fn bool_elem(elem: Elem) -> Elem {
    match elem {
        // I32 are used for bool tensors
        Elem::Bool => Elem::I32,
        _ => elem,
    }
}
