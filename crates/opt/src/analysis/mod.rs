mod cell;

use std::num::Wrapping;

use serde::{Deserialize, Serialize};
use serde_array::{Array, BigArray};
use vmm_ir::Instruction;
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

#[allow(dead_code)]
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
		let mut output = AnalysisOutput {
			snapshots: Vec::new(),
		};

		self.analyze_instructions(self.program, &mut output);

		output
	}

	fn analyze_instructions(&mut self, program: &[Instruction], output: &mut AnalysisOutput) {
		for instr in program {
			match instr {
				Instruction::MovePtr(i) => self.ptr += *i,
				Instruction::RawLoop(i) => {
					self.depth += 1;

					self.analyze_instructions(i, output);

					self.depth -= 1;
				}
				Instruction::IncVal(x) => self.mark(*x as u8),
				Instruction::Write => output.snapshots.push(self.cells.into()),
				_ => {}
			}
		}
	}

	fn mark(&mut self, value: u8) {
		*self.cell_mut() |= CellState::TOUCHED;

		if self.in_loop() {
			*self.cell_mut() |= CellState::IN_LOOP;
		}

		if !self.cell().state().contains(CellState::IN_LOOP) {
			*self.value_mut() += value;
		}
	}

	const fn in_loop(&self) -> bool {
		!matches!(self.depth, 0)
	}

	const fn cell(&self) -> &Cell {
		&self.cells[self.ptr.value()]
	}

	const fn cell_mut(&mut self) -> &mut Cell {
		&mut self.cells[self.ptr.value()]
	}

	const fn value(&self) -> &Wrapping<u8> {
		self.cell().value()
	}

	const fn value_mut(&mut self) -> &mut Wrapping<u8> {
		self.cell_mut().value_mut()
	}

	const fn state(&self) -> &CellState {
		self.cell().state()
	}

	const fn state_mut(&mut self) -> &mut CellState {
		self.cell_mut().state_mut()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisOutput {
	pub snapshots: Vec<Array<Cell, TAPE_SIZE>>,
}
