use rayon::{
	iter::plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
	prelude::*,
};
use vmm_num::Wrapping;

use super::{Excluded, Included, Span, SpanBound, SpanBoundValue, SpanIter, SpanStartBound, Walk};

pub struct SpanParIter<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	span: SpanIter<T, From, To>,
}

impl<T, From, To> SpanParIter<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	pub(super) const fn new(span: SpanIter<T, From, To>) -> Self {
		Self { span }
	}
}

impl<T, From, To> ParallelIterator for SpanParIter<T, From, To>
where
	T: ParWalk + Send,
	From: Send + SpanBoundValue<T> + SpanStartBound<T>,
	To: Send + SpanBoundValue<T>,
	SpanIter<T, From, To>: DoubleEndedIterator<Item = T> + ExactSizeIterator,
	Span<T, From, To>: IntoIterator<IntoIter = SpanIter<T, From, To>>,
{
	type Item = T;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		bridge(self, consumer)
	}
}

impl<T, From, To> IndexedParallelIterator for SpanParIter<T, From, To>
where
	T: ParWalk + Send,
	From: Send + SpanBoundValue<T> + SpanStartBound<T>,
	To: Send + SpanBoundValue<T>,
	SpanIter<T, From, To>: DoubleEndedIterator<Item = T> + ExactSizeIterator,
	Span<T, From, To>: IntoIterator<IntoIter = SpanIter<T, From, To>>,
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
		callback.callback(SpanProducer {
			span: self.span.into(),
		})
	}
}

#[repr(transparent)]
struct SpanProducer<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
{
	span: Span<T, From, To>,
}

impl<T, From, To> IntoIterator for SpanProducer<T, From, To>
where
	From: SpanStartBound<T>,
	To: SpanBound<T>,
	SpanIter<T, From, To>: Iterator,
	Span<T, From, To>: IntoIterator<IntoIter = SpanIter<T, From, To>>,
{
	type IntoIter = SpanIter<T, From, To>;
	type Item = <SpanIter<T, From, To> as Iterator>::Item;

	fn into_iter(self) -> Self::IntoIter {
		self.span.into_iter()
	}
}

impl<T, From, To> Producer for SpanProducer<T, From, To>
where
	T: ParWalk + Send,
	From: Send + SpanBoundValue<T> + SpanStartBound<T>,
	To: Send + SpanBoundValue<T> + SpanBound<T>,
	SpanIter<T, From, To>: DoubleEndedIterator<Item = T> + ExactSizeIterator,
	Span<T, From, To>: IntoIterator<IntoIter = SpanIter<T, From, To>>,
{
	type IntoIter = SpanIter<T, From, To>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		self.span.into_iter()
	}

	fn split_at(self, index: usize) -> (Self, Self) {
		let iter = Producer::into_iter(self);
		assert!(index <= iter.len());

		let start = iter.span.start.value().clone();
		let end = iter.span.end.value().clone();

		let mid = start.split_at(index);
		let left = Span::from((From::from(start), To::from(mid.clone())));
		let right = Span::from((From::from(mid), To::from(end)));

		(Self { span: left }, Self { span: right })
	}
}

pub type SpannedParIter<T> = SpanParIter<T, Included<T>, Excluded<T>>;

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

	use super::SpanProducer;
	use crate::Span;

	#[test]
	fn span_split_at_overflow() {
		let producer = SpanProducer {
			span: Span::from(-100i8..100),
		};
		let (left, right) = producer.split_at(150);
		let r1: i32 = left.span.into_iter().map(i32::from).sum();
		let r2: i32 = right.span.into_iter().map(i32::from).sum();
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
