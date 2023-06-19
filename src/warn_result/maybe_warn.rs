use core::ops::{Deref, DerefMut};

use MaybeWarn::*;

pub enum MaybeWarn<T, W> {
    Ok(T),
    Warn(T, W),
}

impl<T, W> MaybeWarn<T, W> {
    pub fn value(&self) -> &T {
        let (Ok(val) | Warn(val, _)) = self;
        val
    }

    pub fn value_mut(&mut self) -> &mut T {
        let (Ok(val) | Warn(val, _)) = self;
        val
    }

    pub fn discard_warnings(self) -> T {
        let (Ok(val) | Warn(val, _)) = self;
        val
    }

    pub fn as_ref(&self) -> MaybeWarn<&T, &W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
        }
    }

    pub fn as_mut(&mut self) -> MaybeWarn<&mut T, &mut W> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, warn),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> MaybeWarn<U, W> {
        match self {
            Ok(val) => Ok(f(val)),
            Warn(val, warn) => Warn(f(val), warn),
        }
    }

    pub fn map_warn<U>(self, f: impl FnOnce(W) -> U) -> MaybeWarn<T, U> {
        match self {
            Ok(val) => Ok(val),
            Warn(val, warn) => Warn(val, f(warn)),
        }
    }
}

impl<T, W> Deref for MaybeWarn<T, W> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<T, W> DerefMut for MaybeWarn<T, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}

impl<T, W> AsRef<T> for MaybeWarn<T, W> {
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<T, W> AsMut<T> for MaybeWarn<T, W> {
    fn as_mut(&mut self) -> &mut T {
        self.value_mut()
    }
}
