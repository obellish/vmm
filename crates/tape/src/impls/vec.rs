use alloc::vec::Vec;

use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct VecTape {
	cells: Vec<Cell>,
	ptr: TapePointer,
}

impl VecTape {
	#[must_use]
	pub fn new() -> Self {
		Self {
			cells: alloc::vec![Cell::new(0); TAPE_SIZE],
			ptr: TapePointer::zero(),
		}
	}
}

impl Default for VecTape {
	fn default() -> Self {
		Self::new()
	}
}

impl Tape for VecTape {
	fn as_slice(&self) -> &[Cell] {
		self.cells.as_slice()
	}

	fn as_mut_slice(&mut self) -> &mut [Cell] {
		self.cells.as_mut_slice()
	}

	fn ptr(&self) -> &TapePointer {
		&self.ptr
	}

	fn ptr_mut(&mut self) -> &mut TapePointer {
		&mut self.ptr
	}
}
