use std::borrow::Cow;
use std::process::{ExitCode, Termination};
use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};
use core::panic::Location;

pub struct AssertResidual(&'static Location<'static>, Cow<'static, str>);

enum AssertInner {
    Success,
    Failure(&'static Location<'static>, Cow<'static, str>),
}

#[must_use = "use `?` to propagate the assertion"]
pub struct Assert(AssertInner);

impl Assert {
    pub fn success() -> Assert {
        Assert(AssertInner::Success)
    }

    #[track_caller]
    pub fn failure<S>(msg: S) -> Assert
    where
        Cow<'static, str>: From<S>,
    {
        Assert(AssertInner::Failure(Location::caller(), msg.into()))
    }

    #[track_caller]
    pub fn is_true(a: bool) -> Assert {
        if a {
            Assert::success()
        } else {
            Assert::failure("Expected `true`, got `false`")
        }
    }

    #[track_caller]
    pub fn is_false(a: bool) -> Assert {
        if !a {
            Assert::success()
        } else {
            Assert::failure("Expected `false`, got `true`")
        }
    }

    #[track_caller]
    pub fn eq<T, U>(a: T, b: U) -> Assert
    where
        T: Debug + PartialEq<U>,
        U: Debug,
    {
        if a == b {
            Assert::success()
        } else {
            Assert::failure(format!("Expected `{:?}` to equal `{:?}`", a, b))
        }
    }

    #[track_caller]
    pub fn ne<T, U>(a: T, b: U) -> Assert
    where
        T: Debug + PartialEq<U>,
        U: Debug,
    {
        if a != b {
            Assert::success()
        } else {
            Assert::failure(format!("Expected `{:?}` to not equal `{:?}`", a, b))
        }
    }

    pub fn msg<S>(self, msg: S) -> Assert
    where
        Cow<'static, str>: From<S>,
    {
        Assert(match self.0 {
            AssertInner::Failure(loc, _) => AssertInner::Failure(loc, msg.into()),
            assert => assert,
        })
    }

    pub fn with_msg(self, f: impl FnOnce() -> String) -> Assert {
        Assert(match self.0 {
            AssertInner::Failure(loc, _) => AssertInner::Failure(loc, Cow::from(f())),
            assert => assert,
        })
    }

    pub fn to_panic(self) {
        if let AssertInner::Failure(loc, msg) = self.0 {
            panic!("{} at {}", msg, loc)
        }
    }
}

impl Try for Assert {
    type Output = ();
    type Residual = AssertResidual;

    fn from_output(_: Self::Output) -> Self {
        Assert::success()
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            AssertInner::Failure(loc, msg) => ControlFlow::Break(AssertResidual(loc, msg)),
            AssertInner::Success => ControlFlow::Continue(()),
        }
    }
}

impl FromResidual for Assert {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        Assert(AssertInner::Failure(residual.0, residual.1))
    }
}

impl Termination for Assert {
    fn report(self) -> ExitCode {
        if let AssertInner::Failure(loc, msg) = self.0 {
            println!("{} at {}", msg, loc);
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

#[test]
fn foo() -> Assert {
    Assert::is_true(true)?;
    Assert::success()
}
