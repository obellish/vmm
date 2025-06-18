use core::{
	alloc::{Layout, LayoutError},
	ops::{Index, IndexMut},
	ptr::NonNull,
	slice,
};

use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct PtrTape {
	cells: NonNull<Cell>,
	ptr: TapePointer,
}

impl PtrTape {
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

		Ok(Self {
			cells: unsafe { NonNull::new_unchecked(ptr) },
			ptr: unsafe { TapePointer::new_unchecked(0) },
		})
	}
}

impl Default for PtrTape {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for PtrTape {
	fn drop(&mut self) {
		let layout = Layout::array::<Cell>(TAPE_SIZE).unwrap();

		unsafe { alloc::alloc::dealloc(self.cells.as_ptr().cast(), layout) }
	}
}

impl Tape for PtrTape {
	fn as_slice(&self) -> &[Cell] {
		unsafe { slice::from_raw_parts(self.cells.as_ptr(), TAPE_SIZE) }
	}

	fn as_mut_slice(&mut self) -> &mut [Cell] {
		unsafe { slice::from_raw_parts_mut(self.cells.as_ptr(), TAPE_SIZE) }
	}

	fn ptr(&self) -> &TapePointer {
		&self.ptr
	}

	fn ptr_mut(&mut self) -> &mut TapePointer {
		&mut self.ptr
	}

	fn current_cell(&self) -> &Cell {
		unsafe { &*self.cells.as_ptr().add(self.ptr().value()) }
	}

	fn current_cell_mut(&mut self) -> &mut Cell {
		unsafe { &mut *self.cells.as_ptr().add(self.ptr().value()) }
	}

	unsafe fn current_cell_unchecked(&self) -> &Cell {
		self.current_cell()
	}

	unsafe fn current_cell_unchecked_mut(&mut self) -> &mut Cell {
		self.current_cell_mut()
	}
}

impl Index<usize> for PtrTape {
	type Output = Cell;

	fn index(&self, index: usize) -> &Self::Output {
		&self.as_slice()[index % TAPE_SIZE]
	}
}

impl IndexMut<usize> for PtrTape {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.as_mut_slice()[index % TAPE_SIZE]
	}
}
