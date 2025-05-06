#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut, Index, IndexMut},
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct WrappingArray<T, const N: usize>([T; N]);

impl<T, const N: usize> WrappingArray<T, N> {
	pub const fn new(value: T) -> Self
	where
		T: Copy,
	{
		Self([value; N])
	}

	pub fn into_array(self) -> [T; N] {
		self.0
	}

	#[expect(clippy::missing_const_for_fn)]
	pub fn as_slice(&self) -> &[T] {
		&**self
	}

	#[expect(clippy::missing_const_for_fn)]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		&mut **self
	}

	pub const fn as_wrapping_slice(&self) -> WrappingSlice<'_, T> {
		WrappingSlice(&self.0)
	}

	pub const fn as_mut_wrapping_slice(&mut self) -> WrappingSliceMut<'_, T> {
		WrappingSliceMut(&mut self.0)
	}
}

impl<T: Debug, const N: usize> Debug for WrappingArray<T, N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

impl<T: Default, const N: usize> Default for WrappingArray<T, N> {
	fn default() -> Self {
		Self(core::array::from_fn(|_| T::default()))
	}
}

impl<T, const N: usize> Deref for WrappingArray<T, N> {
	type Target = [T; N];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, const N: usize> DerefMut for WrappingArray<T, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T, const N: usize> Index<usize> for WrappingArray<T, N> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index % N]
	}
}

impl<T, const N: usize> IndexMut<usize> for WrappingArray<T, N> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index % N]
	}
}
