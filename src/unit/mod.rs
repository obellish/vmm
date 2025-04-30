mod program;
mod tape;

use std::mem;

use serde::{Deserialize, Serialize};

pub use self::{program::*, tape::*};
use super::Instruction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionUnit {
	program: Program,
	tape: Tape,
}

impl ExecutionUnit {
	pub fn raw(instructions: impl IntoIterator<Item = Instruction>) -> Self {
		Self {
			program: instructions.into_iter().collect(),
			tape: Tape::new(),
		}
	}

	pub(crate) fn optimized(
		instructions: impl IntoIterator<Item = Instruction>,
	) -> Self {
		Self {
			program: Program::Optimized(instructions.into_iter().collect()),
			tape: Tape::new(),
		}
	}

	#[must_use]
	pub const fn program(&self) -> &Program {
		&self.program
	}

	pub const fn program_mut(&mut self) -> &mut Program {
		&mut self.program
	}

	#[must_use]
	pub const fn tape(&self) -> &Tape {
		&self.tape
	}

	pub const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}
}

impl Default for ExecutionUnit {
	fn default() -> Self {
		Self::raw([])
	}
}

impl FromIterator<Instruction> for ExecutionUnit {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = Instruction>,
	{
		Self::raw(iter)
	}
}
