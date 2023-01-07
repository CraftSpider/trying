use core::convert::Infallible;
use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};
use core::result::Result as CoreResult;
use std::io;
use std::io::Write;
use std::process::{ExitCode, Termination};

#[cfg(feature = "yeet")]
use core::ops::Yeet;

mod maybe_warn;

use self::Result::*;
pub use maybe_warn::MaybeWarn;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Result<T, E, W = E> {
    Ok(T),
    Warn(T, W),
    Err(E),
}

impl<T, E, W> Result<T, E, W> {
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }

    #[inline]
    pub fn is_warn(&self) -> bool {
        matches!(self, Warn(_, _))
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, Err(_))
    }

    #[inline]
    pub fn as_ref(&self) -> Result<&T, &E, &W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Result<&mut T, &mut E, &mut W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn map<U>(self, f: impl FnOnce(MaybeWarn<T, W>) -> MaybeWarn<U, W>) -> Result<U, E, W> {
        f(self?).into()
    }

    #[inline]
    pub fn map_val<U>(self, f: impl FnOnce(T) -> U) -> Result<U, E, W> {
        match self {
            Ok(val) => Ok(f(val)),
            Warn(val, warn) => Warn(f(val), warn),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn map_warn<U>(self, f: impl FnOnce(W) -> U) -> Result<T, E, U> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, f(warn)),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn map_err<U>(self, f: impl FnOnce(E) -> U) -> Result<T, U, W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
            Err(err) => Err(f(err)),
        }
    }

    #[inline]
    pub fn and_then<U>(
        self,
        f: impl FnOnce(MaybeWarn<T, W>) -> Result<U, E, W>,
    ) -> Result<U, E, W> {
        f(self?)
    }

    #[inline]
    pub fn or_else<U>(self, f: impl FnOnce(E) -> Result<T, U, W>) -> Result<T, U, W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
            Err(err) => f(err),
        }
    }
}

impl<T, E, W> Result<&T, E, W> {
    #[inline]
    pub fn copied(self) -> Result<T, E, W>
    where
        T: Copy,
    {
        self.map_val(|val| *val)
    }

    #[inline]
    pub fn cloned(self) -> Result<T, E, W>
    where
        T: Clone,
    {
        self.map_val(T::clone)
    }
}

impl<T, E, W> Result<&mut T, E, W> {
    #[inline]
    pub fn copied(self) -> Result<T, E, W>
    where
        T: Copy,
    {
        self.map_val(|val| *val)
    }

    #[inline]
    pub fn cloned(self) -> Result<T, E, W>
    where
        T: Clone,
    {
        self.map_val(|val| val.clone())
    }
}

impl<T, E, W> Result<Option<T>, E, W> {
    pub fn transpose_lossy(self) -> Option<Result<T, E, W>> {
        match self {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) | Warn(None, _) => None,

            Warn(Some(val), warn) => Some(Warn(val, warn)),

            Err(err) => Some(Err(err)),
        }
    }
}

impl<T, E, W> Result<Result<T, E, W>, E, W> {
    #[inline]
    pub fn flatten_inner(self) -> Result<T, E, W> {
        match self {
            Ok(Ok(val)) => Ok(val),

            Ok(Warn(val, warn))
            | Warn(Warn(val, warn), _)
            | Warn(Ok(val), warn)=> Warn(val, warn),

            Ok(Err(err)) | Warn(Err(err), _) | Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn flatten_outer(self) -> Result<T, E, W> {
        match self {
            Ok(Ok(val)) => Ok(val),

            Ok(Warn(val, warn))
            | Warn(Ok(val) | Warn(val, _), warn) => Warn(val, warn),

            Ok(Err(err))
            | Warn(Err(err), _)
            | Err(err) => Err(err),
        }
    }
}

impl<T, E, W> From<MaybeWarn<T, W>> for Result<T, E, W> {
    #[inline]
    fn from(m: MaybeWarn<T, W>) -> Self {
        match m {
            MaybeWarn::Ok(val) => Ok(val),
            MaybeWarn::Warn(val, warn) => Warn(val, warn),
        }
    }
}

impl<T, E, W> From<CoreResult<T, E>> for Result<T, E, W> {
    #[inline]
    fn from(r: CoreResult<T, E>) -> Self {
        match r {
            CoreResult::Ok(val) => Ok(val),
            CoreResult::Err(err) => Err(err),
        }
    }
}

impl<T, E, W> Try for Result<T, E, W> {
    type Output = MaybeWarn<T, W>;
    type Residual = Result<Infallible, E, Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        output.into()
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(val) => ControlFlow::Continue(MaybeWarn::Ok(val)),
            Warn(val, warn) => ControlFlow::Continue(MaybeWarn::Warn(val, warn)),
            Err(err) => ControlFlow::Break(Err(err)),
        }
    }
}

impl<T, E, W> FromResidual for Result<T, E, W> {
    #[inline]
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Err(err) => Err(err),
            Ok(val) | Warn(val, _) => match val {},
        }
    }
}

impl<T, E, W> FromResidual<CoreResult<Infallible, E>> for Result<T, E, W> {
    #[inline]
    fn from_residual(residual: CoreResult<Infallible, E>) -> Self {
        match residual {
            CoreResult::Err(err) => Err(err),
            CoreResult::Ok(val) => match val {},
        }
    }
}

#[cfg(feature = "yeet")]
impl<T, E, W> FromResidual<Yeet<E>> for Result<T, E, W> {
    fn from_residual(residual: Yeet<E>) -> Self {
        Err(residual.0)
    }
}

impl<T: Termination, E: Debug, W: Debug> Termination for Result<T, E, W> {
    fn report(self) -> ExitCode {
        match self {
            Ok(val) => val.report(),
            Warn(val, warn) => {
                drop(writeln!(io::stderr(), "Warning: {warn:?}"));
                val.report()
            }
            Err(err) => {
                drop(writeln!(io::stderr(), "Error: {err:?}"));
                ExitCode::FAILURE
            }
        }
    }
}

impl<T, T1, E, W, W1> FromIterator<Result<T1, E, W1>> for Result<T, E, W>
where
    T: FromIterator<T1>,
    W: Default + Extend<W1>,
{
    fn from_iter<Iter: IntoIterator<Item = Result<T1, E, W1>>>(iter: Iter) -> Self {
        let mut state = Ok(());
        let mut warns = W::default();

        let out = iter
            .into_iter()
            .scan(
                (&mut state, &mut warns),
                |(state, warns), item| match item {
                    Ok(val) => Some(val),
                    Warn(val, warn) => {
                        **state = Warn((), ());
                        warns.extend([warn]);
                        Some(val)
                    }
                    Err(err) => {
                        **state = Err(err);
                        None
                    }
                },
            )
            .collect();

        match state {
            Ok(_) => Ok(out),
            Warn(_, _) => Warn(out, warns),
            Err(err) => Err(err),
        }
    }
}

impl<T, T1, E, W> FromIterator<CoreResult<T1, E>> for Result<T, E, W>
where
    T: FromIterator<T1>,
{
    fn from_iter<Iter: IntoIterator<Item = CoreResult<T1, E>>>(iter: Iter) -> Self {
        let mut state = CoreResult::Ok(());

        let out = iter
            .into_iter()
            .scan(&mut state, |state, item| match item {
                CoreResult::Ok(val) => Some(val),
                CoreResult::Err(err) => {
                    **state = CoreResult::Err(err);
                    None
                }
            })
            .collect();

        match state {
            CoreResult::Ok(_) => Ok(out),
            CoreResult::Err(err) => Err(err),
        }
    }
}
