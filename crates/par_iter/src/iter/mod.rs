mod blocks;
mod chain;
mod chunks;
mod cloned;
pub mod plumbing;

use self::plumbing::{Consumer, ProducerCallback, UnindexedConsumer};
pub use self::{
	blocks::{ExponentialBlocks, UniformBlocks},
	chain::Chain,
	chunks::Chunks,
};

pub trait ParallelIterator: Send + Sized {
	type Item: Send;

	fn drive_unindexed<C>(self, consumer: C) -> C::Result
	where
		C: UnindexedConsumer<Self::Item>;

	fn opt_len(&self) -> Option<usize> {
		None
	}

	fn chain<C>(self, chain: C) -> Chain<Self, C::Iter>
	where
		C: IntoParallelIterator<Item = Self::Item>,
	{
		Chain::new(self, chain.into_par_iter())
	}
}

#[allow(clippy::len_without_is_empty)]
pub trait IndexedParallelIterator: ParallelIterator {
	fn len(&self) -> usize;

	fn drive<C>(self, consumer: C) -> C::Result
	where
		C: Consumer<Self::Item>;

	fn with_producer<CB>(self, callback: CB) -> CB::Output
	where
		CB: ProducerCallback<Self::Item>;

	#[track_caller]
	fn by_uniform_blocks(self, block_size: usize) -> UniformBlocks<Self> {
		assert_ne!(block_size, 0, "block_size must not be zero");
		UniformBlocks::new(self, block_size)
	}

	#[track_caller]
	fn chunks(self, chunk_size: usize) -> Chunks<Self> {
		assert_ne!(chunk_size, 0, "chunk_size must not be zero");
		Chunks::new(self, chunk_size)
	}
}

pub trait IntoParallelIterator {
	type Iter: ParallelIterator<Item = Self::Item>;
	type Item: Send;

	fn into_par_iter(self) -> Self::Iter;
}
