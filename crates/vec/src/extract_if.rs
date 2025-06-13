use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ptr,
};

use super::SmallVec;

pub struct ExtractIf<'a, T, const N: usize, F> {
	pub(super) vec: &'a mut SmallVec<T, N>,
	pub(super) idx: usize,
	pub(super) end: usize,
	pub(super) del: usize,
	pub(super) old_len: usize,
	pub(super) pred: F,
}

impl<T: Debug, const N: usize, F> Debug for ExtractIf<'_, T, N, F> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("ExtractIf")
			.field(&self.vec.as_slice())
			.finish()
	}
}

impl<T, const N: usize, F> Drop for ExtractIf<'_, T, N, F> {
	fn drop(&mut self) {
		unsafe {
			if self.idx < self.old_len && self.del > 0 {
				let ptr = self.vec.as_mut_ptr();
				let src = ptr.add(self.idx);
				let dst = src.sub(self.del);
				let tail_len = self.old_len - self.idx;
				src.copy_to(dst, tail_len);
			}

			self.vec.set_len(self.old_len - self.del);
		}
	}
}

impl<T, const N: usize, F> Iterator for ExtractIf<'_, T, N, F>
where
	F: FnMut(&mut T) -> bool,
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		unsafe {
			while self.idx < self.end {
				let i = self.idx;
				let v = core::slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
				let drained = (self.pred)(&mut v[i]);

				self.idx += 1;
				if drained {
					self.del += 1;
					return Some(ptr::read(&v[i]));
				} else if self.del > 0 {
					let del = self.del;
					let src: *const T = &v[i];
					let dst: *mut T = &mut v[i - del];
					ptr::copy_nonoverlapping(src, dst, 1);
				}
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.end - self.idx))
	}
}
