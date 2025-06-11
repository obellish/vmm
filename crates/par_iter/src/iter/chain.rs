use std::iter;

use vmm_num::Checked;

use super::{
	IndexedParallelIterator, ParallelIterator,
	plumbing::{Consumer, Folder, Producer, ProducerCallback, Reducer, UnindexedConsumer},
};
use crate::join;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct Chain<A, B> {
	a: A,
	b: B,
}

impl<A, B> Chain<A, B> {
	pub(super) const fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: ParallelIterator, B> ParallelIterator for Chain<A, B>
where
	B: ParallelIterator<Item = A::Item>,
{
	type Item = A::Item;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		let Self { a, b } = self;

		let (left, right, reducer) = if let Some(len) = a.opt_len() {
			consumer.split_at(len)
		} else {
			let reducer = consumer.to_reducer();
			(consumer.split_off_left(), consumer, reducer)
		};

		let (a, b) = join(|| a.drive_unindexed(left), || b.drive_unindexed(right));

		reducer.reduce(a, b)
	}

	fn opt_len(&self) -> Option<usize> {
		Checked::add(self.a.opt_len()?, self.b.opt_len()?)
	}
}

impl<A: IndexedParallelIterator, B> IndexedParallelIterator for Chain<A, B>
where
	B: IndexedParallelIterator<Item = A::Item>,
{
	fn drive<C>(self, consumer: C) -> C::Result
	where
		C: Consumer<Self::Item>,
	{
		let Self { a, b } = self;
		let (left, right, reducer) = consumer.split_at(a.len());
		let (a, b) = join(|| a.drive(left), || b.drive(right));
		reducer.reduce(a, b)
	}

	fn len(&self) -> usize {
		Checked::add(self.a.len(), self.b.len()).expect("overflow")
	}

	fn with_producer<CB>(self, callback: CB) -> CB::Output
	where
		CB: ProducerCallback<Self::Item>,
	{
		struct CallbackA<CB, B> {
			callback: CB,
			a_len: usize,
			b: B,
		}

		impl<CB, B: IndexedParallelIterator> ProducerCallback<B::Item> for CallbackA<CB, B>
		where
			CB: ProducerCallback<B::Item>,
		{
			type Output = CB::Output;

			fn callback<P>(self, producer: P) -> Self::Output
			where
				P: Producer<Item = B::Item>,
			{
				self.b.with_producer(CallbackB {
					callback: self.callback,
					a_len: self.a_len,
					a_producer: producer,
				})
			}
		}

		struct CallbackB<CB, A> {
			callback: CB,
			a_len: usize,
			a_producer: A,
		}

		impl<CB, A: Producer> ProducerCallback<A::Item> for CallbackB<CB, A>
		where
			CB: ProducerCallback<A::Item>,
		{
			type Output = CB::Output;

			fn callback<P>(self, producer: P) -> Self::Output
			where
				P: Producer<Item = A::Item>,
			{
				let producer = ChainProducer::new(self.a_len, self.a_producer, producer);
				self.callback.callback(producer)
			}
		}

		let a_len = self.a.len();
		self.a.with_producer(CallbackA {
			callback,
			a_len,
			b: self.b,
		})
	}
}

struct ChainProducer<A, B> {
	a_len: usize,
	a: A,
	b: B,
}

impl<A, B> ChainProducer<A, B> {
	const fn new(a_len: usize, a: A, b: B) -> Self {
		Self { a_len, a, b }
	}
}

impl<A: Producer, B> Producer for ChainProducer<A, B>
where
	B: Producer<Item = A::Item>,
{
	type IntoIter = ChainSeq<A::IntoIter, B::IntoIter>;
	type Item = A::Item;

	fn into_iter(self) -> Self::IntoIter {
		ChainSeq::new(self.a.into_iter(), self.b.into_iter())
	}

	fn split_at(self, index: usize) -> (Self, Self) {
		if index <= self.a_len {
			let a_rem = self.a_len - index;
			let (a_left, a_right) = self.a.split_at(index);
			let (b_left, b_right) = self.b.split_at(0);
			(
				Self::new(index, a_left, b_left),
				Self::new(a_rem, a_right, b_right),
			)
		} else {
			let (a_left, a_right) = self.a.split_at(self.a_len);
			let (b_left, b_right) = self.b.split_at(index - self.a_len);
			(
				Self::new(self.a_len, a_left, b_left),
				Self::new(0, a_right, b_right),
			)
		}
	}

	fn min_len(&self) -> usize {
		Ord::max(self.a.min_len(), self.b.min_len())
	}

	fn max_len(&self) -> usize {
		Ord::min(self.a.max_len(), self.b.max_len())
	}

	fn fold_with<F>(self, mut folder: F) -> F
	where
		F: Folder<Self::Item>,
	{
		folder = self.a.fold_with(folder);
		if folder.full() {
			folder
		} else {
			self.b.fold_with(folder)
		}
	}
}

#[repr(transparent)]
struct ChainSeq<A, B> {
	chain: iter::Chain<A, B>,
}

impl<A, B> ChainSeq<A, B> {
	fn new(a: A, b: B) -> Self
	where
		A: ExactSizeIterator,
		B: ExactSizeIterator<Item = A::Item>,
	{
		Self { chain: a.chain(b) }
	}
}

impl<A: DoubleEndedIterator, B> DoubleEndedIterator for ChainSeq<A, B>
where
	B: DoubleEndedIterator<Item = A::Item>,
{
	fn next_back(&mut self) -> Option<Self::Item> {
		self.chain.next_back()
	}
}

impl<A: ExactSizeIterator, B> ExactSizeIterator for ChainSeq<A, B> where
	B: ExactSizeIterator<Item = A::Item>
{
}

impl<A: Iterator, B> Iterator for ChainSeq<A, B>
where
	B: Iterator<Item = A::Item>,
{
	type Item = A::Item;

	fn next(&mut self) -> Option<Self::Item> {
		self.chain.next()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.chain.size_hint()
	}
}
