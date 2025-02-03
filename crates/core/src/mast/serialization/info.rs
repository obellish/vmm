use alloc::vec::Vec;

use super::{NodeDataOffset, basic_block::BasicBlockDataDecoder};
use crate::{
	crypto::hash::RpoDigest,
	mast::{BasicBlockNode, CallNode, JoinNode, LoopNode, MastNode, MastNodeId, SplitNode},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

const JOIN: u8 = 0;
const SPLIT: u8 = 1;
const LOOP: u8 = 2;
const BLOCK: u8 = 3;
const CALL: u8 = 4;
const SYSCALL: u8 = 5;
const DYN: u8 = 6;
const DYNCALL: u8 = 7;
const EXTERNAL: u8 = 8;

#[derive(Debug)]
pub struct MastNodeInfo {
	ty: MastNodeType,
	digest: RpoDigest,
}

impl MastNodeInfo {
	pub fn new(mast_node: &MastNode, ops_offset: NodeDataOffset) -> Self {
		if !mast_node.is_basic_block() {
			debug_assert_eq!(ops_offset, 0);
		}

		let ty = MastNodeType::new(mast_node, ops_offset);

		Self {
			ty,
			digest: mast_node.digest(),
		}
	}

	pub fn try_into_mast_node(
		self,
		node_count: usize,
		basic_block_data_decoder: &BasicBlockDataDecoder<'_>,
	) -> Result<MastNode, DeserializationError> {
		match self.ty {
			MastNodeType::Block { ops_offset } => {
				let operation = basic_block_data_decoder.decode_operations(ops_offset)?;
				let block = BasicBlockNode::new_unchecked(operation, Vec::new(), self.digest);
				Ok(MastNode::Block(block))
			}
			MastNodeType::Join {
				left_child_id,
				right_child_id,
			} => {
				let left_child = MastNodeId::from_u32_with_node_count(left_child_id, node_count)?;
				let right_child = MastNodeId::from_u32_with_node_count(right_child_id, node_count)?;
				let join = JoinNode::new_unchecked([left_child, right_child], self.digest);
				Ok(MastNode::Join(join))
			}
			MastNodeType::Split {
				if_branch_id,
				else_branch_id,
			} => {
				let if_branch = MastNodeId::from_u32_with_node_count(if_branch_id, node_count)?;
				let else_branch = MastNodeId::from_u32_with_node_count(else_branch_id, node_count)?;
				let split = SplitNode::new_unchecked([if_branch, else_branch], self.digest);
				Ok(MastNode::Split(split))
			}
			MastNodeType::Loop { body_id } => {
				let body_id = MastNodeId::from_u32_with_node_count(body_id, node_count)?;
				let loop_node = LoopNode::new_unchecked(body_id, self.digest);
				Ok(MastNode::Loop(loop_node))
			}
			MastNodeType::Call { callee_id } => {
				let callee_id = MastNodeId::from_u32_with_node_count(callee_id, node_count)?;
				let call = CallNode::new_unchecked(callee_id, self.digest);
				Ok(MastNode::Call(call))
			}
			MastNodeType::SysCall { callee_id } => {
				let callee_id = MastNodeId::from_u32_with_node_count(callee_id, node_count)?;
				let call = CallNode::syscall_unchecked(callee_id, self.digest);
				Ok(MastNode::Call(call))
			}
			MastNodeType::Dyn => Ok(MastNode::r#dyn()),
			MastNodeType::DynCall => Ok(MastNode::dyncall()),
			MastNodeType::External => Ok(MastNode::external(self.digest)),
		}
	}
}

impl Deserializable for MastNodeInfo {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let ty = Deserializable::read_from(source)?;
		let digest = RpoDigest::read_from(source)?;

		Ok(Self { ty, digest })
	}
}

impl Serializable for MastNodeInfo {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let Self { ty, digest } = self;

		ty.write_into(target);
		digest.write_into(target);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MastNodeType {
	Join {
		left_child_id: u32,
		right_child_id: u32,
	} = JOIN,
	Split {
		if_branch_id: u32,
		else_branch_id: u32,
	} = SPLIT,
	Loop {
		body_id: u32,
	} = LOOP,
	Block {
		ops_offset: u32,
	} = BLOCK,
	Call {
		callee_id: u32,
	} = CALL,
	SysCall {
		callee_id: u32,
	} = SYSCALL,
	Dyn = DYN,
	DynCall = DYNCALL,
	External = EXTERNAL,
}

impl MastNodeType {
	pub const fn new(mast_node: &MastNode, ops_offset: NodeDataOffset) -> Self {
		match mast_node {
			MastNode::Block(_) => Self::Block { ops_offset },
			MastNode::Join(node) => Self::Join {
				left_child_id: node.first().0,
				right_child_id: node.second().0,
			},
			MastNode::Split(node) => Self::Split {
				if_branch_id: node.on_true().0,
				else_branch_id: node.on_false().0,
			},
			MastNode::Loop(node) => Self::Loop {
				body_id: node.body().0,
			},
			MastNode::Call(node) if node.is_syscall() => Self::SysCall {
				callee_id: node.callee().0,
			},
			MastNode::Call(node) => Self::Call {
				callee_id: node.callee().0,
			},
			MastNode::Dyn(node) if node.is_dyncall() => Self::DynCall,
			MastNode::Dyn(_) => Self::Dyn,
			MastNode::External(_) => Self::External,
		}
	}

	fn discriminant(&self) -> u8 {
		unsafe { *<*const _>::from(self).cast::<u8>() }
	}

	fn encode_u32_pair(left_value: u32, right_value: u32) -> u64 {
		assert!(
			left_value.leading_zeros() >= 2,
			"left value doesn't fit in 30 bits: {left_value}"
		);

		assert!(
			right_value.leading_zeros() >= 2,
			"right value doesn't fit in 30 bits: {right_value}"
		);

		(u64::from(left_value) << 30) | u64::from(right_value)
	}

	fn encode_u32_payload(payload: u32) -> u64 {
		payload.into()
	}

	const fn decode_u32_pair(payload: u64) -> (u32, u32) {
		let left_value = (payload >> 30) as u32;
		let right_value = (payload & 0x3f_ff_ff_ff) as u32;

		(left_value, right_value)
	}

	pub fn decode_u32_payload(payload: u64) -> Result<u32, DeserializationError> {
		payload.try_into().map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"invalid payload: expected to fit in u32, but was {payload}"
			))
		})
	}
}

impl Deserializable for MastNodeType {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let (discriminant, payload) = {
			let value = source.read_u64()?;

			let discriminant = (value >> 60) as u8;
			let payload = value & 0xf_ff_ff_ff_ff_ff_ff_ff;

			(discriminant, payload)
		};

		match discriminant {
			JOIN => {
				let (left_child_id, right_child_id) = Self::decode_u32_pair(payload);
				Ok(Self::Join {
					left_child_id,
					right_child_id,
				})
			}
			SPLIT => {
				let (if_branch_id, else_branch_id) = Self::decode_u32_pair(payload);
				Ok(Self::Split {
					if_branch_id,
					else_branch_id,
				})
			}
			LOOP => {
				let body_id = Self::decode_u32_payload(payload)?;
				Ok(Self::Loop { body_id })
			}
			BLOCK => {
				let ops_offset = Self::decode_u32_payload(payload)?;
				Ok(Self::Block { ops_offset })
			}
			CALL => {
				let callee_id = Self::decode_u32_payload(payload)?;
				Ok(Self::Call { callee_id })
			}
			SYSCALL => {
				let callee_id = Self::decode_u32_payload(payload)?;
				Ok(Self::SysCall { callee_id })
			}
			DYN => Ok(Self::Dyn),
			DYNCALL => Ok(Self::DynCall),
			EXTERNAL => Ok(Self::External),
			_ => Err(DeserializationError::InvalidValue(format!(
				"invalid tag for MAST node: {discriminant}"
			))),
		}
	}
}

impl Serializable for MastNodeType {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let discriminant = u64::from(self.discriminant());
		assert!(discriminant <= 0b1111);

		let payload = match *self {
			Self::Join {
				left_child_id: left,
				right_child_id: right,
			} => Self::encode_u32_pair(left, right),
			Self::Split {
				if_branch_id: if_branch,
				else_branch_id: else_branch,
			} => Self::encode_u32_pair(if_branch, else_branch),
			Self::Loop { body_id: body } => Self::encode_u32_payload(body),
			Self::Block { ops_offset } => Self::encode_u32_payload(ops_offset),
			Self::Call { callee_id } | Self::SysCall { callee_id } => {
				Self::encode_u32_payload(callee_id)
			}
			Self::Dyn | Self::DynCall | Self::External => 0,
		};

		let value = (discriminant << 60) | payload;
		target.write_u64(value);
	}
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use super::{CALL, MastNodeType};
	use crate::utils::{Deserializable, DeserializationError, Serializable};

	#[test]
	fn serialize_deserialize_60_bit_payload() -> Result<(), DeserializationError> {
		let mast_node_type = MastNodeType::Join {
			left_child_id: 0x3f_ff_ff_ff,
			right_child_id: 0x3f_ff_ff_ff,
		};

		let serialized = mast_node_type.to_bytes();
		let deserialized = MastNodeType::read_from_bytes(&serialized)?;

		assert_eq!(mast_node_type, deserialized);

		Ok(())
	}

	#[test]
	#[should_panic = "left value doesn't fit in 30 bits"]
	fn serialize_large_payload_fails_1() {
		let mast_node_type = MastNodeType::Join {
			left_child_id: 0x4f_ff_ff_ff,
			right_child_id: 0x0,
		};

		_ = mast_node_type.to_bytes();
	}

	#[test]
	#[should_panic = "right value doesn't fit in 30 bits"]
	fn serialize_large_payload_fails_2() {
		let mast_node_type = MastNodeType::Join {
			left_child_id: 0x0,
			right_child_id: 0x4f_ff_ff_ff,
		};

		_ = mast_node_type.to_bytes();
	}

	#[test]
	fn deserialize_large_payload_fails() {
		let serialized = {
			let serialized_value = (u64::from(CALL) << 60) | (u64::from(u32::MAX) + 1u64);

			let mut serialized_buffer = Vec::new();
			serialized_value.write_into(&mut serialized_buffer);

			serialized_buffer
		};

		let deserialized = MastNodeType::read_from_bytes(&serialized);

		assert!(deserialized.is_err());
	}
}
