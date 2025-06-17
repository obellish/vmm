#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod cell;
mod ptr;

use core::{
	alloc::{Layout, LayoutError},
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
	ptr::NonNull,
};

pub use self::{cell::*, ptr::*};

pub const TAPE_SIZE: usize = 30000;

#[derive(Clone, PartialEq, Eq)]
pub struct Tape {
	// Pointer created by Vec::new(), then boxed.
	cells: NonNull<Cell>,
	ptr: TapePointer,
}

impl Tape {
	#[inline]
	#[must_use]
	pub fn new() -> Self {
		Self::try_new().unwrap()
	}

	pub fn try_new() -> Result<Self, LayoutError> {
		let layout = Layout::array::<Cell>(TAPE_SIZE)?;

		let ptr = unsafe {
			let raw = alloc::alloc::alloc_zeroed(layout);

			if raw.is_null() {
				alloc::alloc::handle_alloc_error(layout);
			}

			raw.cast::<Cell>()
		};

		let tape = Self {
			cells: unsafe { NonNull::new_unchecked(ptr) },
			ptr: unsafe { TapePointer::new_unchecked(0) },
		};

		Ok(tape)
	}

	#[must_use]
	pub fn cell(&self) -> &Cell {
		unsafe { &*self.cells.as_ptr().add(self.ptr().value()) }
	}

	pub fn cell_mut(&mut self) -> &mut Cell {
		unsafe { &mut *self.cells.as_ptr().add(self.ptr().value()) }
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
		unsafe { core::slice::from_raw_parts(self.cells.as_ptr(), TAPE_SIZE) }
	}

	pub const fn as_mut_slice(&mut self) -> &mut [Cell] {
		unsafe { core::slice::from_raw_parts_mut(self.cells.as_ptr(), TAPE_SIZE) }
	}

	#[must_use]
	pub const fn as_ptr(&self) -> *const Cell {
		self.cells.as_ptr()
	}

	#[expect(clippy::needless_pass_by_ref_mut)]
	pub const fn as_mut_ptr(&mut self) -> *mut Cell {
		self.cells.as_ptr()
	}
}

impl Debug for Tape {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Tape")
			.field("ptr", &self.ptr)
			.finish_non_exhaustive()
	}
}

impl Default for Tape {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for Tape {
	fn drop(&mut self) {
		let layout = Layout::array::<Cell>(TAPE_SIZE).unwrap();

		unsafe { alloc::alloc::dealloc(self.cells.as_ptr().cast(), layout) }
	}
}

impl Index<usize> for Tape {
	type Output = Cell;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output {
		&self.as_slice()[index % TAPE_SIZE]
	}
}

impl IndexMut<usize> for Tape {
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.as_mut_slice()[index % TAPE_SIZE]
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
