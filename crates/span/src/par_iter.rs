use rayon::{
	iter::plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
	prelude::*,
};
use vmm_num::Wrapping;

use super::{Span, SpanBoundValue, SpannedIter, Walk};

#[repr(transparent)]
pub struct SpanParIter<T> {
	span: SpannedIter<T>,
}

impl<T> SpanParIter<T> {
	pub(super) const fn new(span: SpannedIter<T>) -> Self {
		Self { span }
	}
}

impl<T> ParallelIterator for SpanParIter<T>
where
	T: ParWalk + Send,
{
	type Item = T;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		bridge(self, consumer)
	}

	fn opt_len(&self) -> Option<usize> {
		Some(self.span.len())
	}
}

impl<T> IndexedParallelIterator for SpanParIter<T>
where
	T: ParWalk + Send,
{
	fn len(&self) -> usize {
		self.span.len()
	}

	fn drive<C>(self, consumer: C) -> C::Result
	where
		C: Consumer<Self::Item>,
	{
		bridge(self, consumer)
	}

	fn with_producer<CB>(self, callback: CB) -> CB::Output
	where
		CB: ProducerCallback<Self::Item>,
	{
		callback.callback(SpannedProducer { span: self.span })
	}
}

#[repr(transparent)]
struct SpannedProducer<T> {
	span: SpannedIter<T>,
}

impl<T: Walk> IntoIterator for SpannedProducer<T> {
	type IntoIter = SpannedIter<T>;
	type Item = <SpannedIter<T> as Iterator>::Item;

	fn into_iter(self) -> Self::IntoIter {
		self.span
	}
}

impl<T> Producer for SpannedProducer<T>
where
	T: ParWalk + Send,
	SpannedIter<T>: DoubleEndedIterator + ExactSizeIterator,
{
	type IntoIter = SpannedIter<T>;
	type Item = <SpannedIter<T> as Iterator>::Item;

	fn into_iter(self) -> Self::IntoIter {
		self.span
	}

	fn split_at(self, index: usize) -> (Self, Self) {
		assert!(index <= self.span.len());

		let start = self.span.span.start.value();
		let end = self.span.span.end.value();

		let mid = start.split_at(index);
		let left = Span::from(start.clone()..mid.clone()).into_iter();
		let right = Span::from(mid..end.clone()).into_iter();
		(Self { span: left }, Self { span: right })
	}
}

#[allow(clippy::return_self_not_must_use)]
pub trait ParWalk: Walk {
	fn split_at(&self, index: usize) -> Self;
}

impl ParWalk for i8 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for i16 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for i32 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for i64 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for isize {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for u8 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for u16 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for u32 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for u64 {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

impl ParWalk for usize {
	fn split_at(&self, index: usize) -> Self {
		Wrapping::add(self, index as Self)
	}
}

#[cfg(test)]
mod tests {
	extern crate alloc;

	use alloc::vec::Vec;

	use rayon::{iter::plumbing::Producer, prelude::*};

	use super::SpannedProducer;
	use crate::Span;

	#[test]
	fn span_split_at_overflow() {
		let producer = SpannedProducer {
			span: Span::from(-100i8..100).into_iter(),
		};
		let (left, right) = producer.split_at(150);
		let r1: i32 = left.span.map(i32::from).sum();
		let r2: i32 = right.span.map(i32::from).sum();
		assert_eq!(r1 + r2, -100);
	}

	#[test]
	fn type_inference_works() {
		fn is_even(n: i64) -> bool {
			matches!(n % 2, 0)
		}

		let v: Vec<_> = Span::from(1..100)
			.into_par_iter()
			.filter(|&x| is_even(x))
			.collect();
		assert!(v.into_iter().eq(Span::from(2..100).into_iter().step_by(2)));

		let pos = Span::from(0..100)
			.into_par_iter()
			.position_any(|x| matches!(x, 50i16));
		assert_eq!(pos, Some(50));

		assert!(
			Span::from(0..100)
				.into_par_iter()
				.zip(Span::from(0..100).into_par_iter())
				.all(|(a, b)| i16::eq(&a, &b))
		);
	}
}
