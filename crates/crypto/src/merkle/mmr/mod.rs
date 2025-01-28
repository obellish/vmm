mod bit;
mod delta;
mod error;
mod full;
mod in_order;
mod partial;
mod peaks;
mod proof;

pub use self::{
	delta::MmrDelta,
	error::MmrError,
	full::{Mmr, MmrNodes},
	in_order::InOrderIndex,
	partial::{InnerNodeIterator, PartialMmr},
	peaks::MmrPeaks,
	proof::MmrProof,
};

const fn leaf_to_corresponding_tree(pos: usize, forest: usize) -> Option<u32> {
	if pos >= forest {
		None
	} else {
		let before = forest & pos;
		let after = forest ^ before;
		let tree = after.ilog2();

		Some(tree)
	}
}

const fn nodes_in_forest(forest: usize) -> usize {
	let tree_count = forest.count_ones() as usize;
	forest * 2 - tree_count
}
