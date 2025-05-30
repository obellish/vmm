use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::{Instruction, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LoopInstruction {
	Dynamic(Vec<Instruction>),
	IfNz(Vec<Instruction>),
}

impl LoopInstruction {
	pub fn dynamic(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::Dynamic(i.into_iter().collect())
	}

	pub fn if_nz(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::IfNz(i.into_iter().collect())
	}
}

impl PtrMovement for LoopInstruction {
	fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::Dynamic(instrs) | Self::IfNz(instrs) => instrs.ptr_movement(),
		}
	}
}
