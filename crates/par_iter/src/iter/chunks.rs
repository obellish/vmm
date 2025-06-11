use super::{
	IndexedParallelIterator, ParallelIterator,
	plumbing::{Consumer, Producer, ProducerCallback, UnindexedConsumer, bridge},
};
use crate::math::div_round_up;

#[derive(Debug, Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Chunks<I> {
	size: usize,
	i: I,
}

impl<I> Chunks<I> {
	pub(super) const fn new(i: I, size: usize) -> Self {
		Self { size, i }
	}
}

impl<I: IndexedParallelIterator> ParallelIterator for Chunks<I> {
	type Item = Vec<I::Item>;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>,
	{
		bridge(self, consumer)
	}
}

impl<I: IndexedParallelIterator> IndexedParallelIterator for Chunks<I> {
	fn drive<C>(self, consumer: C) -> C::Result
	where
		C: Consumer<Self::Item>,
	{
		bridge(self, consumer)
	}

	fn len(&self) -> usize {
		div_round_up(self.i.len(), self.size)
	}

	fn with_producer<CB>(self, callback: CB) -> CB::Output
	where
		CB: ProducerCallback<Self::Item>,
	{
		#[allow(clippy::struct_field_names)]
		struct Callback<CB> {
			size: usize,
			len: usize,
			callback: CB,
		}

		impl<T, CB> ProducerCallback<T> for Callback<CB>
		where
			CB: ProducerCallback<Vec<T>>,
		{
			type Output = CB::Output;

			fn callback<P>(self, producer: P) -> Self::Output
			where
				P: Producer<Item = T>,
			{
				let producer = ChunkProducer::new(self.size, self.len, producer, Vec::from_iter);
				self.callback.callback(producer)
			}
		}

		let len = self.i.len();
		self.i.with_producer(Callback {
			size: self.size,
			len,
			callback,
		})
	}
}

pub(super) struct ChunkProducer<P, F> {
	chunk_size: usize,
	len: usize,
	base: P,
	map: F,
}

impl<P, F> ChunkProducer<P, F> {
	pub(super) const fn new(chunk_size: usize, len: usize, base: P, map: F) -> Self {
		Self {
			chunk_size,
			len,
			base,
			map,
		}
	}
}

impl<P: Producer, F, T> Producer for ChunkProducer<P, F>
where
	F: Fn(P::IntoIter) -> T + Send + Clone,
{
	type IntoIter = std::iter::Map<ChunkSeq<P>, F>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		let chunks = ChunkSeq {
			chunk_size: self.chunk_size,
			len: self.len,
			inner: if self.len > 0 { Some(self.base) } else { None },
		};

		chunks.map(self.map)
	}

	fn split_at(self, index: usize) -> (Self, Self) {
		let elem_index = Ord::min(index * self.chunk_size, self.len);
		let (left, right) = self.base.split_at(elem_index);
		(
			Self {
				chunk_size: self.chunk_size,
				len: elem_index,
				base: left,
				map: self.map.clone(),
			},
			Self {
				chunk_size: self.chunk_size,
				len: self.len - elem_index,
				base: right,
				map: self.map,
			},
		)
	}

	fn min_len(&self) -> usize {
		div_round_up(self.base.min_len(), self.chunk_size)
	}

	fn max_len(&self) -> usize {
		self.base.max_len() / self.chunk_size
	}
}

pub(super) struct ChunkSeq<P> {
	chunk_size: usize,
	len: usize,
	inner: Option<P>,
}

impl<P: Producer> DoubleEndedIterator for ChunkSeq<P> {
	fn next_back(&mut self) -> Option<Self::Item> {
		let producer = self.inner.take()?;
		if self.len > self.chunk_size {
			let mut size = self.len % self.chunk_size;
			if matches!(size, 0) {
				size = self.chunk_size;
			}
			let (left, right) = producer.split_at(self.len - size);
			self.inner = Some(left);
			self.len -= size;
			Some(right.into_iter())
		} else {
			debug_assert!(self.len > 0);
			self.len = 0;
			Some(producer.into_iter())
		}
	}
}

impl<P: Producer> ExactSizeIterator for ChunkSeq<P> {}

impl<P: Producer> Iterator for ChunkSeq<P> {
	type Item = P::IntoIter;

	fn next(&mut self) -> Option<Self::Item> {
		let producer = self.inner.take()?;
		if self.len > self.chunk_size {
			let (left, right) = producer.split_at(self.chunk_size);
			self.inner = Some(right);
			self.len -= self.chunk_size;
			Some(left.into_iter())
		} else {
			debug_assert!(self.len > 0);
			self.len = 0;
			Some(producer.into_iter())
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = div_round_up(self.len, self.chunk_size);
		(len, Some(len))
	}
}
