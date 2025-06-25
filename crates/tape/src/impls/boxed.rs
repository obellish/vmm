use alloc::boxed::Box;

use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct BoxTape {
	cells: Box<[Cell; TAPE_SIZE]>,
	ptr: TapePointer,
}

impl BoxTape {
	#[must_use]
	#[allow(clippy::large_stack_frames)]
	pub fn new() -> Self {
		Self {
			cells: Box::new([Cell::new(0); TAPE_SIZE]),
			ptr: TapePointer::zero()
		}
	}
}

impl Default for BoxTape {
	fn default() -> Self {
		Self::new()
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

	unsafe fn get_unchecked(&self, ptr: usize) -> &Cell {
		unsafe { self.cells.get_unchecked(ptr) }
	}

	unsafe fn get_unchecked_mut(&mut self, ptr: usize) -> &mut Cell {
		unsafe { self.cells.get_unchecked_mut(ptr) }
	}
}
