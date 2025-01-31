mod node;

use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use thiserror::Error;

pub use self::node::DynNode;
use crate::{
	crypto::hash::RpoDigest,
	utils::{ByteWriter, DeserializationError, Serializable},
};

pub struct MastForest {}

impl MastForest {
	const MAX_NODES: usize = (1 << 30) - 1;
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
