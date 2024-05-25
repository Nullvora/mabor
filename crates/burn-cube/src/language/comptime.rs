use crate::{unexpanded, CubeArg, CubeContext, CubeType, Runtime, RuntimeArg};

#[derive(Clone, Copy)]
/// Encapsulates a value to signify it must be used at compilation time rather than in the kernel
///
/// Use `Comptime<Option<T>>` to have an alternate runtime behaviour if the compilation time value is not present
pub struct Comptime<T> {
    inner: T,
}

impl<T> Comptime<T> {
    /// Create a new Comptime. Useful when hardcoding values in
    /// Cube kernels. For instance:
    /// if Comptime::new(false) {...} never generates the inner code block
    pub fn new(_inner: T) -> Self {
        unexpanded!()
    }

    /// Get the inner value of a Comptime. For instance:
    /// let c = Comptime::new(false);
    /// if Comptime::get(c) {...}
    pub fn get(_comptime: Self) -> T {
        unexpanded!()
    }
}

impl<T: CubeType + Into<T::ExpandType>> Comptime<Option<T>> {
    /// Map a Comptime optional to a Comptime boolean that tell
    /// whether the optional contained a value
    pub fn is_some(comptime: Self) -> Comptime<bool> {
        Comptime::new(comptime.inner.is_some())
    }

    /// Return the inner value of the Comptime if it exists,
    /// otherwise tell how to compute it at runtime
    pub fn unwrap_or_else<F>(_comptime: Self, mut _alt: F) -> T
    where
        F: FnMut() -> T,
    {
        unexpanded!()
    }

    /// Expanded version of unwrap_or_else
    pub fn unwrap_or_else_expand<F>(
        context: &mut CubeContext,
        t: Option<T>,
        mut alt: F,
    ) -> <T as CubeType>::ExpandType
    where
        F: FnMut(&mut CubeContext) -> T::ExpandType,
    {
        match t {
            Some(t) => t.into(),
            None => alt(context),
        }
    }
}

impl<T: Clone> CubeType for Comptime<T> {
    type ExpandType = T;
}

impl<T: CubeArg> CubeArg for Comptime<T> {
    type ArgType<'a, R: Runtime> = T::ArgType<'a, R>;
}

impl<T: CubeArg> CubeArg for Option<T> {
    type ArgType<'a, R: Runtime> = Option<T::ArgType<'a, R>>;
}

impl<T, R> RuntimeArg<R> for Option<T>
where
    T: RuntimeArg<R>,
    R: Runtime,
{
    fn register(&self, settings: &mut crate::BindingSettings<R>) {
        if let Some(arg) = self {
            arg.register(settings)
        }
    }
}
