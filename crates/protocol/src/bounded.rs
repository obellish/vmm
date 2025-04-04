use std::{
	borrow::Borrow,
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Bounded<T, const MAX: usize>(pub T);

impl<T, const MAX: usize> Bounded<T, MAX> {
	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Bounded<U, MAX> {
		Bounded(f(self.0))
	}

	pub fn map_into<U>(self) -> Bounded<U, MAX>
	where
		T: Into<U>,
	{
		Bounded(self.0.into())
	}
}

impl<T, const MAX: usize> AsRef<T> for Bounded<T, MAX> {
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T, const MAX: usize> Borrow<T> for Bounded<T, MAX> {
	fn borrow(&self) -> &T {
		self
	}
}

impl<T, const MAX: usize> Deref for Bounded<T, MAX> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, const MAX: usize> DerefMut for Bounded<T, MAX> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T: Display, const MAX: usize> Display for Bounded<T, MAX> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<T, const MAX: usize> From<T> for Bounded<T, MAX> {
	fn from(value: T) -> Self {
		Self(value)
	}
}
