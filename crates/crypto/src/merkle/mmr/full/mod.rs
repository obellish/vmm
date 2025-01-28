#[cfg(feature = "serde")]
mod serde;

use alloc::vec::Vec;

use super::{
	MmrDelta, MmrError, MmrPeaks, MmrProof, bit::TrueBitPositionIterator,
	leaf_to_corresponding_tree, nodes_in_forest,
};
use crate::{
	hash::rpo::{Rpo256, RpoDigest},
	merkle::{InnerNodeInfo, MerklePath},
};

#[derive(Debug, Clone)]
pub struct Mmr {
	pub(super) forest: usize,
	pub(super) nodes: Vec<RpoDigest>,
}

impl Mmr {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			forest: 0,
			nodes: Vec::new(),
		}
	}

	#[must_use]
	pub const fn forest(&self) -> usize {
		self.forest
	}

	pub fn open(&self, pos: usize) -> Result<MmrProof, MmrError> {
		self.open_at(pos, self.forest())
	}

	pub fn open_at(&self, pos: usize, forest: usize) -> Result<MmrProof, MmrError> {
		let tree_bit =
			leaf_to_corresponding_tree(pos, forest).ok_or(MmrError::PositionNotFound(pos))?;

		let forest_before = forest & high_bitmask(tree_bit + 1);
		let index_offset = nodes_in_forest(forest_before);

		let relative_pos = pos - forest_before;

		let (_, path) = self.collect_merkle_path_and_value(tree_bit, relative_pos, index_offset);

		Ok(MmrProof {
			forest,
			position: pos,
			merkle_path: MerklePath::new(path),
		})
	}

	pub fn get(&self, pos: usize) -> Result<RpoDigest, MmrError> {
		let tree_bit =
			leaf_to_corresponding_tree(pos, self.forest).ok_or(MmrError::PositionNotFound(pos))?;

		let forest_before = self.forest() & high_bitmask(tree_bit + 1);
		let index_offset = nodes_in_forest(forest_before);

		let relative_pos = pos - forest_before;

		let (value, _) = self.collect_merkle_path_and_value(tree_bit, relative_pos, index_offset);

		Ok(value)
	}

	pub fn add(&mut self, el: RpoDigest) {
		self.nodes.push(el);

		let mut left_offset = self.nodes.len().saturating_sub(2);
		let mut right = el;
		let mut left_tree = 1;
		while !matches!(self.forest() & left_tree, 0) {
			right = Rpo256::merge(&[self.nodes[left_offset], right]);
			self.nodes.push(right);

			left_offset = left_offset.saturating_sub(nodes_in_forest(left_tree));
			left_tree <<= 1;
		}

		self.forest += 1;
	}

	#[must_use]
	pub fn peaks(&self) -> MmrPeaks {
		self.peaks_at(self.forest())
			.expect("failed to get peaks at current forest")
	}

	pub fn peaks_at(&self, forest: usize) -> Result<MmrPeaks, MmrError> {
		if forest > self.forest() {
			return Err(MmrError::InvalidPeaks(format!(
				"request forest {forest} exceeds current forest {}",
				self.forest()
			)));
		}

		let peaks = TrueBitPositionIterator::new(forest)
			.rev()
			.map(|bit| nodes_in_forest(1 << bit))
			.scan(0, |offset, el| {
				*offset += el;
				Some(*offset)
			})
			.map(|offset| self.nodes[offset - 1])
			.collect::<Vec<_>>();

		let peaks = MmrPeaks::new(forest, peaks)?;

		Ok(peaks)
	}

	pub fn get_delta(&self, from_forest: usize, to_forest: usize) -> Result<MmrDelta, MmrError> {
		if to_forest > self.forest || from_forest > to_forest {
			return Err(MmrError::InvalidPeaks(format!(
				"to_forest {to_forest} exceeds the current forest {} or from_forest {from_forest} exceeds to_forest",
				self.forest()
			)));
		}

		if from_forest == to_forest {
			return Ok(MmrDelta {
				forest: to_forest,
				data: Vec::new(),
			});
		}

		let mut result = Vec::new();

		let candidate_trees = to_forest ^ from_forest;
		let mut new_high = 1 << candidate_trees.ilog2();

		let mut merges = from_forest & (new_high - 1);

		let common_trees = from_forest ^ merges;

		if matches!(merges, 0) {
			new_high = 0;
		} else {
			let mut target = 1 << merges.trailing_zeros();

			while target < new_high {
				let known = nodes_in_forest(common_trees | merges | target);
				let sibling = nodes_in_forest(target);
				result.push(self.nodes[known + sibling - 1]);

				target <<= 1;
				while !matches!(merges & target, 0) {
					target <<= 1;
				}

				merges ^= merges & (target - 1);
			}
		}

		let mut new_peaks = to_forest ^ common_trees ^ new_high;
		let old_peaks = to_forest ^ new_peaks;
		let mut offset = nodes_in_forest(old_peaks);
		while !matches!(new_peaks, 0) {
			let target = 1 << new_peaks.ilog2();
			offset += nodes_in_forest(target);
			result.push(self.nodes[offset - 1]);
			new_peaks ^= target;
		}

		Ok(MmrDelta {
			forest: to_forest,
			data: result,
		})
	}

	#[must_use]
	pub const fn inner_nodes(&self) -> MmrNodes<'_> {
		MmrNodes {
			mmr: self,
			forest: 0,
			last_right: 0,
			index: 0,
		}
	}

	fn collect_merkle_path_and_value(
		&self,
		tree_bit: u32,
		relative_pos: usize,
		index_offset: usize,
	) -> (RpoDigest, Vec<RpoDigest>) {
		let tree_depth = (tree_bit + 1) as usize;
		let mut path = Vec::with_capacity(tree_depth);

		let mut forest_target = 1usize << tree_bit;
		let mut index = nodes_in_forest(forest_target) - 1;

		while forest_target > 1 {
			forest_target >>= 1;

			let right_offset = index - 1;
			let left_offset = right_offset - nodes_in_forest(forest_target);

			let left_or_right = relative_pos & forest_target;
			let sibling = if matches!(left_or_right, 0) {
				index = left_offset;
				self.nodes[index_offset + right_offset]
			} else {
				index = right_offset;
				self.nodes[index_offset + left_offset]
			};

			path.push(sibling);
		}

		debug_assert_eq!(path.len(), tree_depth - 1);

		path.reverse();

		let value = self.nodes[index_offset + index];
		(value, path)
	}
}

impl Default for Mmr {
	fn default() -> Self {
		Self::new()
	}
}

impl FromIterator<RpoDigest> for Mmr {
	fn from_iter<T: IntoIterator<Item = RpoDigest>>(iter: T) -> Self {
		let mut mmr = Self::new();
		for v in iter {
			mmr.add(v);
		}

		mmr
	}
}

pub struct MmrNodes<'a> {
	mmr: &'a Mmr,
	forest: usize,
	last_right: usize,
	index: usize,
}

impl Iterator for MmrNodes<'_> {
	type Item = InnerNodeInfo;

	fn next(&mut self) -> Option<Self::Item> {
		debug_assert!(
			self.last_right.count_ones() <= 1,
			"last_right tracks zero or one element"
		);

		let target = self.mmr.forest() & (usize::MAX << 1);

		if self.forest < target {
			if matches!(self.last_right, 0) {
				debug_assert_eq!(self.last_right, 0, "left must be before right");
				self.forest |= 1;
				self.index += 1;

				debug_assert_eq!(self.forest & 1, 1, "right must be after left");
				self.last_right |= 1;
				self.index += 1;
			}

			debug_assert_ne!(
				self.forest & self.last_right,
				0,
				"parent requires both a left and right"
			);

			let right_nodes = nodes_in_forest(self.last_right);

			let parent = self.last_right << 1;

			self.forest ^= self.last_right;
			if matches!(self.forest & parent, 0) {
				debug_assert_eq!(self.forest & 1, 0, "next iteration yields a left leaf");
				self.last_right = 0;
				self.forest ^= parent;
			} else {
				self.last_right = parent;
			}

			let value = self.mmr.nodes[self.index];
			let right = self.mmr.nodes[self.index - 1];
			let left = self.mmr.nodes[self.index - 1 - right_nodes];
			self.index += 1;
			let node = InnerNodeInfo { value, left, right };

			Some(node)
		} else {
			None
		}
	}
}

pub(crate) const fn high_bitmask(bit: u32) -> usize {
	if bit > usize::BITS - 1 {
		0
	} else {
		usize::MAX << bit
	}
}
