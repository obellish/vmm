#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod cell;
mod ptr;

use alloc::boxed::Box;
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
};

use vmm_num::Wrapping;

pub use self::{cell::*, ptr::*};

pub const TAPE_SIZE: usize = 30000;

#[derive(Clone, PartialEq, Eq)]
pub struct Tape {
	// We use a custom allocator, so we put this on the heap
	// cells: Box<[Wrapping<u8>; TAPE_SIZE]>,
	cells: Box<[Cell; TAPE_SIZE]>,
	ptr: TapePointer,
}

impl Tape {
	#[inline]
	#[must_use]
	pub fn new() -> Self {
		let cells = Box::new([Cell::new(0); TAPE_SIZE]);

		Self {
			// cells: Box::new([Wrapping(0); TAPE_SIZE]),
			// cells: Box::new(core::array::from_fn(|idx| Cell::with_index(0, idx))),
			cells,
			ptr: unsafe { TapePointer::new_unchecked(0) },
		}
	}

	#[must_use]
	pub fn cell(&self) -> &Cell {
		unsafe { self.cells.get_unchecked(self.ptr.value()) }
	}

	pub fn cell_mut(&mut self) -> &mut Cell {
		unsafe { self.cells.get_unchecked_mut(self.ptr.value()) }
	}

	#[must_use]
	pub const fn ptr(&self) -> &TapePointer {
		&self.ptr
	}

	pub const fn ptr_mut(&mut self) -> &mut TapePointer {
		&mut self.ptr
	}

	#[must_use]
	pub const fn as_slice(&self) -> &[Cell] {
		&*self.cells
	}

	pub const fn as_mut_slice(&mut self) -> &mut [Cell] {
		&mut *self.cells
	}

	#[must_use]
	pub const fn as_ptr(&self) -> *const Cell {
		self.cells.as_ptr()
	}

	pub const fn as_mut_ptr(&mut self) -> *mut Cell {
		self.cells.as_mut_ptr()
	}
}

impl Debug for Tape {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let pretty_printing = f.alternate();
		let mut state = f.debug_list();

		for (i, cell) in self.cells.iter().copied().map(Cell::value).enumerate() {
			if matches!(cell, 0)
				&& !pretty_printing
				&& self.cells[i..].iter().all(|c| matches!(c.value(), 0))
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
	type Output = Cell;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		&self.cells[index % TAPE_SIZE]
	}
}

impl IndexMut<usize> for Tape {
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.cells[index % TAPE_SIZE]
	}
}

#[cfg(test)]
mod tests {
	use vmm_testing::run_test;

	use super::Tape;

	#[test]
	fn any_index_works() {
		let mut tape = Tape::new();
		_ = run_test(|u| {
			let idx = u.arbitrary::<usize>()?;

			*tape.ptr_mut() += idx;

			assert_eq!(tape.cell().value(), 1);

			Ok(())
		})
		.budget_ms(10);
	}
}
