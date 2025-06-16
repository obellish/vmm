use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	iter::FusedIterator,
	marker::PhantomData,
	mem,
	ptr::{self, NonNull},
};

use super::{DropDealloc, RawSmallVec, SmallVec, TaggedLen};

pub struct IntoIter<T, const N: usize> {
	pub(super) raw: RawSmallVec<T, N>,
	pub(super) begin: usize,
	pub(super) end: TaggedLen,
	pub(super) marker: PhantomData<T>,
}

impl<T, const N: usize> IntoIter<T, N> {
	#[inline]
	const fn is_zst() -> bool {
		matches!(mem::size_of::<T>(), 0)
	}

	#[inline]
	const fn as_ptr(&self) -> *const T {
		let on_heap = self.end.on_heap(Self::is_zst());
		if on_heap {
			unsafe { self.raw.as_heap_ptr() }
		} else {
			self.raw.as_inline_ptr()
		}
	}

	#[inline]
	const fn as_mut_ptr(&mut self) -> *mut T {
		let on_heap = self.end.on_heap(Self::is_zst());
		if on_heap {
			unsafe { self.raw.as_mut_heap_ptr() }
		} else {
			self.raw.as_mut_inline_ptr()
		}
	}

	#[inline]
	pub const fn as_slice(&self) -> &[T] {
		unsafe {
			let ptr = self.as_ptr();
			core::slice::from_raw_parts(
				ptr.add(self.begin),
				self.end.value(Self::is_zst()) - self.begin,
			)
		}
	}

	#[inline]
	pub const fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe {
			let ptr = self.as_mut_ptr();
			core::slice::from_raw_parts_mut(
				ptr.add(self.begin),
				self.end.value(Self::is_zst()) - self.begin,
			)
		}
	}
}

impl<T: Clone, const N: usize> Clone for IntoIter<T, N> {
	fn clone(&self) -> Self {
		SmallVec::from(self.as_slice()).into_iter()
	}
}

impl<T: Debug, const N: usize> Debug for IntoIter<T, N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
	}
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		let mut end = self.end.value(Self::is_zst());
		if self.begin == end {
			None
		} else {
			unsafe {
				let ptr = self.as_mut_ptr();
				let on_heap = self.end.on_heap(Self::is_zst());
				end -= 1;
				self.end = TaggedLen::new(end, on_heap, Self::is_zst());
				let value = ptr.add(end).read();
				Some(value)
			}
		}
	}
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
	fn drop(&mut self) {
		unsafe {
			let is_zst = Self::is_zst();
			let on_heap = self.end.on_heap(is_zst);
			let begin = self.begin;
			let end = self.end.value(is_zst);
			let ptr = self.as_mut_ptr();
			let _drop_dealloc = if on_heap {
				let capacity = self.raw.heap.1;
				Some(DropDealloc {
					ptr: NonNull::new_unchecked(ptr),
					capacity,
				})
			} else {
				None
			};

			ptr::slice_from_raw_parts_mut(ptr.add(begin), end - begin).drop_in_place();
		}
	}
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
	type Item = T;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.begin == self.end.value(Self::is_zst()) {
			None
		} else {
			unsafe {
				let ptr = self.as_mut_ptr();
				let value = ptr.add(self.begin).read();
				self.begin += 1;
				Some(value)
			}
		}
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		let size = self.end.value(Self::is_zst()) - self.begin;
		(size, Some(size))
	}

	#[inline]
	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: Send, const N: usize> Send for IntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for IntoIter<T, N> {}
