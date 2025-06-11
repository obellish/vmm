use vmm_num::Saturating;

use super::{
	IndexedParallelIterator, ParallelIterator,
	plumbing::{
		Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer, bridge_producer_consumer,
	},
};

#[derive(Debug, Clone)]
#[repr(transparent)]
#[must_use = "iterator adapters are lazy and do nothing unless consumed"]
pub struct ExponentialBlocks<I> {
	base: I,
}

impl<I> ExponentialBlocks<I> {
	pub(super) const fn new(base: I) -> Self {
		Self { base }
	}
}

impl<I: IndexedParallelIterator> ParallelIterator for ExponentialBlocks<I> {
	type Item = I::Item;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		let first = crate::current_num_threads();
		let callback = BlocksCallback {
			consumer,
			sizes: std::iter::successors(Some(first), exponential_size),
			len: self.base.len(),
		};

		self.base.with_producer(callback)
	}
}

#[derive(Debug, Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UniformBlocks<I> {
	base: I,
	block_size: usize,
}

impl<I> UniformBlocks<I> {
	pub(super) const fn new(base: I, block_size: usize) -> Self {
		Self { base, block_size }
	}
}

impl<I: IndexedParallelIterator> ParallelIterator for UniformBlocks<I> {
	type Item = I::Item;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		let callback = BlocksCallback {
			consumer,
			sizes: std::iter::repeat(self.block_size),
			len: self.base.len(),
		};

		self.base.with_producer(callback)
	}
}

struct BlocksCallback<S, C> {
	sizes: S,
	consumer: C,
	len: usize,
}

impl<T, S, C> ProducerCallback<T> for BlocksCallback<S, C>
where
	C: UnindexedConsumer<T>,
	S: Iterator<Item = usize>,
{
	type Output = C::Result;

	fn callback<P>(mut self, mut producer: P) -> Self::Output
	where
		P: Producer<Item = T>,
	{
		let mut remaining_len = self.len;
		let mut consumer = self.consumer;

		let (left_consumer, right_consumer, _) = consumer.split_at(0);
		let mut leftmost_res = left_consumer.into_folder().complete();
		consumer = right_consumer;

		while remaining_len > 0 && !consumer.full() {
			let size = self.sizes.next().unwrap_or(usize::MAX);
			let capped_size = remaining_len.min(size);
			remaining_len -= capped_size;

			let (left_producer, right_producer) = producer.split_at(capped_size);
			producer = right_producer;

			let (left_consumer, right_consumer, _) = consumer.split_at(capped_size);
			consumer = right_consumer;

			leftmost_res = consumer.to_reducer().reduce(
				leftmost_res,
				bridge_producer_consumer(capped_size, left_producer, left_consumer),
			);
		}

		leftmost_res
	}
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn exponential_size(size: &usize) -> Option<usize> {
	Some(Saturating::mul(size, 2))
}
