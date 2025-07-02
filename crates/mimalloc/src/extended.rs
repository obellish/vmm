use core::ffi::c_void;

use super::MiMalloc;

#[expect(clippy::unused_self)]
impl MiMalloc {
	#[must_use]
	pub fn version(&self) -> u32 {
		unsafe { ffi::mi_version() as u32 }
	}

	#[must_use]
	pub unsafe fn usable_size(&self, ptr: *const u8) -> usize {
		unsafe { ffi::mi_usable_size(ptr.cast::<c_void>()) }
	}
}
