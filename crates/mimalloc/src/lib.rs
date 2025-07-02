#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate libmimalloc_sys as ffi;

#[cfg(feature = "extended")]
mod extended;

use core::{
	alloc::{GlobalAlloc, Layout},
	ffi::c_void,
};

use ffi::{mi_free, mi_malloc_aligned, mi_realloc_aligned, mi_zalloc_aligned};

pub struct MiMalloc;

unsafe impl GlobalAlloc for MiMalloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		unsafe { mi_malloc_aligned(layout.size(), layout.align()).cast::<u8>() }
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
		unsafe { mi_free(ptr.cast::<c_void>()) };
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		unsafe { mi_zalloc_aligned(layout.size(), layout.align()).cast::<u8>() }
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		unsafe { mi_realloc_aligned(ptr.cast(), new_size, layout.align()).cast() }
	}
}

#[cfg(all(test, not(miri)))]
mod tests {
	use core::alloc::{GlobalAlloc, Layout, LayoutError};

	use super::*;

	#[test]
	fn frees_allocated_memory() -> Result<(), LayoutError> {
		unsafe {
			let layout = Layout::from_size_align(8, 8)?;

			let alloc = MiMalloc;

			let ptr = alloc.alloc(layout);

			assert!(!ptr.is_null());

			alloc.dealloc(ptr, layout);
		}

		Ok(())
	}

	#[test]
	fn frees_big_allocation() -> Result<(), LayoutError> {
		unsafe {
			let layout = Layout::from_size_align(1 << 20, 32)?;
			let alloc = MiMalloc;

			let ptr = alloc.alloc(layout);

			assert!(!ptr.is_null());

			alloc.dealloc(ptr, layout);
		}

		Ok(())
	}

	#[test]
	fn frees_zero_allocated_memory() -> Result<(), LayoutError> {
		unsafe {
			let layout = Layout::from_size_align(8, 8)?;
			let alloc = MiMalloc;

			let ptr = alloc.alloc_zeroed(layout);

			assert!(!ptr.is_null());

			alloc.dealloc(ptr, layout);
		}

		Ok(())
	}
}
