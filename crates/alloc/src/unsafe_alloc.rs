#[cfg(feature = "nightly")]
use core::alloc::{AllocError, Allocator};
use core::{
	alloc::{GlobalAlloc, Layout},
	fmt::{Debug, Formatter, Result as FmtResult},
	hint::assert_unchecked,
	ops::Deref,
	ptr::{self, NonNull},
};

use super::{Align, Alignment, AllocChain, ChainableAlloc, Stalloc};

#[repr(transparent)]
pub struct UnsafeStalloc<const L: usize, const B: usize>(Stalloc<L, B>)
where
	Align<B>: Alignment;

impl<const L: usize, const B: usize> UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	#[must_use]
	pub const unsafe fn new() -> Self {
		Self(Stalloc::new())
	}

	pub const fn chain<T>(self, next: &T) -> AllocChain<'_, Self, T>
	where
		Self: Sized,
	{
		AllocChain::new(self, next)
	}
}

#[cfg(feature = "nightly")]
unsafe impl<const L: usize, const B: usize> Allocator for &UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		(&self.0).allocate(layout)
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		unsafe {
			(&self.0).deallocate(ptr, layout);
		}
	}

	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		(&self.0).allocate_zeroed(layout)
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&self.0).grow(ptr, old_layout, new_layout) }
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&self.0).grow_zeroed(ptr, old_layout, new_layout) }
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&self.0).shrink(ptr, old_layout, new_layout) }
	}

	fn by_ref(&self) -> &Self
	where
		Self: Sized,
	{
		self
	}
}

unsafe impl<const L: usize, const B: usize> ChainableAlloc for UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn addr_in_bounds(&self, addr: usize) -> bool {
		self.0.addr_in_bounds(addr)
	}
}

impl<const L: usize, const B: usize> Deref for UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	type Target = Stalloc<L, B>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<const L: usize, const B: usize> Debug for UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}

unsafe impl<const L: usize, const B: usize> GlobalAlloc for UnsafeStalloc<L, B>
where
	Align<B>: Alignment,
{
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let size = layout.size().div_ceil(B);
		let align = layout.align().div_ceil(B);

		unsafe {
			self.allocate_blocks(size, align)
				.map_or_else(|_| ptr::null_mut(), |p| p.as_ptr().cast())
		}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let size = layout.size().div_ceil(B);

		unsafe {
			self.deallocate_blocks(NonNull::new_unchecked(ptr), size);
		}
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		let size = layout.size().div_ceil(B);

		let new = unsafe { self.alloc(layout) };
		if !new.is_null() {
			unsafe { ptr::write_bytes(new, 0, size * B) };
		}

		new
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		unsafe {
			assert_unchecked(new_size > 0);
		}

		let old_size = layout.size() / B;
		let new_size = new_size.div_ceil(B);

		unsafe {
			let ptr = NonNull::new_unchecked(ptr);

			if new_size > old_size && self.grow_in_place(ptr, old_size, new_size).is_ok() {
				return ptr.as_ptr();
			} else if new_size > old_size {
				let Ok(new) = self.allocate_blocks(new_size, layout.align()) else {
					return ptr::null_mut();
				};

				ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), layout.size());

				self.deallocate_blocks(ptr, old_size);

				return new.as_ptr();
			} else if old_size > new_size {
				self.shrink_in_place(ptr, old_size, new_size);
			}

			ptr.as_ptr()
		}
	}
}

unsafe impl<const L: usize, const B: usize> Sync for UnsafeStalloc<L, B> where Align<B>: Alignment {}
