mod cell;

use std::fmt::{Debug, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use serde_array::BigArray;
use vmm_ir::Instruction;
use vmm_program::Program;
use vmm_tape::{TAPE_SIZE, TapePointer};

pub use self::cell::*;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticAnalyzer<'a> {
	#[serde(skip)]
	program: &'a [Instruction],
	#[serde(with = "BigArray")]
	cells: [Cell; TAPE_SIZE],
	ptr: TapePointer,
	depth: usize,
}

impl<'a> StaticAnalyzer<'a> {
	#[must_use]
	pub const fn new(program: &'a [Instruction]) -> Self {
		Self {
			program,
			cells: [Cell::new(); TAPE_SIZE],
			ptr: unsafe { TapePointer::new_unchecked(0) },
			depth: 0,
		}
	}

	#[must_use]
	pub fn analyze(mut self) -> AnalysisOutput {
		self.analyze_instructions(self.program);

		AnalysisOutput { cells: self.cells }
	}

	fn analyze_instructions(&mut self, program: &[Instruction]) {
		for instr in program {
			match instr {
				Instruction::MovePtr(i) => self.ptr += *i,
				Instruction::RawLoop(i) => {
					self.depth += 1;

					self.analyze_instructions(i);

					self.depth -= 1;
				}
				Instruction::IncVal(_) => self.mark(self.ptr.value()),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisOutput {
	#[serde(with = "BigArray")]
	pub cells: [Cell; TAPE_SIZE],
}
