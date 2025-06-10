#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod ptr;

use alloc::boxed::Box;
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::{Index, IndexMut},
};

use vmm_num::Wrapping;

pub use self::ptr::*;

pub const TAPE_SIZE: usize = 30000;

#[derive(Clone, PartialEq, Eq)]
pub struct Tape {
	// We use a custom allocator, so we put this on the heap
	cells: Box<[Wrapping<u8>; TAPE_SIZE]>,
	ptr: TapePointer,
}

impl Tape {
	#[must_use]
	pub fn new() -> Self {
		Self {
			cells: Box::new([Wrapping(0); TAPE_SIZE]),
			ptr: unsafe { TapePointer::new_unchecked(0) },
		}
	}

	#[must_use]
	pub fn cell(&self) -> &Wrapping<u8> {
		unsafe { self.cells.get_unchecked(self.ptr.value()) }
	}

	pub fn cell_mut(&mut self) -> &mut Wrapping<u8> {
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
	pub const fn as_slice(&self) -> &[Wrapping<u8>] {
		&*self.cells
	}

	pub const fn as_mut_slice(&mut self) -> &mut [Wrapping<u8>] {
		&mut *self.cells
	}

	#[must_use]
	pub const fn as_ptr(&self) -> *const Wrapping<u8> {
		self.cells.as_ptr()
	}

	pub const fn as_mut_ptr(&mut self) -> *mut Wrapping<u8> {
		self.cells.as_mut_ptr()
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
		&self.cells[index % TAPE_SIZE]
	}
}

impl IndexMut<usize> for Tape {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.cells[index % TAPE_SIZE]
	}
}
