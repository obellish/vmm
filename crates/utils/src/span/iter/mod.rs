mod step;

use core::{cmp::Ordering, iter::FusedIterator, mem};

pub use self::step::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IterSpanInclusive<Idx> {
	start: Idx,
	end: Idx,
	exhausted: bool,
}

impl<Idx> IterSpanInclusive<Idx> {
	pub(super) const fn new(start: Idx, end: Idx) -> Self {
		Self {
			start,
			end,
			exhausted: false,
		}
	}
}

impl<Idx: PartialOrd> IterSpanInclusive<Idx> {
	pub fn is_empty(&self) -> bool {
		self.exhausted
			|| matches!(
				self.start.partial_cmp(&self.end),
				None | Some(Ordering::Greater)
			)
	}
}

impl<A: Step> DoubleEndedIterator for IterSpanInclusive<A> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		let is_iterating = self.start < self.end;
		Some(if is_iterating {
			let n =
				Step::backward_checked(self.end.clone(), 1).expect("`Step` invariant not upheld");
			mem::replace(&mut self.end, n)
		} else {
			self.exhausted = true;
			self.end.clone()
		})
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		if let Some(minus_n) = Step::backward_checked(self.end.clone(), n) {
			match minus_n.partial_cmp(&self.start) {
				Some(Ordering::Greater) => {
					self.end = Step::backward(minus_n.clone(), 1);
					return Some(minus_n);
				}
				Some(Ordering::Equal) => {
					self.end = minus_n.clone();
					self.exhausted = true;
					return Some(minus_n);
				}
				_ => {}
			}
		}

		self.end = self.start.clone();
		self.exhausted = true;
		None
	}
}

impl<A: Step> FusedIterator for IterSpanInclusive<A> {}

impl<A: Step> Iterator for IterSpanInclusive<A> {
	type Item = A;

	fn next(&mut self) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		let is_iterating = self.start < self.end;
		Some(if is_iterating {
			let n =
				Step::forward_checked(self.start.clone(), 1).expect("`Step` invariants not upheld");
			mem::replace(&mut self.start, n)
		} else {
			self.exhausted = true;
			self.start.clone()
		})
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		if self.is_empty() {
			return (0, Some(0));
		}

		let hint = Step::steps_between(&self.start, &self.end);
		(
			hint.0.saturating_add(1),
			hint.1.and_then(|steps| steps.checked_add(1)),
		)
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		if let Some(plus_n) = Step::forward_checked(self.start.clone(), n) {
			match plus_n.partial_cmp(&self.end) {
				Some(Ordering::Less) => {
					self.start = Step::forward(plus_n.clone(), 1);
					return Some(plus_n);
				}
				Some(Ordering::Equal) => {
					self.start = plus_n.clone();
					self.exhausted = true;
					return Some(plus_n);
				}
				_ => {}
			}
		}

		self.start = self.end.clone();
		self.exhausted = true;
		None
	}

	fn count(self) -> usize {
		if self.is_empty() {
			return 0;
		}

		Step::steps_between(&self.start, &self.end)
			.1
			.and_then(|steps| steps.checked_add(1))
			.expect("count overflowed usize")
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}

	fn min(mut self) -> Option<Self::Item> {
		self.next()
	}

	fn max(mut self) -> Option<Self::Item> {
		self.next_back()
	}

	fn is_sorted(self) -> bool {
		true
	}
}
