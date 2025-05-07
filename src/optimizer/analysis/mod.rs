use std::fmt::{Debug, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use serde_array::BigArray;

use crate::{Instruction, TAPE_SIZE, TapePointer};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellAnalyzer {
	#[serde(with = "BigArray")]
	pub cells: [bool; TAPE_SIZE],
	pub pointer: TapePointer,
}

impl CellAnalyzer {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: [false; TAPE_SIZE],
			pointer: unsafe { TapePointer::new_unchecked(0) },
		}
	}

	pub fn analyze(&mut self, program: &[Instruction]) {
		for instr in program {
			match instr {
				Instruction::Inc(_) | Instruction::Set(_) => {
					self.mark(self.pointer.value());
				}
				Instruction::MovePtr(i) => self.pointer += *i,
				Instruction::FindZero(i) => {
					while !self.cells[self.pointer.value()] {
						self.pointer += *i;
					}
				}
				Instruction::RawLoop(inner) => self.analyze(inner),
				_ => {}
			}
		}
	}

	#[must_use]
	pub const fn cells(&self) -> [bool; TAPE_SIZE] {
		self.cells
	}

	const fn mark(&mut self, cell: usize) {
		self.cells[cell] = true;
	}
}

impl Debug for CellAnalyzer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().copied().enumerate() {
			if !cell && !pretty_printing && self.cells[i..].iter().all(|v| !*v) {
				return state.finish_non_exhaustive();
			}

			state.entry(&cell);
		}

		state.finish()
	}
}

impl Default for CellAnalyzer {
	fn default() -> Self {
		Self::new()
	}
}
