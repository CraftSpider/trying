//! Type for early-return values - when you want to quickly toss a result up the stack if one is
//! generated early on.

use core::convert::Infallible;
#[cfg(feature = "yeet")]
use core::ops::Yeet;
use core::ops::{ControlFlow, FromResidual, Try};

use Early::*;

/// An early-return value. A type for when a call may return a final result or want to continue
/// execution.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Early<D, T> {
    /// The value to return if computation is done
    Done(D),
    /// The value to return if control-flow should continue
    Todo(T),
}

impl<D, T> Early<D, T> {
    /// Convert this `Early` into its `Done` value
    ///
    /// # Panics
    ///
    /// If the `Early` is not `Done`
    pub fn unwrap(self) -> D {
        if let Done(val) = self {
            val
        } else {
            panic!("Called `unwrap` on Early::Todo")
        }
    }

    /// Convert this `Early` into its `Todo` value
    ///
    /// # Panics
    ///
    /// If the `Early` is not `Todo`
    pub fn unwrap_todo(self) -> T {
        if let Todo(val) = self {
            val
        } else {
            panic!("Called `unwrap` on Early::Done")
        }
    }

    /// Get a new `Early` that holds references to the values in this `Early`
    pub fn as_ref(&self) -> Early<&D, &T> {
        match self {
            Done(val) => Done(val),
            Todo(val) => Todo(val),
        }
    }

    /// Get a new `Early` that holds mutable references to the values in this `Early`
    pub fn as_mut(&mut self) -> Early<&mut D, &mut T> {
        match self {
            Done(val) => Done(val),
            Todo(val) => Todo(val),
        }
    }

    /// If `Done`, return `Early::Done(D)`. If `Todo`, return `f(T)`
    pub fn and_then<U, F>(self, f: F) -> Early<D, U>
    where
        F: FnOnce(T) -> Early<D, U>,
    {
        match self {
            Done(val) => Done(val),
            Todo(val) => f(val),
        }
    }
}

impl<D, T> Try for Early<D, T> {
    type Output = T;
    type Residual = Early<D, Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Todo(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Done(d) => ControlFlow::Break(Done(d)),
            Todo(u) => ControlFlow::Continue(u),
        }
    }
}

impl<D, T> FromResidual for Early<D, T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Done(d) => Done(d),
            Todo(t) => match t {},
        }
    }
}

#[cfg(feature = "yeet")]
impl<D, T> FromResidual<Yeet<D>> for Early<D, T> {
    fn from_residual(residual: Yeet<D>) -> Self {
        Done(residual.0)
    }
}
