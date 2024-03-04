use crate::codegen::dialect::gpu::{macros::gpu, Scope, Variable, Vectorization};
use serde::{Deserialize, Serialize};

/// Write to a global array.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WriteGlobal {
    pub input: Variable,
    pub global: Variable,
}

impl WriteGlobal {
    pub fn expand(self, scope: &mut Scope) {
        let output = self.global;
        let input = self.input;
        let position = Variable::Id;

        gpu!(scope, output[position] = input);
    }

    pub(crate) fn vectorize(&self, vectorization: Vectorization) -> Self {
        Self {
            input: self.input.vectorize(vectorization),
            global: self.global.vectorize(vectorization),
        }
    }
}
