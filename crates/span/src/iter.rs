use core::{
	cmp::Ordering,
	fmt::{Debug, Formatter, Result as FmtResult},
	iter::FusedIterator,
	mem,
	net::Ipv4Addr,
};

use vmm_num::{Checked, Unchecked, Wrapping};

use super::{Excluded, Included, Span, SpanBound, SpanStartBound, Unbounded};

pub struct SpanIter<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	pub(super) span: Span<T, From, To>,
	exhausted: bool,
}

impl<T, From, To> SpanIter<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
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

impl<T: Debug, From, To> Debug for SpanIter<T, From, To>
where
	From: Debug + SpanStartBound<T>,
	To: Debug + SpanBound<T>,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("SpanIter")
			.field("span", &self.span)
			.field("exhausted", &self.exhausted)
			.finish()
	}
}

impl<T: Eq, From, To> Eq for SpanIter<T, From, To>
where
	From: Eq + SpanStartBound<T>,
	To: Eq + SpanBound<T>,
{
}

impl<T: PartialEq, From, To> PartialEq for SpanIter<T, From, To>
where
	From: PartialEq + SpanStartBound<T>,
	To: PartialEq + SpanBound<T>,
{
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.span, &other.span)
	}
}

impl<T: PartialEq, From, To> PartialEq<Span<T, From, To>> for SpanIter<T, From, To>
where
	From: PartialEq + SpanStartBound<T>,
	To: PartialEq + SpanBound<T>,
{
	fn eq(&self, other: &Span<T, From, To>) -> bool {
		PartialEq::eq(&self.span, other)
	}
}

impl<T: Walk> DoubleEndedIterator for SpannedIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		let start = &self.span.start.0;
		let end = &self.span.end.0;

		if start < end {
			self.span.end.0 = Walk::backward_checked(end.clone(), 1)?;
			Some(self.span.end.0.clone())
		} else {
			None
		}
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		let end = &self.span.end.0;
		let start = &self.span.start.0;

		if let Some(minus_n) = Walk::backward_checked(end.clone(), n)
			&& minus_n > *start
		{
			self.span.end.0 = Walk::backward_checked(minus_n, 1)?;
			return Some(self.span.end.0.clone());
		}

		self.span.end.0 = self.span.start.0.clone();
		None
	}
}

impl<T: Walk> ExactSizeIterator for SpannedIter<T> {}

impl<T: Walk> FusedIterator for SpannedIter<T> {}

impl<T: Walk> Iterator for SpannedIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let start = &mut self.span.start.0;
		let end = &mut self.span.end.0;

		if start < end {
			let n = Walk::forward_checked(start.clone(), 1)?;
			Some(mem::replace(start, n))
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let start = &self.span.start.0;
		let end = &self.span.end.0;

		if start < end {
			Walk::steps_between(start, end)
		} else {
			(0, Some(0))
		}
	}

	fn count(self) -> usize {
		let start = &self.span.start.0;
		let end = &self.span.end.0;

		if start < end {
			Walk::steps_between(start, end)
				.1
				.expect("count overflowed usize")
		} else {
			0
		}
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let start = &mut self.span.start.0;
		let end = &self.span.end.0;

		if let Some(plus_n) = Walk::forward_checked(start.clone(), n)
			&& plus_n < *end
		{
			self.span.start.0 = Walk::forward_checked(plus_n.clone(), 1)?;

			return Some(plus_n);
		}

		self.span.start.0 = self.span.end.0.clone();
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
		let n = Walk::forward_checked(self.span.start.0.clone(), 1)?;
		Some(mem::replace(&mut self.span.start.0, n))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(usize::MAX, None)
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let plus_n = Walk::forward_checked(self.span.start.0.clone(), n)?;
		self.span.start.0 = Walk::forward_checked(plus_n.clone(), 1)?;
		Some(plus_n)
	}
}

impl<T: Walk> DoubleEndedIterator for SpannedInclusiveIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if Self::is_empty(self) {
			return None;
		}

		let start = &self.span.start.0;
		let end = &self.span.end.0;
		let is_iterating = start < end;
		Some(if is_iterating {
			let n = Walk::backward_checked(end.clone(), 1)?;
			mem::replace(&mut self.span.end.0, n)
		} else {
			self.exhausted = true;
			end.clone()
		})
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		if Self::is_empty(self) {
			return None;
		}

		if let Some(minus_n) = Walk::backward_checked(self.span.end.0.clone(), n) {
			match minus_n.partial_cmp(&self.span.start.0) {
				Some(Ordering::Greater) => {
					self.span.end.0 = Walk::backward_checked(minus_n.clone(), 1)?;
					return Some(minus_n);
				}
				Some(Ordering::Equal) => {
					self.span.end.0 = minus_n.clone();
					self.exhausted = true;
					return Some(minus_n);
				}
				_ => {}
			}
		}

		self.span.end.0 = self.span.start.0.clone();
		self.exhausted = true;
		None
	}
}

impl<T: Walk> ExactSizeIterator for SpannedInclusiveIter<T> {}

impl<T: Walk> FusedIterator for SpannedInclusiveIter<T> {}

impl<T: Walk> Iterator for SpannedInclusiveIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if Self::is_empty(self) {
			return None;
		}

		let start = &self.span.start.0;
		let end = &self.span.end.0;

		let is_iterating = start < end;

		Some(if is_iterating {
			let n = Walk::forward_checked(start.clone(), 1)?;
			mem::replace(&mut self.span.start.0, n)
		} else {
			self.exhausted = true;
			start.clone()
		})
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		if self.is_empty() {
			return (0, Some(0));
		}

		let start = &self.span.start.0;
		let end = &self.span.end.0;

		let hint = Walk::steps_between(start, end);
		(
			hint.0.saturating_add(1),
			hint.1.and_then(|steps| steps.checked_add(1)),
		)
	}

	fn count(self) -> usize {
		if Self::is_empty(&self) {
			return 0;
		}

		let start = &self.span.start.0;
		let end = &self.span.end.0;

		Walk::steps_between(start, end)
			.1
			.and_then(|steps| steps.checked_add(1))
			.expect("count overflowed usize")
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		if Self::is_empty(self) {
			return None;
		}

		if let Some(plus_n) = Walk::forward_checked(self.span.start.0.clone(), n) {
			match plus_n.partial_cmp(&self.span.end.0) {
				Some(Ordering::Less) => {
					self.span.start.0 = Walk::forward_checked(plus_n.clone(), 1)?;
					return Some(plus_n);
				}
				Some(Ordering::Equal) => {
					self.span.start.0 = plus_n.clone();
					self.exhausted = true;
					return Some(plus_n);
				}
				_ => {}
			}
		}

		self.span.start.0 = self.span.end.0.clone();
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

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::add(start, count as u16).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::sub(start, count as u16).unwrap_unchecked() }
	}
}

impl Walk for i32 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = Wrapping::sub(*end as isize, *start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u32::try_from(count).ok()?;

		let wrapped = Wrapping::add(start, n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u32::try_from(count).ok()?;

		let wrapped = Wrapping::sub(start, n as Self);
		(wrapped <= start).then_some(wrapped)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::add(start, count as u32).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::sub(start, count as u32).unwrap_unchecked() }
	}
}

impl Walk for i64 {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = Wrapping::sub(*end as isize, *start as isize) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u64::try_from(count).ok()?;

		let wrapped = Wrapping::add(start, n as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let n = u64::try_from(count).ok()?;

		let wrapped = Wrapping::sub(start, n as Self);
		(wrapped <= start).then_some(wrapped)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::add(start, count as u64).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::sub(start, count as u64).unwrap_unchecked() }
	}
}

impl Walk for isize {
	fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
		if *start <= *end {
			let steps = Wrapping::sub(end, start) as usize;
			(steps, Some(steps))
		} else {
			(0, None)
		}
	}

	fn forward_checked(start: Self, count: usize) -> Option<Self> {
		let wrapped = Wrapping::add(start, count as Self);
		(wrapped >= start).then_some(wrapped)
	}

	fn backward_checked(start: Self, count: usize) -> Option<Self> {
		let wrapped = Wrapping::sub(start, count as Self);
		(wrapped <= start).then_some(wrapped)
	}

	unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::add(start, count).unwrap_unchecked() }
	}

	unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
		unsafe { Checked::sub(start, count).unwrap_unchecked() }
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
#[expect(clippy::reversed_empty_ranges, clippy::iter_nth_zero)]
mod tests {
	extern crate alloc;

	use alloc::vec::Vec;

	use vmm_num::Checked;

	use crate::*;

	#[test]
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

	#[test]
	fn span_exhaustion() {
		let mut s = Span::from(10..10).into_iter();
		assert!(s.is_empty());
		assert_eq!(s.next(), None);
		assert_eq!(s.next_back(), None);
		assert_eq!(s, Span::from(10..10).into_iter());

		let mut s = Span::from(10..12).into_iter();
		assert_eq!(s.next(), Some(10));
		assert_eq!(s.next(), Some(11));
		assert!(s.is_empty());
		assert_eq!(s, Span::from(12..12));
		assert_eq!(s.next(), None);

		let mut s = Span::from(10..12).into_iter();
		assert_eq!(s.next_back(), Some(11));
		assert_eq!(s.next_back(), Some(10));
		assert!(s.is_empty());
		assert_eq!(s, Span::from(10..10));
		assert_eq!(s.next_back(), None);

		let mut s = Span::from(100..10).into_iter();
		assert!(s.is_empty());
		assert_eq!(s.next(), None);
		assert_eq!(s.next_back(), None);
		assert_eq!(s, Span::from(100..10));
	}

	#[test]
	fn span_inclusive_exhaustion() {
		let mut s = Span::from(10..=10).into_iter();
		assert_eq!(s.next(), Some(10));
		assert!(s.is_empty());
		assert_eq!(s.next(), None);
		assert_eq!(s.next(), None);

		let mut s = Span::from(10..=10).into_iter();
		assert_eq!(s.next_back(), Some(10));
		assert!(s.is_empty());
		assert_eq!(s.next_back(), None);

		let mut s = Span::from(10..=12).into_iter();
		assert_eq!(s.next(), Some(10));
		assert_eq!(s.next(), Some(11));
		assert_eq!(s.next(), Some(12));
		assert!(s.is_empty());
		assert_eq!(s.next(), None);

		let mut s = Span::from(10..=12).into_iter();
		assert_eq!(s.next_back(), Some(12));
		assert_eq!(s.next_back(), Some(11));
		assert_eq!(s.next_back(), Some(10));
		assert!(s.is_empty());
		assert_eq!(s.next_back(), None);

		let mut s = Span::from(10..=12).into_iter();
		assert_eq!(s.nth(2), Some(12));
		assert!(s.is_empty());
		assert_eq!(s.next(), None);

		let mut s = Span::from(10..=12).into_iter();
		assert_eq!(s.nth(5), None);
		assert!(s.is_empty());
		assert_eq!(s.next(), None);

		let mut s = Span::from(100..=10).into_iter();
		assert_eq!(s.next(), None);
		assert!(s.is_empty());
		assert_eq!(s.next(), None);
		assert_eq!(s.next(), None);
		assert_eq!(s, Span::from(100..=10));

		let mut s = Span::from(100..=10).into_iter();
		assert_eq!(s.next_back(), None);
		assert!(s.is_empty());
		assert_eq!(s.next_back(), None);
		assert_eq!(s.next_back(), None);
		assert_eq!(s, Span::from(100..=10));
	}

	#[test]
	fn span_nth() {
		let s = Span::from(10..15);
		assert_eq!(s.into_iter().nth(0), Some(10));
		assert_eq!(s.into_iter().nth(1), Some(11));
		assert_eq!(s.into_iter().nth(4), Some(14));
		assert_eq!(s.into_iter().nth(5), None);

		let mut s = Span::from(10..20).into_iter();
		assert_eq!(s.nth(2), Some(12));
		assert_eq!(s, Span::from(13..20));
		assert_eq!(s.nth(2), Some(15));
		assert_eq!(s, Span::from(16..20));
		assert_eq!(s.nth(10), None);
		assert_eq!(s, Span::from(20..20));
	}

	#[test]
	fn span_nth_back() {
		let s = Span::from(10..15);
		assert_eq!(s.into_iter().nth_back(0), Some(14));
		assert_eq!(s.into_iter().nth_back(1), Some(13));
		assert_eq!(s.into_iter().nth_back(4), Some(10));
		assert_eq!(s.into_iter().nth_back(5), None);

		let mut s = Span::from(10..20).into_iter();
		assert_eq!(s.nth_back(2), Some(17));
		assert_eq!(s, Span::from(10..17));
		assert_eq!(s.nth_back(2), Some(14));
		assert_eq!(s, Span::from(10..14));
		assert_eq!(s.nth_back(10), None);
		assert_eq!(s, Span::from(10..10));
	}

	#[test]
	fn span_from_nth() {
		let s = Span::from(10..);
		assert_eq!(s.into_iter().nth(0), Some(10));
		assert_eq!(s.into_iter().nth(1), Some(11));
		assert_eq!(s.into_iter().nth(4), Some(14));

		let mut s = Span::from(10..).into_iter();
		assert_eq!(s.nth(2), Some(12));
		assert_eq!(s, Span::from(13..));
		assert_eq!(s.nth(2), Some(15));
		assert_eq!(s, Span::from(16..));
		assert_eq!(s.nth(10), Some(26));
		assert_eq!(s, Span::from(27..));

		assert_eq!(Span::from(0..).into_iter().size_hint(), (usize::MAX, None));
	}

	#[test]
	fn span_from_take() {
		let mut s = Span::from(0..).into_iter().take(3);
		assert_eq!(s.next(), Some(0));
		assert_eq!(s.next(), Some(1));
		assert_eq!(s.next(), Some(2));
		assert_eq!(s.next(), None);
		assert_eq!(
			Span::from(0..).into_iter().take(3).size_hint(),
			(3, Some(3))
		);
		assert_eq!(
			Span::from(0..).into_iter().take(0).size_hint(),
			(0, Some(0))
		);
		assert_eq!(
			Span::from(0..).into_iter().take(usize::MAX).size_hint(),
			(usize::MAX, Some(usize::MAX))
		);
	}

	#[test]
	fn span_from_take_collect() {
		let v: Vec<_> = Span::from(0..).into_iter().take(3).collect();
		assert_eq!(v, [0, 1, 2]);
	}

	#[test]
	fn span_inclusive_nth() {
		let s = Span::from(10..=15);
		assert_eq!(s.into_iter().nth(0), Some(10));
		assert_eq!(s.into_iter().nth(1), Some(11));
		assert_eq!(s.into_iter().nth(5), Some(15));
		assert_eq!(s.into_iter().nth(6), None);

		let mut exhausted_via_next = Span::from(10u8..=20).into_iter();
		while exhausted_via_next.next().is_some() {}

		let mut s = Span::from(10u8..=20).into_iter();
		assert_eq!(s.nth(2), Some(12));
		assert_eq!(s, Span::from(13..=20));
		assert_eq!(s.nth(2), Some(15));
		assert_eq!(s, Span::from(16..=20));
		assert!(!s.is_empty());
		assert_eq!(s.nth(10), None);
		assert!(s.is_empty());
		assert_eq!(s, exhausted_via_next);
	}

	#[test]
	fn span_inclusive_nth_back() {
		let s = Span::from(10..=15);
		assert_eq!(s.into_iter().nth_back(0), Some(15));
		assert_eq!(s.into_iter().nth_back(1), Some(14));
		assert_eq!(s.into_iter().nth_back(5), Some(10));
		assert_eq!(s.into_iter().nth_back(6), None);
		assert_eq!(
			Span::from(-120..=80i8).into_iter().nth_back(200),
			Some(-120)
		);

		let mut exhasted_via_next_back = Span::from(10u8..=20).into_iter();
		while exhasted_via_next_back.next_back().is_some() {}

		let mut s = Span::from(10u8..=20).into_iter();
		assert_eq!(s.nth_back(2), Some(18));
		assert_eq!(s, Span::from(10..=17));
		assert_eq!(s.nth_back(2), Some(15));
		assert_eq!(s, Span::from(10..=14));
		assert!(!s.is_empty());
		assert_eq!(s.nth_back(10), None);
		assert!(s.is_empty());
		assert_eq!(s, exhasted_via_next_back);
	}

	#[test]
	fn span_step() {
		assert_eq!(
			Span::from(0..20)
				.into_iter()
				.step_by(5)
				.collect::<Vec<isize>>(),
			[0, 5, 10, 15]
		);
		assert_eq!(
			Span::from(1..21)
				.into_iter()
				.rev()
				.step_by(5)
				.collect::<Vec<isize>>(),
			[20, 15, 10, 5]
		);
		assert_eq!(
			Span::from(1..21)
				.into_iter()
				.rev()
				.step_by(6)
				.collect::<Vec<isize>>(),
			[20, 14, 8, 2]
		);
		assert_eq!(
			Span::from(200..255)
				.into_iter()
				.step_by(50)
				.collect::<Vec<u8>>(),
			[200, 250]
		);
		assert_eq!(
			Span::from(200..-5)
				.into_iter()
				.step_by(1)
				.collect::<Vec<isize>>(),
			[]
		);
		assert_eq!(
			Span::from(200..200)
				.into_iter()
				.step_by(1)
				.collect::<Vec<isize>>(),
			[]
		);

		assert_eq!(
			Span::from(0..20).into_iter().step_by(1).size_hint(),
			(20, Some(20))
		);
		assert_eq!(
			Span::from(0..20).into_iter().step_by(21).size_hint(),
			(1, Some(1))
		);
		assert_eq!(
			Span::from(0..20).into_iter().step_by(5).size_hint(),
			(4, Some(4))
		);
		assert_eq!(
			Span::from(1..21).into_iter().rev().step_by(5).size_hint(),
			(4, Some(4))
		);
		assert_eq!(
			Span::from(1..21).into_iter().rev().step_by(6).size_hint(),
			(4, Some(4))
		);
		assert_eq!(
			Span::from(20..-5).into_iter().step_by(1).size_hint(),
			(0, Some(0))
		);
		assert_eq!(
			Span::from(20..20).into_iter().step_by(1).size_hint(),
			(0, Some(0))
		);
		assert_eq!(
			Span::from(i8::MIN..i8::MAX)
				.into_iter()
				.step_by(-i32::from(i8::MIN) as usize)
				.size_hint(),
			(2, Some(2))
		);
		assert_eq!(
			Span::from(i16::MIN..i16::MAX)
				.into_iter()
				.step_by(i16::MAX as usize)
				.size_hint(),
			(3, Some(3))
		);
		assert_eq!(
			Span::from(isize::MIN..isize::MAX)
				.into_iter()
				.step_by(1)
				.size_hint(),
			(usize::MAX, Some(usize::MAX))
		);
	}

	#[test]
	fn span_inclusive_step() {
		assert_eq!(
			Span::from(0..=50)
				.into_iter()
				.step_by(10)
				.collect::<Vec<_>>(),
			[0, 10, 20, 30, 40, 50]
		);
		assert_eq!(
			Span::from(0..=5).into_iter().step_by(1).collect::<Vec<_>>(),
			[0, 1, 2, 3, 4, 5]
		);
		assert_eq!(
			Span::from(200u8..=255)
				.into_iter()
				.step_by(10)
				.collect::<Vec<_>>(),
			[200, 210, 220, 230, 240, 250]
		);
		assert_eq!(
			Span::from(250u8..=255)
				.into_iter()
				.step_by(1)
				.collect::<Vec<_>>(),
			[250, 251, 252, 253, 254, 255]
		);
	}

	#[test]
	fn span_last_max() {
		assert_eq!(Span::from(0..20).into_iter().last(), Some(19));
		assert_eq!(Span::from(-20..0).into_iter().last(), Some(-1));
		assert_eq!(Span::from(5..5).into_iter().last(), None);

		assert_eq!(Span::from(0..20).into_iter().max(), Some(19));
		assert_eq!(Span::from(-20..0).into_iter().max(), Some(-1));
		assert_eq!(Span::from(5..5).into_iter().max(), None);
	}

	#[test]
	fn span_inclusive_last_max() {
		assert_eq!(Span::from(0..=20).into_iter().last(), Some(20));
		assert_eq!(Span::from(-20..=0).into_iter().last(), Some(0));
		assert_eq!(Span::from(5..=5).into_iter().last(), Some(5));
		let mut s = Span::from(10..=10).into_iter();
		s.next();
		assert_eq!(s.last(), None);

		assert_eq!(Span::from(0..=20).into_iter().max(), Some(20));
		assert_eq!(Span::from(-20..=0).into_iter().max(), Some(0));
		assert_eq!(Span::from(5..=5).into_iter().max(), Some(5));
		let mut s = Span::from(10..=10).into_iter();
		s.next();
		assert_eq!(s.max(), None);
	}

	#[test]
	fn span_min() {
		assert_eq!(Span::from(0..20).into_iter().min(), Some(0));
		assert_eq!(Span::from(-20..0).into_iter().min(), Some(-20));
		assert_eq!(Span::from(5..5).into_iter().min(), None);
	}

	#[test]
	fn span_inclusive_min() {
		assert_eq!(Span::from(0..=20).into_iter().min(), Some(0));
		assert_eq!(Span::from(-20..=0).into_iter().min(), Some(-20));
		assert_eq!(Span::from(5..=5).into_iter().min(), Some(5));
		let mut s = Span::from(10..=10).into_iter();
		s.next();
		assert_eq!(s.min(), None);
	}

	#[test]
	fn span_inclusive_folds() {
		assert_eq!(Span::from(1..=10).into_iter().sum::<i32>(), 55);
		assert_eq!(Span::from(1..=10).into_iter().rev().sum::<i32>(), 55);

		let mut s = Span::from(44i8..=50).into_iter();
		assert_eq!(s.try_fold(0i8, Checked::add), None);
		assert_eq!(s, Span::from(47..=50));
		assert_eq!(s.try_fold(0i8, Checked::add), None);
		assert_eq!(s, Span::from(50..=50));
		assert_eq!(s.try_fold(0i8, Checked::add), Some(50));
		assert!(s.is_empty());
		assert_eq!(s.try_fold(0i8, Checked::add), Some(0));
		assert!(s.is_empty());

		let mut s = Span::from(40i8..=47).into_iter();
		assert_eq!(s.try_rfold(0i8, Checked::add), None);
		assert_eq!(s, Span::from(40..=44));
		assert_eq!(s.try_rfold(0i8, Checked::add), None);
		assert_eq!(s, Span::from(40..=41));
		assert_eq!(s.try_rfold(0i8, Checked::add), Some(81));
		assert!(s.is_empty());
		assert_eq!(s.try_rfold(0i8, Checked::add), Some(0));
		assert!(s.is_empty());

		let mut s = Span::from(10u8..=20).into_iter();
		assert_eq!(s.try_fold(0, |a, b| Some(a + b)), Some(165));
		assert!(s.is_empty());
		assert_eq!(s.try_fold(0, |a, b| Some(a + b)), Some(0));
		assert!(s.is_empty());

		let mut s = Span::from(10u8..=20).into_iter();
		assert_eq!(s.try_rfold(0, |a, b| Some(a + b)), Some(165));
		assert!(s.is_empty());
		assert_eq!(s.try_rfold(0, |a, b| Some(a + b)), Some(0));
		assert!(s.is_empty());
	}

	#[test]
	fn span_size_hint() {
		assert_eq!(Span::from(0..0usize).into_iter().size_hint(), (0, Some(0)));
		assert_eq!(
			Span::from(0..100usize).into_iter().size_hint(),
			(100, Some(100))
		);
		assert_eq!(
			Span::from(0..usize::MAX).into_iter().size_hint(),
			(usize::MAX, Some(usize::MAX))
		);

		assert_eq!(Span::from(0..0isize).into_iter().size_hint(), (0, Some(0)));
		assert_eq!(
			Span::from(-100..100isize).into_iter().size_hint(),
			(200, Some(200))
		);
		assert_eq!(
			Span::from(isize::MIN..isize::MAX).into_iter().size_hint(),
			(usize::MAX, Some(usize::MAX))
		);
	}

	#[test]
	#[allow(clippy::range_minus_one)]
	fn span_inclusive_size_hint() {
		assert_eq!(Span::from(1..=0usize).into_iter().size_hint(), (0, Some(0)));
		assert_eq!(Span::from(0..=0usize).into_iter().size_hint(), (1, Some(1)));
		assert_eq!(
			Span::from(0..=100usize).into_iter().size_hint(),
			(101, Some(101))
		);
		assert_eq!(
			Span::from(0..=usize::MAX - 1).into_iter().size_hint(),
			(usize::MAX, Some(usize::MAX))
		);
		assert_eq!(
			Span::from(0..=usize::MAX).into_iter().size_hint(),
			(usize::MAX, None)
		);

		assert_eq!(
			Span::from(0..=-1isize).into_iter().size_hint(),
			(0, Some(0))
		);
		assert_eq!(Span::from(0..=0isize).into_iter().size_hint(), (1, Some(1)));
		assert_eq!(
			Span::from(-100..=100isize).into_iter().size_hint(),
			(201, Some(201))
		);
		assert_eq!(
			Span::from(isize::MIN..=isize::MAX - 1)
				.into_iter()
				.size_hint(),
			(usize::MAX, Some(usize::MAX))
		);
		assert_eq!(
			Span::from(isize::MIN..=isize::MAX).into_iter().size_hint(),
			(usize::MAX, None)
		);
	}

	#[test]
	#[allow(clippy::never_loop)]
	fn double_ended_span() {
		assert_eq!(
			Span::from(11..14).into_iter().rev().collect::<Vec<_>>(),
			[13, 12, 11]
		);
		for _ in Span::from(10..0).into_iter().rev() {
			unreachable!();
		}

		assert_eq!(
			Span::from(11..14).into_iter().rev().collect::<Vec<_>>(),
			[13, 12, 11]
		);
		for _ in Span::from(10..0).into_iter().rev() {
			unreachable!();
		}
	}
}
