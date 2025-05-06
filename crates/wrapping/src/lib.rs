#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
};

#[repr(transparent)]
pub struct WrappingSlice<'a, T>(&'a [T]);

impl<T> Clone for WrappingSlice<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for WrappingSlice<'_, T> {}

impl<T: Debug> Debug for WrappingSlice<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'a, T, I> From<I> for WrappingSlice<'a, T>
where
	I: Into<&'a [T]>,
{
	fn from(value: I) -> Self {
		Self(value.into())
	}
}

impl<T> Index<usize> for WrappingSlice<'_, T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index % self.0.len()]
	}
}

#[repr(transparent)]
pub struct WrappingSliceMut<'a, T>(&'a mut [T]);

impl<T: Debug> Debug for WrappingSliceMut<'_, T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<'a, T, I> From<I> for WrappingSliceMut<'a, T>
where
	I: Into<&'a mut [T]>,
{
	fn from(value: I) -> Self {
		Self(value.into())
	}
}

impl<T> Index<usize> for WrappingSliceMut<'_, T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index % self.0.len()]
	}
}

impl<T> IndexMut<usize> for WrappingSliceMut<'_, T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index % self.0.len()]
	}
}
