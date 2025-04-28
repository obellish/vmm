use std::fmt::{Debug, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

const TAPE_SIZE: usize = 1000;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tape {
	#[serde(with = "BigArray")]
	cells: [u8; TAPE_SIZE],
	pointer: usize,
}

impl Tape {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: [0; TAPE_SIZE],
			pointer: 0,
		}
	}

	#[must_use]
	pub fn current_cell(&self) -> &u8 {
		unsafe { self.cells.get_unchecked(self.pointer) }
	}

	pub fn current_cell_mut(&mut self) -> &mut u8 {
		unsafe { self.cells.get_unchecked_mut(self.pointer) }
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
