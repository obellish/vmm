#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod cell;
mod impls;
mod ptr;

use core::ops::IndexMut;

pub use self::{cell::*, impls::*, ptr::*};

pub const TAPE_SIZE: usize = 30000;

pub trait Tape: Default + IndexMut<usize, Output = Cell> {
	#[must_use]
	fn init() -> Self {
		let mut tape = Self::default();

		for (i, cell) in tape.as_mut_slice().iter_mut().enumerate() {
			cell.set_index(i);
		}

		tape
	}

	fn as_slice(&self) -> &[Cell];

	fn as_mut_slice(&mut self) -> &mut [Cell];

	fn ptr(&self) -> &TapePointer;

	fn ptr_mut(&mut self) -> &mut TapePointer;

	fn current_cell(&self) -> &Cell {
		&self.as_slice()[self.ptr().value()]
	}

	fn current_cell_mut(&mut self) -> &mut Cell {
		let idx = self.ptr().value();
		&mut self.as_mut_slice()[idx]
	}

	unsafe fn current_cell_unchecked(&self) -> &Cell {
		unsafe { self.as_slice().get_unchecked(self.ptr().value()) }
	}

	unsafe fn current_cell_unchecked_mut(&mut self) -> &mut Cell {
		let idx = self.ptr().value();
		unsafe { self.as_mut_slice().get_unchecked_mut(idx) }
	}
}

#[cfg(test)]
mod tests {
	use vmm_testing::run_test;

	use super::{PtrTape, Tape as _};

	#[test]
	fn any_index_works() {
		let mut tape = PtrTape::new();
		_ = run_test(|u| {
			let idx = u.arbitrary::<usize>()?;

			*tape.ptr_mut() += idx;

			assert_eq!(tape.current_cell().value(), 1);

			Ok(())
		})
		.budget_ms(10);
	}
}
