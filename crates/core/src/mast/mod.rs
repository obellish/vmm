mod node;
mod node_fingerprint;
mod serialization;

use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	ops::{Index, IndexMut},
};
use std::vec::Vec;

use thiserror::Error;

pub use self::{
	node::{
		BasicBlockNode, CallNode, DynNode, ExternalNode, JoinNode, LoopNode, MastNode,
		OP_BATCH_SIZE, OP_GROUP_SIZE, OpBatch, OperationOrDecorator, SplitNode,
	},
	node_fingerprint::{DecoratorFingerprint, MastNodeFingerprint},
};
use crate::{
	AdviceMap, Decorator,
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
		if self.decorators.len() >= u32::MAX as usize {
			return Err(MastForestError::TooManyDecorators);
		}

		let new_decorator_id = DecoratorId(self.decorators.len() as u32);
		self.decorators.push(decorator);

		Ok(new_decorator_id)
	}

	pub fn add_node(&mut self, node: MastNode) -> Result<MastNodeId, MastForestError> {
		if self.nodes.len() == Self::MAX_NODES {
			return Err(MastForestError::TooManyNodes);
		}

		let new_node_id = MastNodeId(self.nodes.len() as u32);
		self.nodes.push(node);

		Ok(new_node_id)
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
