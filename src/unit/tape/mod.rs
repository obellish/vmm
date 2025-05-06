mod ptr;

use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
	slice::SliceIndex,
};

use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

pub use self::ptr::TapePointer;

pub const TAPE_SIZE: usize = 5000;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tape {
	#[serde(with = "BigArray")]
	cells: [u8; TAPE_SIZE],
	pointer: TapePointer,
}

impl Tape {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: [0; TAPE_SIZE],
			pointer: TapePointer::new(),
		}
	}

	#[must_use]
	pub fn current_cell(&self) -> &u8 {
		unsafe { self.cells.get_unchecked(self.pointer.value()) }
	}

	pub fn current_cell_mut(&mut self) -> &mut u8 {
		unsafe { self.cells.get_unchecked_mut(self.pointer.value()) }
	}

	#[must_use]
	pub const fn pointer(&self) -> &TapePointer {
		&self.pointer
	}

	pub const fn pointer_mut(&mut self) -> &mut TapePointer {
		&mut self.pointer
	}
}

impl Debug for Tape {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().enumerate() {
			if matches!(cell, 0)
				&& !pretty_printing
				&& self.cells[i..].iter().all(|c| matches!(c, 0))
			{
				return state.finish_non_exhaustive();
			}

			state.entry(&cell);
		}

		state.finish()
	}
}

impl Default for Tape {
	fn default() -> Self {
		Self::new()
	}
}

impl<I> Index<I> for Tape
where
	I: SliceIndex<[u8]>,
{
	type Output = I::Output;

	fn index(&self, index: I) -> &Self::Output {
		self.cells.index(index)
	}
}

impl<I> IndexMut<I> for Tape
where
	I: SliceIndex<[u8]>,
{
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.cells.index_mut(index)
	}
}
