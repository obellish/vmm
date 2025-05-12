mod cell;

use std::{
	collections::{BTreeMap, HashMap},
	num::Wrapping,
};

use serde::{Deserialize, Serialize, Serializer};
use serde_array::BigArray;
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
				Instruction::SetVal(i) => self.mark((*i).into()),
				Instruction::IncVal(x) => self.mark((*x).into()),
				Instruction::Write => {
					self.mark(CellIncrement::Write);

					let mut snapshot = HashMap::new();

					for (i, cell) in self
						.cells
						.iter()
						.copied()
						.filter(|c| !c.state().is_empty())
						.enumerate()
					{
						snapshot.insert(i, cell);
					}

					output.snapshots.push(snapshot);
				}
				Instruction::Read => self.mark(CellIncrement::Read),
				_ => {}
			}
		}
	}

	fn mark(&mut self, value: CellIncrement) {
		*self.cell_mut() |= CellState::TOUCHED;

		if self.in_loop() {
			*self.cell_mut() |= CellState::IN_LOOP;
		}
		match value {
			CellIncrement::Set(v) => {
				self.value_mut().0 = v;
				if !self.in_loop() {
					self.state_mut().remove(CellState::IN_LOOP);
				}
			}
			CellIncrement::Inc(v) => *self.value_mut() += v,
			CellIncrement::Write => *self.cell_mut() |= CellState::WRITTEN,
			CellIncrement::Read => *self.cell_mut() |= CellState::RECEIVED_FROM_STDIN,
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
	#[serde(serialize_with = "ordered_maps")]
	pub snapshots: Vec<HashMap<usize, Cell>>,
}

#[allow(clippy::ptr_arg)]
fn ordered_maps<S>(values: &Vec<HashMap<usize, Cell>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let ordered = values
		.iter()
		.map(|v| v.iter().map(|(k, v)| (*k, *v)).collect::<BTreeMap<_, _>>())
		.collect::<Vec<_>>();

	ordered.serialize(serializer)
}

enum CellIncrement {
	Inc(u8),
	Set(u8),
	Write,
	Read,
}

impl From<i8> for CellIncrement {
	fn from(value: i8) -> Self {
		Self::Inc(value as u8)
	}
}

impl From<u8> for CellIncrement {
	fn from(value: u8) -> Self {
		Self::Set(value)
	}
}
