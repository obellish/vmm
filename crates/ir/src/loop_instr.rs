use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::Instruction;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LoopInstruction {
	Dynamic(Vec<Instruction>),
}

impl LoopInstruction {
	pub fn dynamic(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::Dynamic(i.into_iter().collect())
	}
}
