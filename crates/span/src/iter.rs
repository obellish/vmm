use core::{cmp::Ordering, iter::FusedIterator, mem, net::Ipv4Addr};

use vmm_num::{Checked, Unchecked, Wrapping};

use super::{Excluded, Included, Span, SpanBound, SpanStartBound, Unbounded};

pub struct SpanIter<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	span: Span<T, From, To>,
	exhausted: bool,
}

impl<T, From, To> SpanIter<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	pub(super) const fn new(span: Span<T, From, To>) -> Self {
		Self {
			span,
			exhausted: false,
		}
	}

	fn is_empty(&self) -> bool
	where
		T: PartialOrd,
	{
		self.span.is_empty() || self.exhausted
	}
}

impl<T: Walk> DoubleEndedIterator for SpannedIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		debug_assert!(self.span.start.is_some());
		debug_assert!(self.span.end.is_some());

		let start = self.span.start.as_ref()?;
		let end = self.span.end.as_ref()?;

		if start < end {
			self.span.end = Walk::backward_checked(end.clone(), 1);
			self.span.end.clone()
		} else {
			None
		}
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		let end = self.span.end.as_ref()?;
		let start = self.span.start.as_ref()?;

		if let Some(minus_n) = Walk::backward_checked(end.clone(), n) {
			if minus_n > *start {
				self.span.end = Walk::backward_checked(minus_n, 1);
				return self.span.end.clone();
			}
		}

		self.span.end = self.span.start.clone();
		None
	}
}

impl<T: Walk> FusedIterator for SpannedIter<T> {}

impl<T: Walk> Iterator for SpannedIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		debug_assert!(self.span.start.is_some());
		debug_assert!(self.span.end.is_some());

		let start = self.span.start.as_mut()?;
		let end = self.span.end.as_mut()?;

		if start < end {
			let n = Walk::forward_checked(start.clone(), 1)?;
			Some(mem::replace(start, n))
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		debug_assert!(self.span.start.is_some());
		debug_assert!(self.span.end.is_some());

		let Some(start) = self.span.start.as_ref() else {
			return (0, None);
		};

		let Some(end) = self.span.end.as_ref() else {
			return (0, None);
		};

		if start < end {
			Walk::steps_between(start, end)
		} else {
			(0, Some(0))
		}
	}

	fn count(self) -> usize {
		debug_assert!(self.span.start.is_some());
		debug_assert!(self.span.end.is_some());

		let start = self.span.start.as_ref().unwrap();
		let end = self.span.end.as_ref().unwrap();

		if start < end {
			Walk::steps_between(start, end)
				.1
				.expect("count overflowed usize")
		} else {
			0
		}
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let start = self.span.start.as_mut()?;
		let end = self.span.end.as_ref()?;

		if let Some(plus_n) = Walk::forward_checked(start.clone(), n) {
			if plus_n < *end {
				self.span.start = Some(Walk::forward_checked(plus_n.clone(), 1)?);

				return Some(plus_n);
			}
		}

		self.span.start = self.span.end.clone();
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

impl<T: Walk> FusedIterator for SpannedFromIter<T> {}

impl<T: Walk> Iterator for SpannedFromIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let n = Walk::forward_checked(self.span.start.clone()?, 1);
		mem::replace(&mut self.span.start, n)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(usize::MAX, None)
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let plus_n = Walk::forward_checked(self.span.start.clone()?, n)?;
		self.span.start = Walk::forward_checked(plus_n.clone(), 1);
		Some(plus_n)
	}
}

impl<T: Walk> DoubleEndedIterator for SpannedInclusiveIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		let start = self.span.start.as_ref()?;
		let end = self.span.end.as_ref()?;
		let is_iterating = start < end;
		Some(if is_iterating {
			let n = Walk::backward_checked(end.clone(), 1);
			mem::replace(&mut self.span.end, n)?
		} else {
			self.exhausted = true;
			end.clone()
		})
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		if let Some(minus_n) = Walk::backward_checked(self.span.end.clone()?, n) {
			match minus_n.partial_cmp(self.span.start.as_ref()?) {
				Some(Ordering::Greater) => {
					self.span.end = Walk::backward_checked(minus_n.clone(), 1);
					return Some(minus_n);
				}
				Some(Ordering::Equal) => {
					self.span.end = Some(minus_n.clone());
					self.exhausted = true;
					return Some(minus_n);
				}
				_ => {}
			}
		}

		self.span.end = self.span.start.clone();
		self.exhausted = true;
		None
	}
}

impl<T: Walk> FusedIterator for SpannedInclusiveIter<T> {}

impl<T: Walk> Iterator for SpannedInclusiveIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		let start = self.span.start.as_ref()?;
		let end = self.span.end.as_ref()?;
		let is_iterating = start < end;
		Some(if is_iterating {
			let n = Walk::forward_checked(start.clone(), 1);
			mem::replace(&mut self.span.start, n)?
		} else {
			self.exhausted = true;
			start.clone()
		})
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		if self.is_empty() {
			return (0, Some(0));
		}

		let Some(start) = self.span.start.as_ref() else {
			return (0, Some(0));
		};

		let Some(end) = self.span.end.as_ref() else {
			return (0, Some(0));
		};

		let hint = Walk::steps_between(start, end);
		(
			hint.0.saturating_add(1),
			hint.1.and_then(|steps| steps.checked_add(1)),
		)
	}

	fn count(self) -> usize {
		if self.is_empty() {
			return 0;
		}

		let Some((start, end)) = self.span.start.as_ref().zip(self.span.end.as_ref()) else {
			return 0;
		};

		Walk::steps_between(start, end)
			.1
			.and_then(|steps| steps.checked_add(1))
			.expect("count overflowed usize")
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if self.is_empty() {
			return None;
		}

		if let Some(plus_n) = Walk::forward_checked(self.span.start.clone()?, n) {
			match plus_n.partial_cmp(self.span.end.as_ref()?) {
				Some(Ordering::Less) => {
					self.span.start = Walk::forward_checked(plus_n.clone(), 1);
					return Some(plus_n);
				}
				Some(Ordering::Equal) => {
					self.span.start = Some(plus_n.clone());
					self.exhausted = true;
					return Some(plus_n);
				}
				_ => {}
			}
		}

		self.span.start = self.span.end.clone();
		self.exhausted = true;
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

		Checked::add(start, n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		Checked::sub(start, n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::add(start, count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::sub(start, count as Self) }
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

		Checked::add(start, n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		Checked::sub(start, n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::add(start, count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::sub(start, count as Self) }
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

		Checked::add(start, n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		Checked::sub(start, n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::add(start, count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::sub(start, count as Self) }
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

		Checked::add(start, n)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = Self::try_from(count).ok()?;

		Checked::sub(start, n)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::add(start, count as Self) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::sub(start, count as Self) }
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
		Checked::add(start, count)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		Checked::sub(start, count)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::add(start, count) }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Unchecked::sub(start, count) }
	}
}

impl Walk for i8 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = Wrapping::sub(*end as isize, *start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u8::try_from(count).ok()?;

		let wrapped = Wrapping::add(start, n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u8::try_from(count).ok()?;

		let wrapped = Wrapping::sub(start, n as Self);
		(wrapped <= start).then_some(wrapped)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::add(start, count as u8).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::sub(start, count as u8).unwrap_unchecked() }
	}
}

impl Walk for i16 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = Wrapping::sub(*end as isize, *start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u16::try_from(count).ok()?;

		let wrapped = Wrapping::add(start, n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u16::try_from(count).ok()?;

		let wrapped = Wrapping::sub(start, n as Self);
		(wrapped <= start).then_some(wrapped)
	}
}

impl Walk for i32 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end as isize).wrapping_sub(*start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u32::try_from(count).ok()?;

		let wrapped = start.wrapping_add(n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u32::try_from(count).ok()?;

		let wrapped = start.wrapping_sub(n as Self);
		(wrapped <= start).then_some(wrapped)
	}
}

impl Walk for i64 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = (*end as isize).wrapping_sub(*start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u64::try_from(count).ok()?;

		let wrapped = start.wrapping_add(n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u64::try_from(count).ok()?;

		let wrapped = start.wrapping_sub(n as Self);
		(wrapped <= start).then_some(wrapped)
	}
}

impl Walk for isize {
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
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let wrapped = start.wrapping_sub(count as Self);
		(wrapped <= start).then_some(wrapped)
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

impl Walk for Ipv4Addr {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		u32::steps_between(&start.to_bits(), &end.to_bits())
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		u32::forward_checked(start.to_bits(), count).map(Self::from_bits)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		u32::backward_checked(start.to_bits(), count).map(Self::from_bits)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		Self::from_bits(unsafe { u32::forward_unchecked(start.to_bits(), count) })
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		Self::from_bits(unsafe { u32::backward_unchecked(start.to_bits(), count) })
	}
}

pub type SpannedIter<T> = SpanIter<T, Included<T>, Excluded<T>>;

pub type SpannedFromIter<T> = SpanIter<T, Included<T>, Unbounded<T>>;

pub type SpannedInclusiveIter<T> = SpanIter<T, Included<T>, Included<T>>;

#[cfg(test)]
mod tests {
	extern crate alloc;

	use alloc::vec::Vec;

	use crate::*;

	#[test]
	#[expect(clippy::reversed_empty_ranges)]
	fn span() {
		assert_eq!(
			Span::from(0..5).into_iter().collect::<Vec<_>>(),
			[0, 1, 2, 3, 4]
		);

		assert_eq!(
			Span::from(-10..-1).into_iter().collect::<Vec<_>>(),
			[-10, -9, -8, -7, -6, -5, -4, -3, -2]
		);

		assert_eq!(
			Span::from(0..5).into_iter().rev().collect::<Vec<_>>(),
			[4, 3, 2, 1, 0]
		);

		assert_eq!(Span::from(200..-5).into_iter().count(), 0);
		assert_eq!(Span::from(200..-5).into_iter().rev().count(), 0);
		assert_eq!(Span::from(200..200).into_iter().count(), 0);
		assert_eq!(Span::from(200..200).into_iter().rev().count(), 0);

		assert_eq!(Span::from(0..100).into_iter().size_hint(), (100, Some(100)));

		assert_eq!(
			Span::from(usize::MAX - 1..usize::MAX)
				.into_iter()
				.size_hint(),
			(1, Some(1))
		);
		assert_eq!(Span::from(-10..-1).into_iter().size_hint(), (9, Some(9)));
		assert_eq!(Span::from(-1..-10).into_iter().size_hint(), (0, Some(0)));

		assert_eq!(
			Span::from(-70..58).into_iter().size_hint(),
			(128, Some(128))
		);
		assert_eq!(
			Span::from(-128..127).into_iter().size_hint(),
			(255, Some(255))
		);
		assert_eq!(
			Span::from(-2..isize::MAX).into_iter().size_hint(),
			(isize::MAX as usize + 2, Some(isize::MAX as usize + 2))
		);
	}

	#[test]
	fn char_span() {
		let from = if cfg!(miri) {
			char::from_u32(0xD800 - 10).unwrap()
		} else {
			'\0'
		};
		let to = if cfg!(miri) {
			char::from_u32(0xDFFF + 10).unwrap()
		} else {
			char::MAX
		};
		assert!(
			Span::from(from..=to)
				.into_iter()
				.eq(Span::from(from as u32..=to as u32)
					.into_iter()
					.filter_map(char::from_u32))
		);
	}
}
