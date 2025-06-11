#![allow(clippy::return_self_not_must_use)]

use std::io::Split;

use super::IndexedParallelIterator;
use crate::join_context;

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Splitter {
	splits: usize,
}

impl Splitter {
	fn new() -> Self {
		Self {
			splits: crate::current_num_threads(),
		}
	}

	fn try_split(&mut self, stolen: bool) -> bool {
		let Self { splits } = *self;

		if stolen {
			self.splits = Ord::max(crate::current_num_threads(), self.splits / 2);
			true
		} else if splits > 0 {
			self.splits /= 2;
			true
		} else {
			false
		}
	}
}

#[derive(Clone, Copy)]
struct LengthSplitter {
	inner: Splitter,
	min: usize,
}

impl LengthSplitter {
	fn new(min: usize, max: usize, len: usize) -> Self {
		let mut splitter = Self {
			inner: Splitter::new(),
			min: Ord::max(min, 1),
		};

		let min_splits = len / Ord::max(max, 1);

		if min_splits > splitter.inner.splits {
			splitter.inner.splits = min_splits;
		}

		splitter
	}

	fn try_split(&mut self, len: usize, stolen: bool) -> bool {
		len / 2 >= self.min && self.inner.try_split(stolen)
	}
}

pub trait ProducerCallback<T> {
	type Output;

	fn callback<P>(self, producer: P) -> Self::Output
	where
		P: Producer<Item = T>;
}

pub trait Producer: Send + Sized {
	type Item;

	type IntoIter: DoubleEndedIterator<Item = Self::Item> + ExactSizeIterator;

	fn into_iter(self) -> Self::IntoIter;

	fn split_at(self, index: usize) -> (Self, Self);

	fn min_len(&self) -> usize {
		1
	}

	fn max_len(&self) -> usize {
		usize::MAX
	}

	fn fold_with<F>(self, folder: F) -> F
	where
		F: Folder<Self::Item>,
	{
		folder.consume_iter(self.into_iter())
	}
}

pub trait Reducer<Result> {
	fn reduce(self, left: Result, right: Result) -> Result;
}

pub trait Folder<Item>: Sized {
	type Result;

	fn consume(self, item: Item) -> Self;

	fn complete(self) -> Self::Result;

	fn full(&self) -> bool;

	#[must_use]
	fn consume_iter(mut self, iter: impl IntoIterator<Item = Item>) -> Self {
		for item in iter {
			self = self.consume(item);
			if self.full() {
				break;
			}
		}

		self
	}
}

pub trait Consumer<Item>: Send + Sized {
	type Folder: Folder<Item, Result = Self::Result>;

	type Reducer: Reducer<Self::Result>;

	type Result: Send;

	fn split_at(self, index: usize) -> (Self, Self, Self::Reducer);

	fn into_folder(self) -> Self::Folder;

	fn full(&self) -> bool;
}

pub trait UnindexedConsumer<I>: Consumer<I> {
	fn split_off_left(&self) -> Self;

	fn to_reducer(&self) -> Self::Reducer;
}

pub trait UnindexedProducer: Send + Sized {
	type Item;

	fn split(self) -> (Self, Option<Self>);

	fn fold_with<F>(self, folder: F) -> F
	where
		F: Folder<Self::Item>;
}

pub fn bridge<I: IndexedParallelIterator, C>(par_iter: I, consumer: C) -> C::Result
where
	C: Consumer<I::Item>,
{
	struct Callback<C> {
		len: usize,
		consumer: C,
	}

	impl<C, I> ProducerCallback<I> for Callback<C>
	where
		C: Consumer<I>,
	{
		type Output = C::Result;

		fn callback<P>(self, producer: P) -> Self::Output
		where
			P: Producer<Item = I>,
		{
			bridge_producer_consumer(self.len, producer, self.consumer)
		}
	}

	let len = par_iter.len();
	par_iter.with_producer(Callback { len, consumer })
}

pub fn bridge_producer_consumer<P: Producer, C>(len: usize, producer: P, consumer: C) -> C::Result
where
	C: Consumer<P::Item>,
{
	let splitter = LengthSplitter::new(producer.min_len(), producer.max_len(), len);

	bridge_producer_consumer_helper(len, false, splitter, producer, consumer)
}

pub fn bridge_unindexed<P: UnindexedProducer, C>(producer: P, consumer: C) -> C::Result
where
	C: UnindexedConsumer<P::Item>,
{
	let splitter = Splitter::new();
	bridge_unindexed_producer_consumer(false, splitter, producer, consumer)
}

fn bridge_unindexed_producer_consumer<P: UnindexedProducer, C>(
	migrated: bool,
	mut splitter: Splitter,
	producer: P,
	consumer: C,
) -> C::Result
where
	C: UnindexedConsumer<P::Item>,
{
	if consumer.full() {
		consumer.into_folder().complete()
	} else if splitter.try_split(migrated) {
		match producer.split() {
			(left_producer, Some(right_producer)) => {
				let (reducer, left_consumer, right_consumer) =
					(consumer.to_reducer(), consumer.split_off_left(), consumer);
				let bridge = bridge_unindexed_producer_consumer;
				let (left_result, right_result) = join_context(
					|context| bridge(context.migrated(), splitter, left_producer, left_consumer),
					|context| bridge(context.migrated(), splitter, right_producer, right_consumer),
				);

				reducer.reduce(left_result, right_result)
			}
			(producer, None) => producer.fold_with(consumer.into_folder()).complete(),
		}
	} else {
		producer.fold_with(consumer.into_folder()).complete()
	}
}

fn bridge_producer_consumer_helper<P: Producer, C>(
	len: usize,
	migrated: bool,
	mut splitter: LengthSplitter,
	producer: P,
	consumer: C,
) -> C::Result
where
	C: Consumer<P::Item>,
{
	if consumer.full() {
		consumer.into_folder().complete()
	} else if splitter.try_split(len, migrated) {
		let mid = len / 2;
		let (left_producer, right_producer) = producer.split_at(mid);
		let (left_consumer, right_consumer, reducer) = consumer.split_at(mid);
		let (left_result, right_result) = join_context(
			|context| {
				bridge_producer_consumer_helper(
					mid,
					context.migrated(),
					splitter,
					left_producer,
					left_consumer,
				)
			},
			|context| {
				bridge_producer_consumer_helper(
					len - mid,
					context.migrated(),
					splitter,
					right_producer,
					right_consumer,
				)
			},
		);

		reducer.reduce(left_result, right_result)
	} else {
		producer.fold_with(consumer.into_folder()).complete()
	}
}
