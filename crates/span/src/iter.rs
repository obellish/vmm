use core::{iter::FusedIterator, marker::PhantomData, mem};

use super::SpanBound;
use crate::{Excluded, Included};

pub struct SpanIter<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
	start: Option<T>,
	end: Option<T>,
	marker_from: PhantomData<From>,
	marker_to: PhantomData<To>,
}

impl<T, From, To> SpanIter<T, From, To>
where
	From: ?Sized + SpanBound<T>,
	To: ?Sized + SpanBound<T>,
{
	pub(super) const fn new(start: Option<T>, end: Option<T>) -> Self {
		Self {
			start,
			end,
			marker_from: PhantomData,
			marker_to: PhantomData,
		}
	}
}

impl<T: Walk> DoubleEndedIterator for SpannedIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		debug_assert!(self.start.is_some());
		debug_assert!(self.end.is_some());

		let start = self.start.as_ref()?;
		let end = self.end.as_ref()?;

		if start < end {
			self.end = Walk::backward_checked(end.clone(), 1);
			self.end.clone()
		} else {
			None
		}
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		let end = self.end.as_ref()?;
		let start = self.start.as_ref()?;

		if let Some(minus_n) = Walk::backward_checked(end.clone(), n) {
			if minus_n > *start {
				self.end = Walk::backward_checked(minus_n, 1);
				return self.end.clone();
			}
		}

		self.end = self.start.clone();
		None
	}
}

impl<T: Walk> FusedIterator for SpannedIter<T> {}

impl<T: Walk> Iterator for SpannedIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		debug_assert!(self.start.is_some());
		debug_assert!(self.end.is_some());

		let start = self.start.as_mut()?;
		let end = self.end.as_mut()?;

		if start < end {
			let n = Walk::forward_checked(start.clone(), 1)?;
			Some(mem::replace(start, n))
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		debug_assert!(self.start.is_some());
		debug_assert!(self.end.is_some());

		let Some(start) = self.start.as_ref() else {
			return (0, None);
		};

		let Some(end) = self.end.as_ref() else {
			return (0, None);
		};

		if start < end {
			Walk::steps_between(start, end)
		} else {
			(0, Some(0))
		}
	}

	fn count(self) -> usize {
		debug_assert!(self.start.is_some());
		debug_assert!(self.end.is_some());

		let start = self.start.as_ref().unwrap();
		let end = self.end.as_ref().unwrap();

		if start < end {
			Walk::steps_between(start, end)
				.1
				.expect("count overflowed usize")
		} else {
			0
		}
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let start = self.start.as_mut()?;
		let end = self.end.as_ref()?;

		if let Some(plus_n) = Walk::forward_checked(start.clone(), n) {
			if plus_n < *end {
				self.start = Some(Walk::forward_checked(plus_n.clone(), 1)?);

				return Some(plus_n);
			}
		}

		self.start = self.end.clone();
		None
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

pub trait Walk: Clone + PartialOrd + Sized {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>);

	fn forward_checked(start: Self, count: usize) -> Option<Self>;

	fn backward_checked(start: Self, count: usize) -> Option<Self>;

	fn forward(start: Self, count: usize) -> Self {
		Self::forward_checked(start, count).expect("overflow in `Walk::forward`")
	}

	fn backward(start: Self, count: usize) -> Self {
		Self::backward_checked(start, count).expect("overflow in `Walk::backward`")
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Self::forward_checked(start, count).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Self::backward_checked(start, count).unwrap_unchecked() }
	}
}

impl Walk for u8 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end - *start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_add(n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_sub(n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_add(count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_sub(count as Self) }
	}
}

impl Walk for u16 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end - *start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_add(n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_sub(n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_add(count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_sub(count as Self) }
	}
}

impl Walk for u32 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end - *start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_add(n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_sub(n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_add(count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_sub(count as Self) }
	}
}

impl Walk for u64 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end - *start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_add(n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		start.checked_sub(n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_add(count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_sub(count as Self) }
	}
}

impl Walk for usize {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = *end - *start;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		start.checked_add(count)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		start.checked_sub(count)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_add(count) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { start.unchecked_sub(count) }
	}
}

impl Walk for i8 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end as isize).wrapping_sub(*start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u8::try_from(count).ok()?;

		let wrapped = start.wrapping_add(n as Self);
		if wrapped >= start {
			Some(wrapped)
		} else {
			None
		}
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u8::try_from(count).ok()?;

		let wrapped = start.wrapping_sub(n as Self);
		if wrapped <= start {
			Some(wrapped)
		} else {
			None
		}
	}
}

impl Walk for char {
	fn steps_between(&start: &Self, &end: &Self) -> (usize, Option<usize>) {
		let start = start as u32;
		let end = end as u32;
		if start <= end {
			let count = end - start;
			if start < 0xD800 && 0xE000 <= end {
				if let Ok(steps) = usize::try_from(count - 0x800) {
					(steps, Some(steps))
				} else {
					(usize::MAX, None)
				}
			} else if let Ok(steps) = usize::try_from(count) {
				(steps, Some(steps))
			} else {
				(usize::MAX, None)
			}
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let start = start as u32;
		let mut res = Walk::forward_checked(start, count)?;
		if start < 0xD800 && 0xD800 <= res {
			res = Walk::forward_checked(res, 0x800)?;
		}

		if res <= Self::MAX as u32 {
			Some(unsafe { Self::from_u32_unchecked(res) })
		} else {
			None
		}
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let start = start as u32;
		let mut res = Walk::backward_checked(start, count)?;
		if start >= 0xE000 && 0xE000 > res {
			res = Walk::backward_checked(res, 0x800)?;
		}

		Some(unsafe { Self::from_u32_unchecked(res) })
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		let start = start as u32;

		let mut res = unsafe { Walk::forward_unchecked(start, count) };
		if start < 0xD800 && 0xD800 <= res {
			res = unsafe { Walk::forward_unchecked(res, 0x800) };
		}

		unsafe { Self::from_u32_unchecked(res) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		let start = start as u32;

		let mut res = unsafe { Walk::backward_unchecked(start, count) };
		if start >= 0xE000 && 0xE000 > res {
			res = unsafe { Walk::backward_unchecked(res, 0x800) };
		}

		unsafe { Self::from_u32_unchecked(res) }
	}
}

pub type SpannedIter<T> = SpanIter<T, Included<T>, Excluded<T>>;

#[cfg(test)]
mod tests {
	extern crate alloc;

	use alloc::vec::Vec;

	use crate::*;

	#[test]
	fn span() {
		assert_eq!(
			Span::from(0u8..5).into_iter().collect::<Vec<_>>(),
			[0, 1, 2, 3, 4]
		);
	}
}
