use core::convert::Infallible;
#[cfg(feature = "yeet")]
use core::ops::Yeet;
use core::ops::{ControlFlow, FromResidual, Try};

use Early::*;

pub enum Early<T, U> {
    Done(T),
    Todo(U),
}

impl<T, U> Try for Early<T, U> {
    type Output = U;
    type Residual = Early<T, Infallible>;

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

impl<T, U> FromResidual for Early<T, U> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Done(d) => Done(d),
            Todo(t) => match t {},
        }
    }
}

#[cfg(feature = "yeet")]
impl<T, U> FromResidual<Yeet<T>> for Early<T, U> {
    fn from_residual(residual: Yeet<T>) -> Self {
        Done(residual.0)
    }
}
