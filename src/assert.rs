use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};
use core::panic::Location;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Formatter;
use std::process::{ExitCode, Termination};

pub struct AssertResidual(&'static Location<'static>, Cow<'static, str>);

enum AssertInner {
    Success,
    Failure(&'static Location<'static>, Cow<'static, str>),
}

/// A struct representing a logical assertion made at runtime. This assertion must be consumed in
/// some way, or it will panic/abort on drop.
///
/// The most common things to do with an assertion are propagate it upwards using `?` or
/// convert it into a panic with [`to_panic`](Self::to_panic). If you receive an assertion and
/// really need to get rid of it, call [`defuse`](Self::defuse) to consume it harmlessly.
///
/// # Examples
///
/// The most basic usage of assert is as follows:
///
/// ```
/// # use trying::assert::Assert;
/// fn frobnicate(a: i32) -> i32 {
///     a * 2
/// }
///
/// #[test]
/// fn ret_assert() -> Assert {
///     let a = frobnicate(1);
///
///     Assert::eq(&a, &2).msg("Failed to frobnicate a")?;
///     Assert::ne(&frobnicate(1), &frobnicate(2))?;
///
///     Assert::success()
/// }
/// ```
///
/// However, this seem a bit weird - we either need to treat the last assertion differently, or
/// return a pointless successful assert. So, `Assert` can be used with [`Result`] seamlessly:
///
/// ```
/// # use trying::assert::Assert;
/// # fn frobnicate(a: i32) -> i32 {
/// #     a * 2
/// # }
/// #[test]
/// fn ret_result() -> Result<(), Assert> {
///     let a = frobnicate(1);
///
///     Assert::eq(&a, &2).msg("Failed to frobnicate a")?;
///     Assert::ne(&frobnicate(1), &frobnicate(2))?;
///
///     Ok(())
/// }
/// ```
///
/// Now we have something that feels a bit rustier. The final useful trait of `Assert` is if
/// you want to use some other method that returns a result with an error type.
///
/// ```
/// # use trying::assert::Assert;
/// #[test]
/// fn ret_result() -> Result<(), Assert> {
///     let a = 2u32;
///     let b = u32::try_from(2u64)?;
///
///     Assert::eq(&a, &b).msg("2 != 2?")?;
///
///     Ok(())
/// }
/// ```
///
/// As we see here, `Assert` implements `From` for any type that implements [`Error`](std::error::Error).
/// That way, tests can call normal methods with more fitting error types, with no extra cost to the
/// user.
#[must_use = "use `?` to propagate the assertion or `to_panic` to panic on failure"]
pub struct Assert(AssertInner);

impl Assert {
    fn inner_defuse(mut self) -> AssertInner {
        std::mem::replace(&mut self.0, AssertInner::Success)
    }

    /// Create a successful assertion
    pub fn success() -> Assert {
        Assert(AssertInner::Success)
    }

    /// Create a failed assertion
    #[track_caller]
    pub fn failure() -> Assert {
        Assert(AssertInner::Failure(
            Location::caller(),
            Cow::from("Assertion failed"),
        ))
    }

    /// Assert that the result of a boolean value is true
    #[track_caller]
    pub fn is_true(a: bool) -> Assert {
        if a {
            Assert::success()
        } else {
            Assert::failure().msg("Expected `true`, got `false`")
        }
    }

    /// Assert that the result of a boolean value is false
    #[track_caller]
    pub fn is_false(a: bool) -> Assert {
        if !a {
            Assert::success()
        } else {
            Assert::failure().msg("Expected `false`, got `true`")
        }
    }

    /// Assert that two values are equal
    #[track_caller]
    pub fn eq<T, U>(a: &T, b: &U) -> Assert
    where
        T: Debug + PartialEq<U>,
        U: Debug,
    {
        if a == b {
            Assert::success()
        } else {
            Assert::failure().msg(format!("Expected `{:?}` to equal `{:?}`", a, b))
        }
    }

    /// Assert that two values are not equal
    #[track_caller]
    pub fn ne<T, U>(a: &T, b: &U) -> Assert
    where
        T: Debug + PartialEq<U>,
        U: Debug,
    {
        if a != b {
            Assert::success()
        } else {
            Assert::failure().msg(format!("Expected `{:?}` to not equal `{:?}`", a, b))
        }
    }

    /// Attach a custom message to an assertion. This message is discarded if the assertion was
    /// successful.
    pub fn msg<S>(self, msg: S) -> Assert
    where
        Cow<'static, str>: From<S>,
    {
        Assert(match self.inner_defuse() {
            AssertInner::Failure(loc, _) => AssertInner::Failure(loc, msg.into()),
            assert => assert,
        })
    }

    /// Attach a custom message to an assertion, only calling the provided function to generate
    /// the message if the assertion failed.
    pub fn with_msg(self, f: impl FnOnce() -> String) -> Assert {
        Assert(match self.inner_defuse() {
            AssertInner::Failure(loc, _) => AssertInner::Failure(loc, Cow::from(f())),
            assert => assert,
        })
    }

    /// Convert this assertion to a panic if it failed, or do nothing on a success.
    pub fn to_panic(self) {
        if let AssertInner::Failure(loc, msg) = self.inner_defuse() {
            panic!("{} at {}", msg, loc)
        }
    }

    /// Consume this assertion harmlessly, doing nothing. This is probably not what you want,
    /// unless you really need to ignore a failed assertion for some reason.
    pub fn defuse(self) {
        self.inner_defuse();
    }

    /// Check whether this assertion failed
    pub fn is_failure(&self) -> bool {
        matches!(self.0, AssertInner::Failure(..))
    }

    /// Check whether this assertion succeeded
    pub fn is_success(&self) -> bool {
        matches!(self.0, AssertInner::Success)
    }
}

impl Drop for Assert {
    fn drop(&mut self) {
        if let AssertInner::Failure(_, _) = self.0 {
            panic!("Failed assertion dropped. (Did you forget a `?` or `to_panic`?)\n{:?}", self);
        }
    }
}

impl Debug for Assert {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            AssertInner::Failure(loc, msg) => {
                write!(f, "Assertion Failed: {} at {}", msg, loc)
            }
            AssertInner::Success => {
                write!(f, "Assertion Successful")
            }
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
        match self.inner_defuse() {
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

impl<T> FromResidual<AssertResidual> for Result<T, Assert> {
    fn from_residual(residual: AssertResidual) -> Self {
        Err(Assert::from_residual(residual))
    }
}

impl Termination for Assert {
    fn report(self) -> ExitCode {
        if let AssertInner::Failure(_, _) = self.0 {
            println!("{:?}", self);
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

impl<T> From<Assert> for Result<T, Assert> {
    fn from(a: Assert) -> Self {
        Err(a)
    }
}

impl<E> From<E> for Assert
where
    E: Error,
{
    fn from(err: E) -> Self {
        Assert::failure().msg(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_true() -> Assert {
        Assert::is_true(true)
    }

    #[test]
    #[should_panic]
    fn test_assert_true_failure() {
        Assert::is_true(false).to_panic();
    }

    #[test]
    fn test_assert_false() -> Assert {
        Assert::is_false(false)
    }

    #[test]
    #[should_panic]
    fn test_assert_false_failure() {
        Assert::is_false(true).to_panic()
    }

    #[test]
    fn test_assert_eq() -> Assert {
        Assert::eq(&1u32, &1u32)
    }

    #[test]
    #[should_panic]
    fn test_assert_eq_failure() {
        Assert::eq(&"hello", &"world").to_panic()
    }

    #[test]
    fn test_assert_ne() -> Assert {
        Assert::ne(&1u32, &2u32)
    }

    #[test]
    #[should_panic]
    fn test_assert_ne_failure() {
        Assert::ne(&1.0, &1.0).to_panic()
    }

    #[test]
    #[should_panic = "[Custom Message]"]
    fn test_assert_msg() {
        Assert::failure().msg("[Custom Message]").to_panic()
    }

    #[test]
    #[should_panic = "[Custom Message]"]
    fn test_assert_with_msg() {
        Assert::failure().with_msg(|| String::from("[Custom Message]")).to_panic()
    }
}
