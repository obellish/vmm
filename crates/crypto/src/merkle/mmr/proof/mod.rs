#[cfg(feature = "serde")]
mod serde;

use super::{full::high_bitmask, leaf_to_corresponding_tree};
use crate::merkle::MerklePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmrProof {
	pub forest: usize,
	pub position: usize,
	pub merkle_path: MerklePath,
}

impl MmrProof {
	#[must_use]
	pub const fn relative_pos(&self) -> usize {
		let tree_bit = leaf_to_corresponding_tree(self.position, self.forest)
			.expect("position must be a part of the forest");
		let forest_before = self.forest & high_bitmask(tree_bit + 1);
		self.position - forest_before
	}

	#[must_use]
	pub const fn peak_index(&self) -> usize {
		let root = leaf_to_corresponding_tree(self.position, self.forest)
			.expect("position must be a part of the forest");
		let smaller_peak_mask = 2usize.pow(root) as usize - 1;
		let num_smaller_peaks = (self.forest & smaller_peak_mask).count_ones();
		(self.forest.count_ones() - num_smaller_peaks - 1) as usize
	}
}

#[cfg(test)]
mod tests {
	use super::{MerklePath, MmrProof};

	fn make_dummy_proof(forest: usize, position: usize) -> MmrProof {
		MmrProof {
			forest,
			position,
			merkle_path: MerklePath::default(),
		}
	}

	#[test]
	fn peak_index() {
		let forest = 11;

		for position in 0..8 {
			let proof = make_dummy_proof(forest, position);
			assert_eq!(proof.peak_index(), 0);
		}

		let forest = 11;

		for position in 0..8 {
			let proof = make_dummy_proof(forest, position);
			assert_eq!(proof.peak_index(), 0);
		}

		for position in 8..10 {
			let proof = make_dummy_proof(forest, position);
			assert_eq!(proof.peak_index(), 1);
		}

		let proof = make_dummy_proof(forest, 10);
		assert_eq!(proof.peak_index(), 2);

		let forest = 7;

		for position in 0..4 {
			let proof = make_dummy_proof(forest, position);
			assert_eq!(proof.peak_index(), 0);
		}

		for position in 4..6 {
			let proof = make_dummy_proof(forest, position);
			assert_eq!(proof.peak_index(), 1);
		}

		let proof = make_dummy_proof(forest, 6);
		assert_eq!(proof.peak_index(), 2);
	}
}
