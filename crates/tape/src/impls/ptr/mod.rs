mod unique;

use core::{
	alloc::{Layout, LayoutError},
	slice,
};

use self::unique::Unique;
use crate::{Cell, TAPE_SIZE, Tape, TapePointer};

#[derive(Clone, PartialEq, Eq)]
pub struct PtrTape {
	cells: Unique<Cell>,
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
			let raw = alloc::alloc::alloc(layout);

			if raw.is_null() {
				alloc::alloc::handle_alloc_error(layout);
			}

			let p = Unique::new_unchecked(raw.cast::<Cell>());

			(0..TAPE_SIZE).for_each(|i| p.add(i).write(Cell::with_index(0, i)));
			p
		};

		Ok(Self {
			cells: ptr,
			ptr: TapePointer::zero(),
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

unsafe impl Send for PtrTape {}
unsafe impl Sync for PtrTape {}

impl Tape for PtrTape {
	fn init(&mut self) {}

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
}
