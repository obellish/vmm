use core::{cmp::Ordering, iter::FusedIterator, mem};

use crate::Offset;

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
				Step::backward_checked(self.end.clone(), 1).expect("`Step` invariants not upheld");
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

	fn max(mut self) -> Option<Self::Item>
	where
		Self::Item: Ord,
	{
		self.next_back()
	}

	fn is_sorted(self) -> bool {
		true
	}
}

pub trait Step: Clone + PartialOrd + Sized {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>);

	fn forward_checked(start: Self, count: usize) -> Option<Self>;

	fn backward_checked(start: Self, count: usize) -> Option<Self>;

	fn forward(start: Self, count: usize) -> Self {
		Self::forward_checked(start, count).expect("overflow in `Step::forward`")
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Self::forward_checked(start, count).unwrap_unchecked() }
	}

	fn backward(start: Self, count: usize) -> Self {
		Self::backward_checked(start, count).expect("overflow in `Step::backward`")
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Self::backward_checked(start, count).unwrap_unchecked() }
	}
}

impl Step for isize {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = end.wrapping_sub(*start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let wrapped = start.wrapping_add(count as Self);
		if wrapped >= start {
			Some(wrapped)
		} else {
			None
		}
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let wrapped = start.wrapping_sub(count as Self);
		if wrapped <= start {
			Some(wrapped)
		} else {
			None
		}
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.checked_add_unsigned(count).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.checked_sub_unsigned(count).unwrap_unchecked() }
	}
}

impl Step for Offset {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		isize::steps_between(&start.0, &end.0)
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(isize::forward_checked(start.0, count)?))
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		Some(Self(isize::backward_checked(start.0, count)?))
	}
}
