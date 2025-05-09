mod cell;

use std::fmt::{Debug, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use serde_array::BigArray;

pub use self::cell::*;
use crate::{Instruction, StackedInstruction, TAPE_SIZE, TapePointer};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticAnalyzer<'a> {
	#[serde(skip)]
	program: &'a [Instruction],
	#[serde(with = "BigArray")]
	cells: [CellState; TAPE_SIZE],
	pointer: TapePointer,
	depth: usize,
}

impl<'a> StaticAnalyzer<'a> {
	#[must_use]
	pub const fn new(program: &'a [Instruction]) -> Self {
		Self {
			program,
			cells: [CellState::empty(); TAPE_SIZE],
			pointer: unsafe { TapePointer::new_unchecked(0) },
			depth: 0,
		}
	}

	#[must_use]
	pub fn analyze(mut self) -> AnalysisOutput {
		self.analyze_inner(self.program);

		AnalysisOutput { cells: self.cells }
	}

	fn analyze_inner(&mut self, program: &[Instruction]) {
		for instr in program {
			match instr {
				Instruction::Stacked(StackedInstruction::MovePtr(i)) => self.pointer += *i,
				Instruction::RawLoop(i) => {
					self.depth += 1;

					self.analyze_inner(i);

					self.depth -= 1;
				}
				Instruction::Stacked(StackedInstruction::IncVal(_)) => {
					self.mark(self.pointer.value());
				}
				_ => {}
			}
		}
	}

	fn mark(&mut self, cell: usize) {
		self.cells[cell] |= CellState::TOUCHED;

		if self.in_loop() {
			self.cells[cell] |= CellState::IN_LOOP;
		}
	}

	const fn in_loop(&self) -> bool {
		!matches!(self.depth, 0)
	}
}

impl Debug for StaticAnalyzer<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().copied().enumerate() {
			if cell.is_empty()
				&& !pretty_printing
				&& self.cells[i..].iter().all(CellState::is_empty)
			{
				return state.finish_non_exhaustive();
			}

			state.entry(&cell);
		}

		state.finish()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisOutput {
	#[serde(with = "BigArray")]
	pub cells: [CellState; TAPE_SIZE],
}
