use super::{Drain, SmallVec};

pub struct Splice<'a, I, const N: usize>
where
	I: Iterator + 'a,
{
	pub(super) drain: Drain<'a, I::Item, N>,
	pub(super) replace_with: I,
}

impl<I: Iterator, const N: usize> DoubleEndedIterator for Splice<'_, I, N> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.drain.next_back()
	}
}

impl<I: Iterator, const N: usize> Drop for Splice<'_, I, N> {
	#[allow(clippy::iter_on_empty_collections)]
	fn drop(&mut self) {
		self.drain.by_ref().for_each(drop);
		self.drain.iter = [].iter();

		unsafe {
			if matches!(self.drain.tail_len, 0) {
				self.drain.vec.as_mut().extend(self.replace_with.by_ref());
				return;
			}

			if !self.drain.fill(&mut self.replace_with) {
				return;
			}

			let (lower_bound, _) = self.replace_with.size_hint();
			if lower_bound > 0 {
				self.drain.move_tail(lower_bound);
				if !self.drain.fill(&mut self.replace_with) {
					return;
				}
			}

			let mut collected = self
				.replace_with
				.by_ref()
				.collect::<SmallVec<I::Item, N>>()
				.into_iter();
			if collected.len() > 0 {
				self.drain.move_tail(collected.len());
				let filled = self.drain.fill(&mut collected);
				debug_assert!(filled);
				debug_assert_eq!(collected.len(), 0);
			}
		}
	}
}

impl<I: Iterator, const N: usize> ExactSizeIterator for Splice<'_, I, N> {
	fn len(&self) -> usize {
		self.drain.len()
	}
}

impl<I: Iterator, const N: usize> Iterator for Splice<'_, I, N> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.drain.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.drain.size_hint()
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}
