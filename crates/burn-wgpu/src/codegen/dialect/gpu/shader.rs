use crate::kernel::WORKGROUP_DEFAULT;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::Scope;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Location {
    Storage,
    Workgroup,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Visibility {
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Serialize, Deserialize)]
pub enum Elem {
    Float,
    Int,
    UInt,
    Bool,
}

impl From<Elem> for Item {
    fn from(val: Elem) -> Self {
        Item::Scalar(val)
    }
}

impl Display for Elem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float => f.write_str("float"),
            Self::Int => f.write_str("int"),
            Self::UInt => f.write_str("uint"),
            Self::Bool => f.write_str("bool"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum Item {
    Vec4(Elem),
    Vec3(Elem),
    Vec2(Elem),
    Scalar(Elem),
}

impl Item {
    pub fn elem(&self) -> Elem {
        match self {
            Self::Vec4(elem) => *elem,
            Self::Vec3(elem) => *elem,
            Self::Vec2(elem) => *elem,
            Self::Scalar(elem) => *elem,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub location: Location,
    pub visibility: Visibility,
    pub item: Item,
    pub size: Option<usize>,
}

#[derive(new, Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub struct WorkgroupSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Default for WorkgroupSize {
    fn default() -> Self {
        Self {
            x: WORKGROUP_DEFAULT as u32,
            y: WORKGROUP_DEFAULT as u32,
            z: 1,
        }
    }
}

impl WorkgroupSize {
    pub fn num_elements(&self) -> u32 {
        self.x * self.y * self.z
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeShader {
    pub inputs: Vec<Binding>,
    pub outputs: Vec<Binding>,
    pub named: Vec<(String, Binding)>,
    pub workgroup_size: WorkgroupSize,
    pub body: Scope,
}
