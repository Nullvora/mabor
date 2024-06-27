use burn_cube::{
    frontend::{CubePrimitive, Float, Int, UInt, BF16, F16, F32, I32},
    CubeElement,
};

/// The base element trait for the jit backend.
pub trait JitElement: burn_tensor::Element + CubeElement {
    /// Cube primitive representing the jit element.
    type Primitive: CubePrimitive;
}

/// The float element type for the jit backend.
pub trait FloatElement: JitElement {
    /// Cube primitive representing the jit element.
    type FloatPrimitive: Float;
}

/// The int element type for the jit backend.
pub trait IntElement: JitElement {
    /// Cube primitive representing the jit element.
    type IntPrimitive: Int;
}

impl JitElement for u32 {
    type Primitive = UInt;
}

impl JitElement for i32 {
    type Primitive = I32;
}

impl JitElement for f32 {
    type Primitive = F32;
}

impl JitElement for half::f16 {
    type Primitive = F16;
}

impl JitElement for half::bf16 {
    type Primitive = BF16;
}
impl FloatElement for f32 {
    type FloatPrimitive = F32;
}
impl FloatElement for half::bf16 {
    type FloatPrimitive = BF16;
}
impl FloatElement for half::f16 {
    type FloatPrimitive = F16;
}
impl IntElement for i32 {
    type IntPrimitive = I32;
}
