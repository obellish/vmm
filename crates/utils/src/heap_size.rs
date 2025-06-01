use alloc::{boxed::Box, vec::Vec};
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
