mod ptr;
#[cfg(test)]
mod tests;

use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	num::Wrapping,
	ops::{Index, IndexMut},
};

use serde::{Deserialize, Serialize};
use serde_array::BigArray;

pub use self::ptr::TapePointer;

pub const TAPE_SIZE: usize = 500;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tape {
	#[serde(with = "BigArray")]
	cells: [Wrapping<u8>; TAPE_SIZE],
	pointer: TapePointer,
}

impl Tape {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: [Wrapping(0); TAPE_SIZE],
			pointer: unsafe { TapePointer::new_unchecked(0) },
		}
	}

	#[must_use]
	pub fn current_cell(&self) -> &Wrapping<u8> {
		unsafe { self.cells.get_unchecked(self.pointer.value()) }
	}

	pub fn current_cell_mut(&mut self) -> &mut Wrapping<u8> {
		unsafe { self.cells.get_unchecked_mut(self.pointer.value()) }
	}

	#[must_use]
	pub const fn pointer(&self) -> &TapePointer {
		&self.pointer
	}

	pub const fn pointer_mut(&mut self) -> &mut TapePointer {
		&mut self.pointer
	}

	#[expect(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn as_slice(&self) -> &[Wrapping<u8>] {
		&self.cells
	}

	#[expect(clippy::missing_const_for_fn)]
	pub fn as_mut_slice(&mut self) -> &mut [Wrapping<u8>] {
		&mut self.cells
	}
}

impl Debug for Tape {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().copied().map(|i| i.0).enumerate() {
			if matches!(cell, 0)
				&& !pretty_printing
				&& self.cells[i..].iter().all(|c| matches!(c.0, 0))
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

impl Index<usize> for Tape {
	type Output = Wrapping<u8>;

	fn index(&self, index: usize) -> &Self::Output {
		self.cells.index(index)
	}
}

impl IndexMut<usize> for Tape {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		self.cells.index_mut(index)
	}
}
