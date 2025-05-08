mod cell;

use std::fmt::{Debug, Formatter, Result as FmtResult};

pub use self::cell::*;
use crate::{Instruction, TAPE_SIZE, TapePointer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StaticAnalyzer<'a> {
	program: &'a [Instruction],
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

	pub fn analyze(&mut self) {
		self.analyze_inner(self.program);
	}

	fn analyze_inner(&mut self, program: &[Instruction]) {
		for instr in program {
			match instr {
				Instruction::MovePtr(i) => self.pointer += *i,
				Instruction::RawLoop(i) => {
					self.depth += 1;

					self.analyze_inner(i);

					self.depth -= 1;
				}
				Instruction::IncVal(_) => self.mark(self.pointer.value()),
				_ => {}
			}
		}
	}

	#[must_use]
	pub const fn cells(&self) -> [CellState; TAPE_SIZE] {
		self.cells
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
