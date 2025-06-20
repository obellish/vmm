use alloc::vec::Vec;
use core::{
	ops::{Index, IndexMut},
	slice::SliceIndex,
};

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
			cells: {
				let mut cells = Vec::with_capacity(TAPE_SIZE);
				for i in 0..TAPE_SIZE {
					cells.push(Cell::with_index(0, i));
				}

				cells.shrink_to_fit();

				cells
			},
			ptr: unsafe { TapePointer::new_unchecked(0) },
		}
	}
}

impl Default for VecTape {
	fn default() -> Self {
		Self::new()
	}
}

impl<I> Index<I> for VecTape
where
	I: SliceIndex<[Cell]>,
{
	type Output = I::Output;

	fn index(&self, index: I) -> &Self::Output {
		self.cells.index(index)
	}
}

impl<I> IndexMut<I> for VecTape
where
	I: SliceIndex<[Cell]>,
{
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.cells.index_mut(index)
	}
}

impl Tape for VecTape {
	fn init(&mut self) {}

	// We don't need to init, as we do it in `new`

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
