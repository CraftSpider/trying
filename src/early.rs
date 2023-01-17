use core::convert::Infallible;
#[cfg(feature = "yeet")]
use core::ops::Yeet;
use core::ops::{ControlFlow, FromResidual, Try};

use Early::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Early<D, T> {
    Done(D),
    Todo(T),
}

impl<D, T> Early<D, T> {
    pub fn unwrap(self) -> D {
        if let Done(val) = self {
            val
        } else {
            panic!("Called `unwrap` on Early::Todo")
        }
    }

    pub fn unwrap_todo(self) -> T {
        if let Todo(val) = self {
            val
        } else {
            panic!("Called `unwrap` on Early::Done")
        }
    }

    pub fn as_ref(&self) -> Early<&D, &T> {
        match self {
            Done(val) => Done(val),
            Todo(val) => Todo(val),
        }
    }

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
