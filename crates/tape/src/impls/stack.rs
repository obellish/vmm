#![allow(clippy::large_stack_frames)]

use core::{
	ops::{Deref, DerefMut},
	pin::Pin,
};

use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct StackTape {
	cells: Pin<Stack>,
	ptr: TapePointer,
}

impl StackTape {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			cells: Pin::new(Stack::new()),
			ptr: TapePointer::zero(),
		}
	}
}

impl Default for StackTape {
	fn default() -> Self {
		Self::new()
	}
}

impl Tape for StackTape {
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
}

#[derive(Clone, PartialEq, Eq)]
pub struct Stack {
	inner: [Cell; TAPE_SIZE],
}

impl Stack {
	const fn new() -> Self {
		Self {
			inner: [Cell::new(0); TAPE_SIZE],
		}
	}
}

impl Deref for Stack {
	type Target = [Cell; TAPE_SIZE];

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Stack {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
