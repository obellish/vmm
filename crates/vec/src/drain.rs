use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	iter::FusedIterator,
	mem, ptr,
};

use super::SmallVec;

pub struct Drain<'a, T: 'a, const N: usize> {
	pub(super) tail_start: usize,
	pub(super) tail_len: usize,
	pub(super) iter: core::slice::Iter<'a, T>,
	pub(super) vec: core::ptr::NonNull<SmallVec<T, N>>,
}

impl<T, const N: usize> Drain<'_, T, N> {
	#[must_use]
	pub fn as_slice(&self) -> &[T] {
		self.iter.as_slice()
	}

	pub(super) unsafe fn fill<I>(&mut self, replace_with: &mut I) -> bool
	where
		I: Iterator<Item = T>,
	{
		let vec = unsafe { self.vec.as_mut() };
		let range_start = vec.len();
		let range_end = self.tail_start;
		let range_slice = unsafe {
			core::slice::from_raw_parts_mut(
				vec.as_mut_ptr().add(range_start),
				range_end - range_start,
			)
		};

		for place in range_slice {
			if let Some(new_item) = replace_with.next() {
				unsafe { ptr::write(place, new_item) };
				unsafe { vec.set_len(vec.len() + 1) };
			} else {
				return false;
			}
		}

		true
	}

	#[track_caller]
	pub(super) unsafe fn move_tail(&mut self, additional: usize) {
		let vec = unsafe { self.vec.as_mut() };
		let len = self.tail_start + self.tail_len;

		let old_len = vec.len();
		unsafe {
			vec.set_len(len);
			vec.reserve(additional);
			vec.set_len(old_len);
		}

		let new_tail_start = self.tail_start + additional;
		unsafe {
			let src = vec.as_ptr().add(self.tail_start);
			let dst = vec.as_mut_ptr().add(new_tail_start);
			ptr::copy(src, dst, self.tail_len);
		}

		self.tail_start = new_tail_start;
	}
}

impl<T: Debug, const N: usize> Debug for Drain<'_, T, N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
	}
}

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for Drain<'a, T, N> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.iter
			.next_back()
			.map(|reference| unsafe { ptr::read(reference) })
	}
}

impl<'a, T: 'a, const N: usize> Drop for Drain<'a, T, N> {
	fn drop(&mut self) {
		struct DropGuard<'r, 'a, T, const N: usize>(&'r mut Drain<'a, T, N>);

		impl<T, const N: usize> Drop for DropGuard<'_, '_, T, N> {
			fn drop(&mut self) {
				if self.0.tail_len > 0 {
					unsafe {
						let source_vec = self.0.vec.as_mut();
						let start = source_vec.len();
						let tail = self.0.tail_start;
						if tail != start {
							let ptr = source_vec.as_mut_ptr();
							let src = ptr.add(tail);
							let dst = ptr.add(start);
							ptr::copy(src, dst, self.0.tail_len);
						}
						source_vec.set_len(start + self.0.tail_len);
					}
				}
			}
		}

		let iter = mem::take(&mut self.iter);
		let drop_len = iter.len();

		let mut vec = self.vec;

		if SmallVec::<T, N>::is_zst() {
			unsafe {
				let vec = vec.as_mut();
				let old_len = vec.len();
				vec.set_len(old_len + drop_len + self.tail_len);
				vec.truncate(old_len + self.tail_len);
			}

			return;
		}

		let _guard = DropGuard(self);
		if matches!(drop_len, 0) {
			return;
		}

		let drop_ptr = iter.as_slice().as_ptr();

		unsafe {
			let vec_ptr = vec.as_mut().as_mut_ptr();
			let drop_offset = drop_ptr.offset_from(vec_ptr) as usize;
			let to_drop = ptr::slice_from_raw_parts_mut(vec_ptr.add(drop_offset), drop_len);
			ptr::drop_in_place(to_drop);
		}
	}
}

impl<'a, T: 'a, const N: usize> ExactSizeIterator for Drain<'a, T, N> {
	fn len(&self) -> usize {
		self.iter.len()
	}
}

impl<'a, T: 'a, const N: usize> FusedIterator for Drain<'a, T, N> {}

impl<'a, T: 'a, const N: usize> Iterator for Drain<'a, T, N> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter
			.next()
			.map(|reference| unsafe { ptr::read(reference) })
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}
