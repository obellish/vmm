mod empty_roots;
mod error;
mod index;
mod mmr;
mod node;
mod partial_mt;
mod path;
mod smt;
mod store;
mod tree;

pub use self::{
	empty_roots::EmptySubtreeRoots,
	error::MerkleError,
	index::NodeIndex,
	mmr::{
		InOrderIndex, InnerNodeIterator as PartialMmrInnerNodeIterator, Mmr, MmrDelta, MmrError,
		MmrNodes, MmrPeaks, MmrProof, PartialMmr,
	},
	node::InnerNodeInfo,
	partial_mt::PartialMerkleTree,
	path::{InnerNodeIterator, MerklePath, RootPath, ValuePath},
	smt::{
		InnerNode, LeafIndex, MutationSet, NodeMutation, SMT_MAX_DEPTH, SMT_MIN_DEPTH, SimpleSmt,
	},
	store::{DefaultMerkleStore, MerkleStore, RecordingMerkleStore, StoreNode},
	tree::{
		InnerNodeIterator as MerkleTreeInnerNodeIterator, MerkleTree, path_to_text, tree_to_text,
	},
};

#[cfg(test)]
const fn int_to_node(value: u64) -> crate::hash::rpo::RpoDigest {
	crate::hash::rpo::RpoDigest::new(int_to_leaf(value))
}

#[cfg(test)]
const fn int_to_leaf(value: u64) -> crate::Word {
	[
		crate::Felt::new(value),
		crate::ZERO,
		crate::ZERO,
		crate::ZERO,
	]
}

#[cfg(test)]
fn digests_to_words(digests: &[crate::hash::rpo::RpoDigest]) -> alloc::vec::Vec<crate::Word> {
	digests.iter().map(Into::into).collect()
}
