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
	has_started: bool,
}

impl ExecutionUnit {
	pub fn raw(instructions: impl IntoIterator<Item = Instruction>) -> Self {
		Self {
			program: instructions.into_iter().collect(),
			tape: Tape::new(),
			has_started: false,
		}
	}

	pub(crate) fn optimized(
		instructions: impl IntoIterator<Item = Instruction>,
		tape: Tape,
	) -> Self {
		Self {
			program: Program::Optimized(instructions.into_iter().collect()),
			tape,
			has_started: false,
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
	pub const fn has_started(&self) -> bool {
		self.has_started
	}

	#[must_use]
	pub const fn tape(&self) -> &Tape {
		&self.tape
	}

	pub const fn tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn run(&mut self) {
		self.has_started = true;

		mem::take(&mut self.program)
			.iter()
			.copied()
			.for_each(|instr| {
				// noop
			});
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
