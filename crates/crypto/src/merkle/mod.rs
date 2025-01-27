mod error;
mod index;
mod mmr;
mod node;
mod path;

pub use self::{
	error::MerkleError,
	index::NodeIndex,
	mmr::{InOrderIndex, MmrDelta, MmrError, MmrProof},
	node::InnerNodeInfo,
	path::{InnerNodeIterator, MerklePath, RootPath, ValuePath},
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
