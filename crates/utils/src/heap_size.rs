use alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	string::String,
	vec::Vec,
};
use core::mem::{size_of, size_of_val};

pub trait HeapSize {
	fn heap_size(&self) -> usize;
}

impl<T> HeapSize for Vec<T> {
	fn heap_size(&self) -> usize {
		self.capacity() * size_of::<T>()
	}
}

impl<T> HeapSize for Box<T> {
	fn heap_size(&self) -> usize {
		size_of::<T>()
	}
}

impl<T> HeapSize for Box<[T]> {
	fn heap_size(&self) -> usize {
		self.len() * size_of::<T>()
	}
}

impl<T> HeapSize for [T] {
	fn heap_size(&self) -> usize {
		size_of_val(self)
	}
}

impl HeapSize for String {
	fn heap_size(&self) -> usize {
		self.capacity() * size_of::<u8>()
	}
}

impl<T: HeapSize> HeapSize for Option<T> {
	fn heap_size(&self) -> usize {
		self.as_ref().map_or(0, HeapSize::heap_size)
	}
}

impl<T> HeapSize for Cow<'_, T>
where
	T: ?Sized + ToOwned,
	T::Owned: HeapSize,
{
	fn heap_size(&self) -> usize {
		match self {
			Self::Borrowed(_) => 0,
			Self::Owned(v) => v.heap_size(),
		}
	}
}

#[cfg(test)]
#[allow(unused_allocation)]
mod tests {
	use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};

	use quickcheck::{TestResult, quickcheck};

	use super::HeapSize;

	fn string_impl(a: String) -> TestResult {
		TestResult::from_bool(a.heap_size() == a.len())
	}

	fn vec_impl<T>(vec: Vec<T>) -> TestResult {
		TestResult::from_bool(vec.heap_size() == vec.capacity() * core::mem::size_of::<T>())
	}

	#[test]
	fn basic() {
		quickcheck(string_impl as fn(String) -> TestResult);
		quickcheck(vec_impl as fn(Vec<u8>) -> TestResult);
		quickcheck(vec_impl as fn(Vec<u16>) -> TestResult);
		quickcheck(vec_impl as fn(Vec<()>) -> TestResult);
	}

	#[test]
	fn empty_sizes() {
		assert_eq!(Vec::<u32>::new().heap_size(), 0);
		assert_eq!(Box::new(()).heap_size(), 0);
		assert_eq!(Vec::<u32>::new().into_boxed_slice().heap_size(), 0);

		assert_eq!(String::new().heap_size(), 0);
		assert_eq!(Option::<String>::None.heap_size(), 0);

		assert_eq!(Cow::Borrowed("hello, world!").heap_size(), 0);
	}

	#[test]
	fn cow() {
		let mut c = Cow::Borrowed(&[0u8, 1, 2, 3] as &[u8]);
		assert_eq!(c.heap_size(), 0);

		c.to_mut();

		assert_eq!(c.heap_size(), 4);
	}
}
