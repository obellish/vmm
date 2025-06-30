#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod cell;
mod impls;
mod ptr;

use alloc::boxed::Box;
use core::ops::IndexMut;

pub use self::{cell::*, impls::*, ptr::*};

pub const TAPE_SIZE: usize = 5000;

pub trait Tape: Default {
	/// Initialize the tape, setting all cells indices and values.
	fn init(&mut self) {
		for (i, cell) in self.as_mut_slice().iter_mut().enumerate() {
			cell.set_value(0);

			cell.set_index(i);
		}
	}

	fn as_slice(&self) -> &[Cell];

	fn as_mut_slice(&mut self) -> &mut [Cell];

	fn ptr(&self) -> &TapePointer;

	fn ptr_mut(&mut self) -> &mut TapePointer;

	fn get(&self, ptr: usize) -> &Cell {
		self.as_slice().get(ptr).unwrap()
	}

	fn get_mut(&mut self, ptr: usize) -> &mut Cell {
		self.as_mut_slice().get_mut(ptr).unwrap()
	}

	fn current_cell(&self) -> &Cell {
		self.get(self.ptr().value())
	}

	fn current_cell_mut(&mut self) -> &mut Cell {
		self.get_mut(self.ptr().value())
	}

	unsafe fn current_cell_unchecked(&self) -> &Cell {
		unsafe { self.get_unchecked(self.ptr().value()) }
	}

	unsafe fn current_cell_unchecked_mut(&mut self) -> &mut Cell {
		unsafe { self.get_unchecked_mut(self.ptr().value()) }
	}

	unsafe fn get_unchecked(&self, ptr: usize) -> &Cell {
		unsafe { self.as_slice().get_unchecked(ptr) }
	}

	unsafe fn get_unchecked_mut(&mut self, ptr: usize) -> &mut Cell {
		unsafe { self.as_mut_slice().get_unchecked_mut(ptr) }
	}
}

impl<T: Tape> Tape for Box<T>
where
	Self: IndexMut<usize, Output = Cell>,
{
	fn init(&mut self) {
		(**self).init();
	}

	fn as_slice(&self) -> &[Cell] {
		(**self).as_slice()
	}

	fn as_mut_slice(&mut self) -> &mut [Cell] {
		(**self).as_mut_slice()
	}

	fn ptr(&self) -> &TapePointer {
		(**self).ptr()
	}

	fn ptr_mut(&mut self) -> &mut TapePointer {
		(**self).ptr_mut()
	}

	fn current_cell(&self) -> &Cell {
		(**self).current_cell()
	}

	fn current_cell_mut(&mut self) -> &mut Cell {
		(**self).current_cell_mut()
	}

	unsafe fn current_cell_unchecked(&self) -> &Cell {
		unsafe { (**self).current_cell_unchecked() }
	}

	unsafe fn current_cell_unchecked_mut(&mut self) -> &mut Cell {
		unsafe { (**self).current_cell_unchecked_mut() }
	}
}
