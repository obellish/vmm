#[cfg(feature = "nightly")]
use core::alloc::Allocator;
use core::{
	alloc::{GlobalAlloc, Layout},
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
	ops::Deref,
	ptr::NonNull,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{Align, Alignment, AllocChain, AllocError, ChainableAlloc, UnsafeStalloc};

#[repr(C)]
pub struct SyncStalloc<const L: usize, const B: usize>(Mutex<()>, UnsafeStalloc<L, B>)
where
	Align<B>: Alignment;

impl<const L: usize, const B: usize> SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	#[must_use]
	pub const fn new() -> Self {
		Self(Mutex::new(()), unsafe { UnsafeStalloc::new() })
	}

	pub fn is_oom(&self) -> bool {
		self.acquire_locked().is_oom()
	}

	pub fn is_empty(&self) -> bool {
		self.acquire_locked().is_empty()
	}

	pub unsafe fn clear(&self) {
		unsafe {
			self.acquire_locked().clear();
		}
	}

	pub unsafe fn allocate_blocks(
		&self,
		size: usize,
		align: usize,
	) -> Result<NonNull<u8>, AllocError> {
		unsafe { self.acquire_locked().allocate_blocks(size, align) }
	}

	pub unsafe fn deallocate_blocks(&self, ptr: NonNull<u8>, size: usize) {
		unsafe { self.acquire_locked().deallocate_blocks(ptr, size) };
	}

	pub unsafe fn shrink_in_place(&self, ptr: NonNull<u8>, old_size: usize, new_size: usize) {
		unsafe {
			self.acquire_locked()
				.shrink_in_place(ptr, old_size, new_size);
		}
	}

	pub unsafe fn grow_in_place(
		&self,
		ptr: NonNull<u8>,
		old_size: usize,
		new_size: usize,
	) -> Result<(), AllocError> {
		unsafe { self.acquire_locked().grow_in_place(ptr, old_size, new_size) }
	}

	pub unsafe fn grow_up_to(&self, ptr: NonNull<u8>, old_size: usize, new_size: usize) -> usize {
		unsafe { self.acquire_locked().grow_up_to(ptr, old_size, new_size) }
	}

	pub fn acquire_locked(&self) -> StallocGuard<'_, L, B> {
		StallocGuard {
			_guard: match self.0.lock() {
				Ok(l) => l,
				Err(e) => PoisonError::into_inner(e),
			},
			inner: &self.1,
			marker: PhantomData,
		}
	}

	pub const fn chain<T>(self, next: &T) -> AllocChain<'_, Self, T> {
		AllocChain::new(self, next)
	}
}

#[cfg(feature = "nightly")]
unsafe impl<const L: usize, const B: usize> Allocator for &SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		(&*self.acquire_locked()).allocate(layout)
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		unsafe { (&*self.acquire_locked()).deallocate(ptr, layout) };
	}

	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		(&*self.acquire_locked()).allocate_zeroed(layout)
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&*self.acquire_locked()).grow(ptr, old_layout, new_layout) }
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&*self.acquire_locked()).grow_zeroed(ptr, old_layout, new_layout) }
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe { (&*self.acquire_locked()).shrink(ptr, old_layout, new_layout) }
	}

	fn by_ref(&self) -> &Self
	where
		Self: Sized,
	{
		self
	}
}

unsafe impl<const L: usize, const B: usize> ChainableAlloc for SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn addr_in_bounds(&self, addr: usize) -> bool {
		self.1.addr_in_bounds(addr)
	}
}

impl<const L: usize, const B: usize> Debug for SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.acquire_locked().inner, f)
	}
}

impl<const L: usize, const B: usize> Default for SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	fn default() -> Self {
		Self::new()
	}
}

unsafe impl<const L: usize, const B: usize> GlobalAlloc for SyncStalloc<L, B>
where
	Align<B>: Alignment,
{
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		unsafe { self.acquire_locked().alloc(layout) }
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		unsafe {
			self.acquire_locked().dealloc(ptr, layout);
		}
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		unsafe { self.acquire_locked().alloc_zeroed(layout) }
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		unsafe { self.acquire_locked().realloc(ptr, layout, new_size) }
	}
}

pub struct StallocGuard<'a, const L: usize, const B: usize>
where
	Align<B>: Alignment,
{
	_guard: MutexGuard<'a, ()>,
	inner: &'a UnsafeStalloc<L, B>,
	marker: PhantomData<*const ()>,
}

impl<const L: usize, const B: usize> Deref for StallocGuard<'_, L, B>
where
	Align<B>: Alignment,
{
	type Target = UnsafeStalloc<L, B>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}
