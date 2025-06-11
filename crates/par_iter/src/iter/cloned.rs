use std::iter;

use super::{plumbing::*, *};

#[derive(Debug, Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[repr(transparent)]
pub struct Cloned<I> {
	base: I,
}

impl<I> Cloned<I> {
	pub(super) const fn new(base: I) -> Self {
		Self { base }
	}
}

impl<'a, T, I> ParallelIterator for Cloned<I>
where
	I: ParallelIterator<Item = &'a T>,
	T: Clone + Send + Sync + 'a,
{
	type Item = T;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		todo!()
	}
}

#[repr(transparent)]
struct ClonedProducer<P> {
	base: P,
}

impl<'a, T, P> Producer for ClonedProducer<P>
where
	P: Producer<Item = &'a T>,
	T: Clone + 'a,
{
	type IntoIter = iter::Cloned<P::IntoIter>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		self.base.into_iter().cloned()
	}

	fn split_at(self, index: usize) -> (Self, Self) {
		let (left, right) = self.base.split_at(index);
		(Self { base: left }, Self { base: right })
	}

	fn min_len(&self) -> usize {
		self.base.min_len()
	}

	fn max_len(&self) -> usize {
		self.base.max_len()
	}

	fn fold_with<F>(self, folder: F) -> F
	where
		F: Folder<Self::Item>,
	{
		self.base.fold_with(ClonedFolder { base: folder }).base
	}
}

#[repr(transparent)]
struct ClonedFolder<F> {
	base: F,
}

impl<'a, T, F> Folder<&'a T> for ClonedFolder<F>
where
	F: Folder<T>,
	T: Clone + 'a,
{
	type Result = F::Result;

	fn consume(self, item: &'a T) -> Self {
		Self {
			base: self.base.consume(item.clone()),
		}
	}

	fn consume_iter(mut self, iter: impl IntoIterator<Item = &'a T>) -> Self {
		self.base = self.base.consume_iter(iter.into_iter().cloned());
		self
	}

	fn complete(self) -> Self::Result {
		self.base.complete()
	}

	fn full(&self) -> bool {
		self.base.full()
	}
}

#[repr(transparent)]
struct ClonedConsumer<C> {
	base: C,
}

impl<C> ClonedConsumer<C> {
	const fn new(base: C) -> Self {
		Self { base }
	}
}

impl<'a, T, C> Consumer<&'a T> for ClonedConsumer<C>
where
	C: Consumer<T>,
	T: Clone + 'a,
{
	type Folder = ClonedFolder<C::Folder>;
	type Reducer = C::Reducer;
	type Result = C::Result;

	fn split_at(self, index: usize) -> (Self, Self, Self::Reducer) {
		let (left, right, reducer) = self.base.split_at(index);
		(Self::new(left), Self::new(right), reducer)
	}

	fn into_folder(self) -> Self::Folder {
		ClonedFolder {
			base: self.base.into_folder(),
		}
	}

	fn full(&self) -> bool {
		self.base.full()
	}
}
