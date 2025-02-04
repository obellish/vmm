mod merger;
mod multi_forest_node_iterator;
mod node;
mod node_fingerprint;
mod serialization;

use alloc::{
	collections::{BTreeMap, BTreeSet},
	vec::Vec,
};
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	mem,
	ops::{Index, IndexMut},
};

use thiserror::Error;

pub(crate) use self::{merger::MastForestMerger, multi_forest_node_iterator::*};
pub use self::{
	merger::MastForestRootMap,
	node::{
		BasicBlockNode, CallNode, DynNode, ExternalNode, JoinNode, LoopNode, MastNode,
		OP_BATCH_SIZE, OP_GROUP_SIZE, OpBatch, OperationOrDecorator, SplitNode,
	},
	node_fingerprint::{DecoratorFingerprint, MastNodeFingerprint},
};
use crate::{
	AdviceMap, Decorator, DecoratorList, Operation,
	crypto::hash::RpoDigest,
	utils::{ByteWriter, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MastForest {
	nodes: Vec<MastNode>,
	roots: Vec<MastNodeId>,
	decorators: Vec<Decorator>,
	advice_map: AdviceMap,
}

impl MastForest {
	const MAX_NODES: usize = (1 << 30) - 1;

	#[must_use]
	pub const fn new() -> Self {
		Self {
			nodes: Vec::new(),
			roots: Vec::new(),
			decorators: Vec::new(),
			advice_map: AdviceMap::new(),
		}
	}

	pub fn add_decorator(&mut self, decorator: Decorator) -> Result<DecoratorId, MastForestError> {
		if self.decorators().len() >= u32::MAX as usize {
			return Err(MastForestError::TooManyDecorators);
		}

		let new_decorator_id = DecoratorId(self.decorators().len() as u32);
		self.decorators.push(decorator);

		Ok(new_decorator_id)
	}

	pub fn add_node(&mut self, node: MastNode) -> Result<MastNodeId, MastForestError> {
		if self.nodes().len() == Self::MAX_NODES {
			return Err(MastForestError::TooManyNodes);
		}

		let new_node_id = MastNodeId(self.nodes().len() as u32);
		self.nodes.push(node);

		Ok(new_node_id)
	}

	pub fn add_block(
		&mut self,
		operations: Vec<Operation>,
		decorators: Option<DecoratorList>,
	) -> Result<MastNodeId, MastForestError> {
		let block = MastNode::basic_block(operations, decorators)?;
		self.add_node(block)
	}

	pub fn add_join(
		&mut self,
		left_child: MastNodeId,
		right_child: MastNodeId,
	) -> Result<MastNodeId, MastForestError> {
		let join = MastNode::join(left_child, right_child, self)?;
		self.add_node(join)
	}

	pub fn add_split(
		&mut self,
		if_branch: MastNodeId,
		else_branch: MastNodeId,
	) -> Result<MastNodeId, MastForestError> {
		let split = MastNode::split(if_branch, else_branch, self)?;
		self.add_node(split)
	}

	pub fn add_loop(&mut self, body: MastNodeId) -> Result<MastNodeId, MastForestError> {
		let loop_node = MastNode::r#loop(body, self)?;
		self.add_node(loop_node)
	}

	pub fn add_call(&mut self, callee: MastNodeId) -> Result<MastNodeId, MastForestError> {
		let call = MastNode::call(callee, self)?;
		self.add_node(call)
	}

	pub fn add_syscall(&mut self, callee: MastNodeId) -> Result<MastNodeId, MastForestError> {
		let call = MastNode::syscall(callee, self)?;
		self.add_node(call)
	}

	pub fn add_dyn(&mut self) -> Result<MastNodeId, MastForestError> {
		self.add_node(MastNode::r#dyn())
	}

	pub fn add_dyncall(&mut self) -> Result<MastNodeId, MastForestError> {
		self.add_node(MastNode::dyncall())
	}

	pub fn add_external(&mut self, mast_root: RpoDigest) -> Result<MastNodeId, MastForestError> {
		self.add_node(MastNode::external(mast_root))
	}

	pub fn make_root(&mut self, new_root_id: MastNodeId) {
		assert!((new_root_id.0 as usize) < self.nodes.len());

		if !self.procedure_roots().contains(&new_root_id) {
			self.roots.push(new_root_id);
		}
	}

	pub fn remove_nodes(
		&mut self,
		nodes_to_remove: &BTreeSet<MastNodeId>,
	) -> Option<BTreeMap<MastNodeId, MastNodeId>> {
		if nodes_to_remove.is_empty() {
			None
		} else {
			let old_nodes = mem::take(&mut self.nodes);
			let old_root_ids = mem::take(&mut self.roots);
			let (retained_nodes, id_remappings) = remove_nodes(old_nodes, nodes_to_remove);

			self.remap_and_add_nodes(retained_nodes, &id_remappings);
			self.remap_and_add_roots(old_root_ids, &id_remappings);

			Some(id_remappings)
		}
	}

	pub fn set_before_enter(
		&mut self,
		node_id: MastNodeId,
		decorator_ids: impl IntoIterator<Item = DecoratorId>,
	) {
		self[node_id].set_before_enter(decorator_ids);
	}

	pub fn set_after_exit(
		&mut self,
		node_id: MastNodeId,
		decorator_ids: impl IntoIterator<Item = DecoratorId>,
	) {
		self[node_id].set_after_exit(decorator_ids);
	}

	pub fn merge<'forest>(
		forests: impl IntoIterator<Item = &'forest Self>,
	) -> Result<(Self, MastForestRootMap), MastForestError> {
		MastForestMerger::merge(forests)
	}

	#[cfg(test)]
	pub fn add_block_with_raw_decorators(
		&mut self,
		operations: impl IntoIterator<Item = Operation>,
		decorators: impl IntoIterator<Item = (usize, Decorator)>,
	) -> Result<MastNodeId, MastForestError> {
		let block = MastNode::basic_block_with_raw_decorators(operations, decorators, self)?;
		self.add_node(block)
	}

	fn remap_and_add_nodes(
		&mut self,
		nodes_to_add: impl IntoIterator<Item = MastNode>,
		id_remappings: &BTreeMap<MastNodeId, MastNodeId>,
	) {
		assert!(self.nodes.is_empty());

		for live_node in nodes_to_add {
			match &live_node {
				MastNode::Join(join_node) => {
					let first_child = id_remappings
						.get(&join_node.first())
						.copied()
						.unwrap_or_else(|| join_node.first());
					let second_child = id_remappings
						.get(&join_node.second())
						.copied()
						.unwrap_or_else(|| join_node.second());

					self.add_join(first_child, second_child).unwrap();
				}
				MastNode::Split(split_node) => {
					let on_true_child = id_remappings
						.get(&split_node.on_true())
						.copied()
						.unwrap_or_else(|| split_node.on_true());
					let on_false_child = id_remappings
						.get(&split_node.on_false())
						.copied()
						.unwrap_or_else(|| split_node.on_false());

					self.add_split(on_true_child, on_false_child).unwrap();
				}
				MastNode::Loop(loop_node) => {
					let body_id = id_remappings
						.get(&loop_node.body())
						.copied()
						.unwrap_or_else(|| loop_node.body());

					self.add_loop(body_id).unwrap();
				}
				MastNode::Call(call_node) => {
					let callee_id = id_remappings
						.get(&call_node.callee())
						.copied()
						.unwrap_or_else(|| call_node.callee());

					if call_node.is_syscall() {
						self.add_syscall(callee_id).unwrap();
					} else {
						self.add_call(callee_id).unwrap();
					}
				}
				_ => {
					self.add_node(live_node).unwrap();
				}
			}
		}
	}

	fn remap_and_add_roots(
		&mut self,
		old_root_ids: impl IntoIterator<Item = MastNodeId>,
		id_remappings: &BTreeMap<MastNodeId, MastNodeId>,
	) {
		assert!(self.roots.is_empty());

		for old_root_id in old_root_ids {
			let new_root_id = id_remappings
				.get(&old_root_id)
				.copied()
				.unwrap_or(old_root_id);
			self.make_root(new_root_id);
		}
	}

	#[must_use]
	pub fn get_decorator_by_id(&self, decorator_id: DecoratorId) -> Option<&Decorator> {
		let idx = decorator_id.as_usize();

		self.decorators().get(idx)
	}

	#[must_use]
	pub fn get_node_by_id(&self, node_id: MastNodeId) -> Option<&MastNode> {
		let idx = node_id.as_usize();

		self.nodes().get(idx)
	}

	#[must_use]
	pub fn find_procedure_root(&self, digest: RpoDigest) -> Option<MastNodeId> {
		self.procedure_roots()
			.iter()
			.find(|&&root_id| {
				self.get_node_by_id(root_id)
					.is_some_and(|node| node.digest() == digest)
			})
			.copied()
	}

	#[must_use]
	pub fn is_procedure_root(&self, node_id: MastNodeId) -> bool {
		self.procedure_roots().contains(&node_id)
	}

	pub fn procedure_digests(&self) -> impl Iterator<Item = RpoDigest> + '_ {
		self.procedure_roots()
			.iter()
			.filter_map(|&root_id| self.get_node_by_id(root_id).map(MastNode::digest))
	}

	pub fn local_procedure_digests(&self) -> impl Iterator<Item = RpoDigest> + '_ {
		self.procedure_roots().iter().filter_map(|&root_id| {
			let node = self.get_node_by_id(root_id)?;
			if node.is_external() {
				None
			} else {
				Some(node.digest())
			}
		})
	}

	#[must_use]
	pub fn procedure_roots(&self) -> &[MastNodeId] {
		&self.roots
	}

	#[must_use]
	pub fn num_procedures(&self) -> u32 {
		self.procedure_roots()
			.len()
			.try_into()
			.expect("MAST forest contains more than 2^32 procedures")
	}

	#[must_use]
	pub fn num_nodes(&self) -> u32 {
		self.nodes().len() as u32
	}

	#[must_use]
	pub fn nodes(&self) -> &[MastNode] {
		&self.nodes
	}

	#[must_use]
	pub fn decorators(&self) -> &[Decorator] {
		&self.decorators
	}

	#[must_use]
	pub const fn advice_map(&self) -> &AdviceMap {
		&self.advice_map
	}

	pub fn advice_map_mut(&mut self) -> &mut AdviceMap {
		&mut self.advice_map
	}
}

impl Index<MastNodeId> for MastForest {
	type Output = MastNode;

	fn index(&self, index: MastNodeId) -> &Self::Output {
		let idx = index.0 as usize;

		&self.nodes[idx]
	}
}

impl IndexMut<MastNodeId> for MastForest {
	fn index_mut(&mut self, index: MastNodeId) -> &mut Self::Output {
		let idx = index.0 as usize;

		&mut self.nodes[idx]
	}
}

impl Index<DecoratorId> for MastForest {
	type Output = Decorator;

	fn index(&self, index: DecoratorId) -> &Self::Output {
		let idx = index.0 as usize;

		&self.decorators[idx]
	}
}

impl IndexMut<DecoratorId> for MastForest {
	fn index_mut(&mut self, index: DecoratorId) -> &mut Self::Output {
		let idx = index.0 as usize;

		&mut self.decorators[idx]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MastNodeId(u32);

impl MastNodeId {
	pub(crate) const fn new_unchecked(value: u32) -> Self {
		Self(value)
	}

	pub fn from_u32(value: u32, mast_forest: &MastForest) -> Result<Self, DeserializationError> {
		Self::from_u32_with_node_count(value, mast_forest.nodes().len())
	}

	pub fn from_usize(
		node_id: usize,
		mast_forest: &MastForest,
	) -> Result<Self, DeserializationError> {
		let node_id: u32 = node_id.try_into().map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"node id '{node_id}' does not fit into a u32"
			))
		})?;

		Self::from_u32(node_id, mast_forest)
	}

	pub(super) fn from_u32_with_node_count(
		id: u32,
		node_count: usize,
	) -> Result<Self, DeserializationError> {
		if (id as usize) < node_count {
			Ok(Self::new_unchecked(id))
		} else {
			Err(DeserializationError::InvalidValue(format!(
				"invalid deserialized MAST node ID '{id}', but {node_count} is the number of nodes in the forest"
			)))
		}
	}

	#[must_use]
	pub const fn as_usize(self) -> usize {
		self.0 as usize
	}

	#[must_use]
	pub const fn as_u32(self) -> u32 {
		self.0
	}
}

impl Display for MastNodeId {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("MastNodeId(")?;
		Display::fmt(&self.0, f)?;
		f.write_char(')')
	}
}

impl From<MastNodeId> for usize {
	fn from(value: MastNodeId) -> Self {
		value.as_usize()
	}
}

impl From<MastNodeId> for u32 {
	fn from(value: MastNodeId) -> Self {
		value.as_u32()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DecoratorId(u32);

impl DecoratorId {
	pub(crate) const fn new_unchecked(value: u32) -> Self {
		Self(value)
	}

	#[must_use]
	pub const fn as_usize(self) -> usize {
		self.0 as usize
	}

	#[must_use]
	pub const fn as_u32(self) -> u32 {
		self.0
	}

	pub fn from_u32(value: u32, mast_forest: &MastForest) -> Result<Self, DeserializationError> {
		Self::from_u32_with_node_count(value, mast_forest.nodes().len())
	}

	pub fn from_usize(
		node_id: usize,
		mast_forest: &MastForest,
	) -> Result<Self, DeserializationError> {
		let node_id: u32 = node_id.try_into().map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"node id '{node_id}' does not fit into a u32"
			))
		})?;

		Self::from_u32(node_id, mast_forest)
	}

	pub(super) fn from_u32_with_node_count(
		id: u32,
		node_count: usize,
	) -> Result<Self, DeserializationError> {
		if (id as usize) < node_count {
			Ok(Self::new_unchecked(id))
		} else {
			Err(DeserializationError::InvalidValue(format!(
				"invalid deserialized MAST node id '{id}', but {node_count} is the number of nodes in the forest"
			)))
		}
	}
}

impl Display for DecoratorId {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("DecoratorId(")?;
		Display::fmt(&self.0, f)?;
		f.write_char(')')
	}
}

impl From<DecoratorId> for usize {
	fn from(value: DecoratorId) -> Self {
		value.as_usize()
	}
}

impl From<DecoratorId> for u32 {
	fn from(value: DecoratorId) -> Self {
		value.as_u32()
	}
}

impl Serializable for DecoratorId {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.0.write_into(target);
	}
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MastForestError {
	#[error(
		"MAST forest decorator count exceeds the maximum of {} decorators",
		u32::MAX
	)]
	TooManyDecorators,
	#[error(
		"MAST forest node count exceeds the maximum of {} nodes",
		MastForest::MAX_NODES
	)]
	TooManyNodes,
	#[error("node id {0} is greater than or equal to forest length {1}")]
	NodeIdOverflow(MastNodeId, usize),
	#[error("decorator id {0} is greater than or equal to decorator count {1}")]
	DecoratorIdOverflow(DecoratorId, usize),
	#[error("basic block cannot be created from an empty list of operations")]
	EmptyBasicBlock,
	#[error(
		"decorator root of child with node id {0} is missing but required for fingerprint computation"
	)]
	ChildFingerprintMissing(MastNodeId),
	#[error("advice map key {0} already exists when merging forests")]
	AdviceMapKeyCollisionOnMerge(RpoDigest),
}

fn remove_nodes(
	mast_nodes: Vec<MastNode>,
	nodes_to_remove: &BTreeSet<MastNodeId>,
) -> (Vec<MastNode>, BTreeMap<MastNodeId, MastNodeId>) {
	assert!(mast_nodes.len() < u32::MAX as usize);

	let mut retained_nodes = Vec::with_capacity(mast_nodes.len());
	let mut id_remappings = BTreeMap::new();

	for (old_node_index, old_node) in mast_nodes.into_iter().enumerate() {
		let old_node_id = MastNodeId(old_node_index as u32);

		if !nodes_to_remove.contains(&old_node_id) {
			let new_node_id = MastNodeId(retained_nodes.len() as u32);
			id_remappings.insert(old_node_id, new_node_id);

			retained_nodes.push(old_node);
		}
	}

	(retained_nodes, id_remappings)
}
