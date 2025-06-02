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
