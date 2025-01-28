use alloc::{
	collections::{BTreeMap, BTreeSet},
	vec::Vec,
};

use super::{MmrDelta, MmrProof};
use crate::{
	hash::rpo::{Rpo256, RpoDigest},
	merkle::{
		InOrderIndex, InnerNodeInfo, MerklePath, MmrError, MmrPeaks,
		mmr::{leaf_to_corresponding_tree, nodes_in_forest},
	},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialMmr {
	pub(crate) forest: usize,
	pub(crate) peaks: Vec<RpoDigest>,
	pub(crate) nodes: NodeMap,
	pub(crate) track_latest: bool,
}

impl PartialMmr {
	#[must_use]
	pub fn from_peaks(peaks: MmrPeaks) -> Self {
		Self::from_parts(peaks, BTreeMap::new(), false)
	}

	#[must_use]
	pub fn from_parts(peaks: MmrPeaks, nodes: NodeMap, track_latest: bool) -> Self {
		let forest = peaks.num_leaves();
		let peaks = peaks.into_iter().collect();

		Self {
			forest,
			peaks,
			nodes,
			track_latest,
		}
	}

	#[must_use]
	pub const fn forest(&self) -> usize {
		self.forest
	}

	#[must_use]
	pub const fn num_leaves(&self) -> usize {
		self.forest()
	}

	#[must_use]
	pub fn peaks(&self) -> MmrPeaks {
		MmrPeaks::new(self.forest, self.peaks.clone()).expect("invalid mmr peaks")
	}

	#[must_use]
	pub fn is_tracked(&self, pos: usize) -> bool {
		if pos >= self.forest() {
			return false;
		} else if pos == self.forest() - 1 && !matches!(self.forest() & 1, 0) {
			return self.track_latest;
		}

		let leaf_index = InOrderIndex::from_leaf_pos(pos);
		self.is_tracked_node(leaf_index)
	}

	pub fn open(&self, pos: usize) -> Result<Option<MmrProof>, MmrError> {
		let tree_bit = leaf_to_corresponding_tree(pos, self.forest())
			.ok_or(MmrError::PositionNotFound(pos))?;
		let depth = tree_bit as usize;

		let mut nodes = Vec::with_capacity(depth);
		let mut idx = InOrderIndex::from_leaf_pos(pos);

		while let Some(node) = self.nodes.get(&idx.sibling()) {
			nodes.push(*node);
			idx = idx.parent();
		}

		debug_assert!(nodes.is_empty() || nodes.len() == depth);

		if nodes.len() == depth {
			Ok(Some(MmrProof {
				forest: self.forest(),
				position: pos,
				merkle_path: MerklePath::new(nodes),
			}))
		} else {
			Ok(None)
		}
	}

	pub fn nodes(&self) -> impl Iterator<Item = (&InOrderIndex, &RpoDigest)> {
		self.nodes.iter()
	}

	pub fn inner_nodes<I>(&self, mut leaves: I) -> InnerNodeIterator<'_, I>
	where
		I: Iterator<Item = (usize, RpoDigest)>,
	{
		let stack = if let Some((pos, leaf)) = leaves.next() {
			let idx = InOrderIndex::from_leaf_pos(pos);
			vec![(idx, leaf)]
		} else {
			Vec::new()
		};

		InnerNodeIterator {
			nodes: &self.nodes,
			leaves,
			stack,
			seen_nodes: BTreeSet::new(),
		}
	}

	pub fn add(&mut self, leaf: RpoDigest, track: bool) -> Vec<(InOrderIndex, RpoDigest)> {
		self.forest += 1;
		let merges = self.forest.trailing_zeros() as usize;
		let mut new_nodes = Vec::with_capacity(merges);

		let peak = if matches!(merges, 0) {
			self.track_latest = track;
			leaf
		} else {
			let mut track_right = track;
			let mut track_left = self.track_latest;

			let mut right = leaf;
			let mut right_idx = forest_to_rightmost_index(self.forest);

			for _ in 0..merges {
				let left = self.peaks.pop().expect("missing peak");
				let left_idx = right_idx.sibling();

				if track_right {
					let old = self.nodes.insert(left_idx, left);
					new_nodes.push((left_idx, left));

					debug_assert!(
						old.is_none(),
						"idx {left_idx:?} already contained an element {old:?}"
					);
				}

				if track_left {
					let old = self.nodes.insert(right_idx, right);
					new_nodes.push((right_idx, right));

					debug_assert!(
						old.is_none(),
						"idx {right_idx:?} already contained an element {old:?}"
					);
				}

				right_idx = right_idx.parent();

				right = Rpo256::merge(&[left, right]);

				track_right = track_right || track_left;

				track_left = self.is_tracked_node(right_idx.sibling());
			}

			right
		};

		self.peaks.push(peak);

		new_nodes
	}

	pub fn track(
		&mut self,
		leaf_pos: usize,
		leaf: RpoDigest,
		path: &MerklePath,
	) -> Result<(), MmrError> {
		let tree = 1 << path.depth();
		if matches!(tree & self.forest(), 0) {
			return Err(MmrError::UnknownPeak(path.depth()));
		}

		if leaf_pos + 1 == self.forest()
			&& matches!(path.depth(), 0)
			&& self.peaks.last().is_some_and(|v| *v == leaf)
		{
			self.track_latest = true;
			return Ok(());
		}

		let target_forest = self.forest() ^ (self.forest() & (tree - 1));
		let peak_pos = (target_forest.count_ones() - 1) as usize;

		let path_idx = leaf_pos - (target_forest ^ tree);

		let computed = path
			.compute_root(path_idx as u64, leaf)
			.map_err(MmrError::MerkleRootComputationFailed)?;
		if self.peaks[peak_pos] != computed {
			return Err(MmrError::PeakPathMismatch);
		}

		let mut idx = InOrderIndex::from_leaf_pos(leaf_pos);
		for leaf in path.nodes() {
			self.nodes.insert(idx.sibling(), *leaf);
			idx = idx.parent();
		}

		Ok(())
	}

	pub fn untrack(&mut self, leaf_pos: usize) {
		let mut idx = InOrderIndex::from_leaf_pos(leaf_pos);

		self.nodes.remove(&idx.sibling());

		while !self.nodes.contains_key(&idx) {
			idx = idx.parent();
			self.nodes.remove(&idx.sibling());
		}
	}

	#[allow(clippy::tuple_array_conversions)]
	pub fn apply(&mut self, delta: MmrDelta) -> Result<Vec<(InOrderIndex, RpoDigest)>, MmrError> {
		if delta.forest < self.forest() {
			return Err(MmrError::InvalidPeaks(format!(
				"forest of mmr delta {} is less than current forest {}",
				delta.forest,
				self.forest()
			)));
		}

		let mut inserted_nodes = Vec::new();

		if delta.forest == self.forest() {
			if !delta.data.is_empty() {
				return Err(MmrError::InvalidUpdate);
			}

			return Ok(inserted_nodes);
		}

		let changes = self.forest() ^ delta.forest;
		let largest = 1 << changes.ilog2();
		let merges = self.forest() & (largest - 1);

		debug_assert!(
			!self.track_latest || matches!(merges & 1, 1),
			"if there is an odd element, a merge is required"
		);

		let (merge_count, new_peaks) = if matches!(merges, 0) {
			(0, changes)
		} else {
			let depth = largest.trailing_zeros();
			let skipped = merges.trailing_zeros();
			let computed = merges.count_ones() - 1;
			let merge_count = depth - skipped - computed;

			let new_peaks = delta.forest & (largest - 1);

			(merge_count, new_peaks)
		};

		if (delta.data.len() as u32) != merge_count + new_peaks.count_ones() {
			return Err(MmrError::InvalidUpdate);
		}

		let mut update_count = 0;

		if !matches!(merges, 0) {
			let mut peak_idx = forest_to_root_index(self.forest);

			self.peaks.reverse();

			let mut track = self.track_latest;
			self.track_latest = false;

			let mut peak_count = 0;
			let mut target = 1 << merges.trailing_zeros();
			let mut new = delta.data[0];
			update_count += 1;

			while target < largest {
				if !matches!(target, 1) && !track {
					track = self.is_tracked_node(peak_idx);
				}

				let (left, right) = if matches!(target & merges, 0) {
					let update = delta.data[update_count];
					update_count += 1;
					(new, update)
				} else {
					let peak = self.peaks[peak_count];
					let sibling_idx = peak_idx.sibling();

					if self.is_tracked_node(sibling_idx) {
						self.nodes.insert(peak_idx, new);
						inserted_nodes.push((peak_idx, new));
					}
					peak_count += 1;
					(peak, new)
				};

				if track {
					let sibling_idx = peak_idx.sibling();
					if peak_idx.is_left_child() {
						self.nodes.insert(sibling_idx, right);
						inserted_nodes.push((sibling_idx, right));
					} else {
						self.nodes.insert(sibling_idx, left);
						inserted_nodes.push((sibling_idx, left));
					}
				}

				peak_idx = peak_idx.parent();
				new = Rpo256::merge(&[left, right]);
				target <<= 1;
			}

			debug_assert_eq!(peak_count, merges.count_ones() as usize);

			self.peaks.reverse();
			self.peaks.truncate(self.peaks.len() - peak_count);
			self.peaks.push(new);
		}

		self.peaks.extend_from_slice(&delta.data[update_count..]);
		self.forest = delta.forest;

		debug_assert_eq!(self.peaks.len(), self.forest().count_ones() as usize);

		Ok(inserted_nodes)
	}

	fn is_tracked_node(&self, node_index: InOrderIndex) -> bool {
		if node_index.is_leaf() {
			self.nodes.contains_key(&node_index.sibling())
		} else {
			let left_child = node_index.left_child();
			let right_child = node_index.right_child();
			self.nodes.contains_key(&left_child) | self.nodes.contains_key(&right_child)
		}
	}
}

impl Deserializable for PartialMmr {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let forest = usize::read_from(source)?;
		let peaks = Vec::<RpoDigest>::read_from(source)?;
		let nodes = NodeMap::read_from(source)?;
		let track_latest = source.read_bool()?;

		Ok(Self {
			forest,
			peaks,
			nodes,
			track_latest,
		})
	}
}

impl From<MmrPeaks> for PartialMmr {
	fn from(value: MmrPeaks) -> Self {
		Self::from_peaks(value)
	}
}

impl Serializable for PartialMmr {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.forest.write_into(target);
		self.peaks.write_into(target);
		self.nodes.write_into(target);
		target.write_bool(self.track_latest);
	}
}

pub struct InnerNodeIterator<'a, I>
where
	I: Iterator<Item = (usize, RpoDigest)>,
{
	nodes: &'a NodeMap,
	leaves: I,
	stack: Vec<(InOrderIndex, RpoDigest)>,
	seen_nodes: BTreeSet<InOrderIndex>,
}

impl<I> Iterator for InnerNodeIterator<'_, I>
where
	I: Iterator<Item = (usize, RpoDigest)>,
{
	type Item = InnerNodeInfo;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((idx, node)) = self.stack.pop() {
			let parent_idx = idx.parent();
			let new_node = self.seen_nodes.insert(parent_idx);

			if new_node {
				if let Some(sibling) = self.nodes.get(&idx.sibling()) {
					let (left, right) = if parent_idx.left_child() == idx {
						(node, *sibling)
					} else {
						(*sibling, node)
					};
					let parent = Rpo256::merge(&[left, right]);
					let inner_node = InnerNodeInfo {
						value: parent,
						left,
						right,
					};

					self.stack.push((parent_idx, parent));
					return Some(inner_node);
				}
			}

			if let Some((pos, leaf)) = self.leaves.next() {
				let idx = InOrderIndex::from_leaf_pos(pos);
				self.stack.push((idx, leaf));
			}
		}

		None
	}
}

type NodeMap = BTreeMap<InOrderIndex, RpoDigest>;

fn forest_to_root_index(forest: usize) -> InOrderIndex {
	let nodes = nodes_in_forest(forest);
	let open_trees = (forest.count_ones() - 1) as usize;
	let right_subtree_count = ((1u32 << forest.trailing_zeros()) - 1) as usize;
	let idx = nodes + open_trees - right_subtree_count;

	InOrderIndex::new(idx.try_into().unwrap())
}

fn forest_to_rightmost_index(forest: usize) -> InOrderIndex {
	let nodes = nodes_in_forest(forest);
	let open_trees = (forest.count_ones() - 1) as usize;
	let idx = nodes + open_trees;

	InOrderIndex::new(idx.try_into().unwrap())
}

#[cfg(test)]
mod tests {
	use alloc::{collections::BTreeSet, vec::Vec};

	use num::bigint::ParseBigIntError;

	use super::{
		InOrderIndex, MmrPeaks, PartialMmr, RpoDigest, forest_to_rightmost_index,
		forest_to_root_index,
	};
	use crate::{
		merkle::{Mmr, MmrError, NodeIndex, int_to_node},
		utils::{Deserializable, Serializable},
	};

	const LEAVES: [RpoDigest; 7] = [
		int_to_node(0),
		int_to_node(1),
		int_to_node(2),
		int_to_node(3),
		int_to_node(4),
		int_to_node(5),
		int_to_node(6),
	];

	fn idx(pos: usize) -> InOrderIndex {
		InOrderIndex::new(pos.try_into().unwrap())
	}

	fn validate_apply_delta(mmr: &Mmr, partial: &mut PartialMmr) -> Result<(), MmrError> {
		let tracked_leaves = partial
			.nodes()
			.filter_map(|(index, _)| {
				if index.is_leaf() {
					Some(index.sibling())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		let nodes_before = partial.nodes.clone();

		let delta = mmr.get_delta(partial.forest(), mmr.forest())?;
		let nodes_delta = partial.apply(delta)?;

		assert_eq!(mmr.peaks(), partial.peaks());

		let mut expected_nodes = nodes_before;
		for (key, value) in nodes_delta {
			assert!(expected_nodes.insert(key, value).is_none());
		}

		assert_eq!(expected_nodes, partial.nodes);

		for index in tracked_leaves {
			let index_value: u64 = index.into();
			let pos = index_value / 2;
			let proof1 = partial.open(pos as usize)?.unwrap();
			let proof2 = mmr.open(pos as usize)?;
			assert_eq!(proof1, proof2);
		}

		Ok(())
	}

	#[test]
	fn forest_to_root_index_test() {
		assert_eq!(forest_to_root_index(0b0001), idx(1));
		assert_eq!(forest_to_root_index(0b0010), idx(2));
		assert_eq!(forest_to_root_index(0b0100), idx(4));
		assert_eq!(forest_to_root_index(0b1000), idx(8));

		assert_eq!(forest_to_root_index(0b0011), idx(5));
		assert_eq!(forest_to_root_index(0b0101), idx(9));
		assert_eq!(forest_to_root_index(0b1001), idx(17));
		assert_eq!(forest_to_root_index(0b0111), idx(13));
		assert_eq!(forest_to_root_index(0b1011), idx(21));
		assert_eq!(forest_to_root_index(0b1111), idx(29));

		assert_eq!(forest_to_root_index(0b0110), idx(10));
		assert_eq!(forest_to_root_index(0b1010), idx(18));
		assert_eq!(forest_to_root_index(0b1100), idx(20));
		assert_eq!(forest_to_root_index(0b1110), idx(26));
	}

	#[test]
	fn forest_to_rightmost_index_test() {
		for forest in 1..256 {
			assert_eq!(
				forest_to_rightmost_index(forest).inner() % 2,
				1,
				"leaves are always odd"
			);
		}

		assert_eq!(forest_to_rightmost_index(0b0001), idx(1));
		assert_eq!(forest_to_rightmost_index(0b0010), idx(3));
		assert_eq!(forest_to_rightmost_index(0b0011), idx(5));
		assert_eq!(forest_to_rightmost_index(0b0100), idx(7));
		assert_eq!(forest_to_rightmost_index(0b0101), idx(9));
		assert_eq!(forest_to_rightmost_index(0b0110), idx(11));
		assert_eq!(forest_to_rightmost_index(0b0111), idx(13));
		assert_eq!(forest_to_rightmost_index(0b1000), idx(15));
		assert_eq!(forest_to_rightmost_index(0b1001), idx(17));
		assert_eq!(forest_to_rightmost_index(0b1010), idx(19));
		assert_eq!(forest_to_rightmost_index(0b1011), idx(21));
		assert_eq!(forest_to_rightmost_index(0b1100), idx(23));
		assert_eq!(forest_to_rightmost_index(0b1101), idx(25));
		assert_eq!(forest_to_rightmost_index(0b1110), idx(27));
		assert_eq!(forest_to_rightmost_index(0b1111), idx(29));
	}

	#[test]
	fn partial_mmr_apply_delta() -> Result<(), MmrError> {
		let mut mmr = Mmr::default();
		(0..10).for_each(|i| mmr.add(int_to_node(i)));
		let mut partial_mmr: PartialMmr = mmr.peaks().into();

		{
			let node = mmr.get(1)?;
			let proof = mmr.open(1)?;
			partial_mmr.track(1, node, &proof.merkle_path)?;
		}

		{
			let node = mmr.get(8)?;
			let proof = mmr.open(8)?;
			partial_mmr.track(8, node, &proof.merkle_path)?;
		}

		(10..12).for_each(|i| mmr.add(int_to_node(i)));
		validate_apply_delta(&mmr, &mut partial_mmr)?;

		mmr.add(int_to_node(12));
		validate_apply_delta(&mmr, &mut partial_mmr)?;
		{
			let node = mmr.get(12)?;
			let proof = mmr.open(12)?;
			partial_mmr.track(12, node, &proof.merkle_path)?;
			assert!(partial_mmr.track_latest);
		}

		(13..16).for_each(|i| mmr.add(int_to_node(i)));
		validate_apply_delta(&mmr, &mut partial_mmr)
	}

	#[test]
	fn partial_mmr_inner_nodes_iterator() -> Result<(), MmrError> {
		let mmr: Mmr = LEAVES.into_iter().collect();
		let first_peak = mmr.peaks().peaks()[0];

		let node1 = mmr.get(1)?;
		let proof1 = mmr.open(1)?;

		let mut partial_mmr: PartialMmr = mmr.peaks().into();
		partial_mmr.track(1, node1, &proof1.merkle_path)?;

		assert_eq!(partial_mmr.inner_nodes(core::iter::empty()).next(), None);

		todo!();

		Ok(())
	}
}
