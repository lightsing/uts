use std::{
    fmt,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

mod fixed_bytes;
#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Wrapped<T>(pub T);

impl<T> Wrapped<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Debug> Debug for Wrapped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display> Display for Wrapped<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Wrapped<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Wrapped<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Wrapped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Wrapped<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
