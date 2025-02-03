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

	fn decode_u32_pair(payload: u64) -> (u32, u32) {
		let left_value = (payload >> 30) as u32;
		let right_value = (payload & 0x3f_ff_ff_ff) as u32;

		(left_value, right_value)
	}

	pub fn decode_u32_payload(payload: u64) -> Result<u32, DeserializationError> {
		payload.try_into().map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"invalid payload: expected to fit in u32, but was {}",
				payload
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
