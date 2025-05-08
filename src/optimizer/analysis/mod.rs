mod cell;

use std::fmt::{Debug, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use serde_array::BigArray;

pub use self::cell::*;
use crate::{Instruction, TAPE_SIZE, TapePointer};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticAnalyzer {
	#[serde(with = "BigArray")]
	pub cells: [CellState; TAPE_SIZE],
	pub pointer: TapePointer,
	depth: usize,
}

impl StaticAnalyzer {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: [CellState::Untouched; TAPE_SIZE],
			pointer: unsafe { TapePointer::new_unchecked(0) },
			depth: 0,
		}
	}

	pub fn analyze(&mut self, program: &[Instruction]) {
		for instr in program {
			match instr {
				Instruction::IncVal(_) | Instruction::SetVal(_) => {
					self.mark(self.pointer.value());
				}
				Instruction::MovePtr(i) => self.pointer += *i,
				Instruction::FindZero(i) => {
					while !self.cells[self.pointer.value()].is_touched() {
						self.pointer += *i;
					}
				}
				Instruction::RawLoop(inner) => {
					self.depth += 1;

					self.analyze(inner);

					self.depth -= 1;
				}
				_ => {}
			}
		}
	}

	#[must_use]
	pub const fn cells(&self) -> [CellState; TAPE_SIZE] {
		self.cells
	}

	#[must_use]
	pub fn output(&self) -> String {
		self.cells.into_iter().map(|c| c.to_string()).collect()
	}

	const fn mark(&mut self, cell: usize) {
		if self.cells[cell].is_untouched() {
			self.cells[cell] = if matches!(self.depth, 0) {
				CellState::Touched
			} else {
				CellState::InLoop
			};
		}
	}
}

impl Debug for StaticAnalyzer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().copied().enumerate() {
			if cell.is_untouched()
				&& !pretty_printing
				&& self.cells[i..].iter().all(|v| v.is_untouched())
			{
				return state.finish_non_exhaustive();
			}

			state.entry(&cell);
		}

		state.finish()
	}
}

impl Default for StaticAnalyzer {
	fn default() -> Self {
		Self::new()
	}
}
