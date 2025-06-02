use alloc::vec::Vec;
use core::cmp::Ordering;

#[allow(clippy::return_self_not_must_use)]
pub trait Sorted<T: Ord> {
	fn sorted(self) -> Self;

	fn sorted_by(self, compare: impl FnMut(&T, &T) -> Ordering) -> Self;

	fn sorted_by_key<K: Ord>(self, compare: impl FnMut(&T) -> K) -> Self;
}

impl<T: Ord> Sorted<T> for Vec<T> {
	fn sorted(mut self) -> Self {
		self.sort();
		self
	}

	fn sorted_by(mut self, compare: impl FnMut(&T, &T) -> Ordering) -> Self {
		self.sort_by(compare);
		self
	}

	fn sorted_by_key<K: Ord>(mut self, compare: impl FnMut(&T) -> K) -> Self {
		self.sort_by_key(compare);
		self
	}
}
