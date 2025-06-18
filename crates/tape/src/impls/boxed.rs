use alloc::boxed::Box;
use core::ops::{Index, IndexMut};

use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct BoxTape {
	cells: Box<[Cell; TAPE_SIZE]>,
	ptr: TapePointer,
}

impl BoxTape {
	#[must_use]
	#[expect(clippy::large_stack_frames)]
	pub fn new() -> Self {
		Self {
			cells: Box::new([Cell::new(0); TAPE_SIZE]),
			ptr: unsafe { TapePointer::new_unchecked(0) },
		}
	}
}

impl Default for BoxTape {
	fn default() -> Self {
		Self::new()
	}
}

impl Index<usize> for BoxTape {
	type Output = Cell;

	fn index(&self, index: usize) -> &Self::Output {
		&self.cells[index % TAPE_SIZE]
	}
}

impl IndexMut<usize> for BoxTape {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.cells[index % TAPE_SIZE]
	}
}

impl Tape for BoxTape {
	fn as_slice(&self) -> &[Cell] {
		&*self.cells
	}

	fn as_mut_slice(&mut self) -> &mut [Cell] {
		&mut *self.cells
	}

	fn ptr(&self) -> &TapePointer {
		&self.ptr
	}

	fn ptr_mut(&mut self) -> &mut TapePointer {
		&mut self.ptr
	}

	unsafe fn current_cell_unchecked(&self) -> &Cell {
		unsafe { self.cells.get_unchecked(self.ptr().value()) }
	}

	unsafe fn current_cell_unchecked_mut(&mut self) -> &mut Cell {
		let idx = self.ptr().value();
		unsafe { self.cells.get_unchecked_mut(idx) }
	}
}
