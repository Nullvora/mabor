use crate::dialect::{Item, Variable};
use crate::language::{CubeContext, CubeElem, CubeType, ExpandElement};
use crate::{index_assign, unexpanded, Abs, Max, Min, Remainder};

/// Type that encompasses both (unsigned or signed) integers and floats
/// Used in kernels that should work for both.
pub trait Numeric:
    Clone
    + Copy
    + CubeElem
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::DivAssign
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::cmp::PartialOrd
    + Abs
    + Max
    + Min
    + Remainder
{
    /// Create a new constant numeric.
    ///
    /// Note: since this must work for both integer and float
    /// only the less expressive of both can be created (int)
    /// If a number with decimals is needed, use Float::new.
    ///
    /// This method panics when unexpanded. For creating an element
    /// with a val, use the new method of the sub type.
    fn from_int(_val: i64) -> Self {
        unexpanded!()
    }

    /// Expand version of from_int
    fn from_int_expand(_context: &mut CubeContext, val: i64) -> <Self as CubeType>::ExpandType {
        let new_var = Variable::ConstantScalar(val as f64, Self::as_elem());
        ExpandElement::Plain(new_var)
    }

    fn from_vec(_vec: &[i64]) -> Self {
        unexpanded!()
    }

    fn from_vec_expand(context: &mut CubeContext, vec: &[i64]) -> <Self as CubeType>::ExpandType {
        let mut new_var = context.create_local(Item {
            elem: Self::as_elem(),
            vectorization: (vec.len() as u8),
        });
        for (i, element) in vec.iter().enumerate() {
            new_var = index_assign::expand(context, new_var, i.into(), (*element).into());
        }

        new_var
    }
}
