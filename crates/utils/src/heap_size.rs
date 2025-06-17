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
	use alloc::{
		borrow::{Cow, ToOwned},
		boxed::Box,
		string::String,
		vec::Vec,
	};

	use vmm_testing::{arbitrary, run_test};

	use super::HeapSize;

	trait TestHelper<'a>: arbitrary::Arbitrary<'a> + HeapSize {
		fn expected(&self) -> usize;
	}

	impl<'a, T> TestHelper<'a> for Vec<T>
	where
		T: arbitrary::Arbitrary<'a>,
	{
		fn expected(&self) -> usize {
			self.capacity() * core::mem::size_of::<T>()
		}
	}

	impl TestHelper<'_> for String {
		fn expected(&self) -> usize {
			self.capacity()
		}
	}

	impl<'a, T> TestHelper<'a> for Cow<'a, T>
	where
		T: ?Sized + ToOwned + 'a,
		T::Owned: TestHelper<'a>,
	{
		fn expected(&self) -> usize {
			match self {
				Self::Borrowed(_) => 0,
				Self::Owned(v) => v.expected(),
			}
		}
	}

	fn check<'a, T>(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<()>
	where
		T: TestHelper<'a>,
	{
		let value = u.arbitrary::<T>()?;

		assert_eq!(value.heap_size(), value.expected());

		Ok(())
	}

	#[test]
	#[expect(clippy::redundant_closure)]
	fn basic() {
		run_test(|u| check::<Vec<u8>>(u));
		run_test(|u| check::<Vec<u16>>(u));
		run_test(|u| check::<String>(u));
		run_test(|u| check::<Vec<()>>(u));
		run_test(|u| check::<Cow<'_, str>>(u));
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
