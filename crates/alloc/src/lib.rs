#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod align;
mod chain;
#[cfg(feature = "std")]
mod sync;
#[cfg(all(test, feature = "nightly", feature = "std"))]
mod tests;
mod unsafe_alloc;

#[cfg(feature = "nightly")]
use core::{
	alloc::{AllocError, Allocator, Layout},
	ptr,
};
use core::{
	cell::UnsafeCell,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hint::assert_unchecked,
	mem::MaybeUninit,
	ptr::NonNull,
};

#[cfg(feature = "std")]
pub use self::sync::*;
pub use self::{align::*, chain::*, unsafe_alloc::*};

const OOM_MARKER: u16 = u16::MAX;

#[cfg(not(feature = "nightly"))]
pub struct AllocError;

#[repr(C)]
pub struct Stalloc<const L: usize, const B: usize>
where
	Align<B>: Alignment,
{
	data: UnsafeCell<[Block<B>; L]>,
	base: UnsafeCell<Header>,
}

impl<const L: usize, const B: usize> Stalloc<L, B>
where
	Align<B>: Alignment,
{
	#[must_use]
	pub const fn new() -> Self {
		assert!(L >= 1 && L <= 0xffff, "block count must be in 1..65536");
		assert!(B >= 4, "block size must be at least 4 bytes");

		let mut blocks = [Block {
			bytes: const { [MaybeUninit::uninit(); B] },
		}; L];

		blocks[0].header = Header {
			next: 0,
			length: L as u16,
		};

		Self {
			data: UnsafeCell::new(blocks),
			base: UnsafeCell::new(Header { next: 0, length: 0 }),
		}
	}

	pub fn is_oom(&self) -> bool {
		matches!(unsafe { *self.base.get() }.length, OOM_MARKER)
	}

	pub fn is_empty(&self) -> bool {
		!self.is_oom() && matches!(unsafe { *self.base.get() }.next, 0)
	}

	pub unsafe fn clear(&self) {
		unsafe {
			(*self.base.get()).next = 0;
			(*self.base.get()).length = 0;
			(*self.header_at(0)).next = 0;
			(*self.header_at(0)).length = L as u16;
		}
	}

	pub unsafe fn allocate_blocks(
		&self,
		size: usize,
		align: usize,
	) -> Result<NonNull<u8>, AllocError> {
		unsafe {
			assert_unchecked(size >= 1 && align.is_power_of_two() && align < 2usize.pow(29) / B);
		}

		if self.is_oom() {
			return Err(AllocError);
		}

		unsafe {
			let base = self.base.get();
			let mut prev = base;
			let mut curr = self.header_at((*base).next as usize);

			loop {
				let curr_idx = (*prev).next as usize;
				let next_idx = (*curr).next as usize;

				let curr_chunk_length = (*curr).length as usize;

				let spare_front = (curr.addr() / B).wrapping_neg() % align;

				if spare_front + size <= curr_chunk_length {
					let avail_blocks = curr_chunk_length - spare_front;
					let avail_blocks_ptr = self.block_at(curr_idx + spare_front);
					let spare_back = avail_blocks - size;

					if spare_back > 0 {
						let spare_back_idx = curr_idx + spare_front + size;
						let spare_back_ptr = self.header_at(spare_back_idx);
						(*spare_back_ptr).next = next_idx as u16;
						(*spare_back_ptr).length = spare_back as u16;

						if spare_front > 0 {
							(*curr).next = spare_back_idx as u16;
							(*curr).length = spare_front as u16;
						} else {
							(*prev).next = spare_back_idx as u16;
						}
					} else if spare_front > 0 {
						(*curr).next = (curr_idx + spare_front + size) as u16;
						(*curr).length = spare_front as u16;
						(*prev).next = next_idx as u16;
					} else {
						(*prev).next = next_idx as u16;

						if matches!(next_idx, 0) {
							(*base).length = OOM_MARKER;
						}
					}

					return Ok(NonNull::new_unchecked(avail_blocks_ptr.cast()));
				}

				if matches!(next_idx, 0) {
					return Err(AllocError);
				}

				prev = curr;
				curr = self.header_at(next_idx);
			}
		}
	}

	pub unsafe fn deallocate_blocks(&self, ptr: NonNull<u8>, size: usize) {
		unsafe {
			assert_unchecked(size >= 1 && size <= L);
		}

		let freed_ptr = header_in_block(ptr.as_ptr().cast());
		let freed_idx = self.index_of(freed_ptr);
		let base = self.base.get();
		let before = self.header_before(freed_idx);

		unsafe {
			let prev_next = (*before).next as usize;
			(*freed_ptr).next = prev_next as u16;
			(*freed_ptr).length = size as u16;

			if freed_idx + size == prev_next {
				let header_to_merge = self.header_at(prev_next);
				(*freed_ptr).next = (*header_to_merge).next;
				(*freed_ptr).length += (*header_to_merge).length;
			}

			if before.eq(&base) {
				(*base).next = freed_idx as u16;
				(*base).length = 0;
			} else if self.index_of(before) + (*before).length as usize == freed_idx {
				(*before).next = (*freed_ptr).next;
				(*before).length += (*freed_ptr).length;
			} else {
				(*before).next = freed_idx as u16;
			}
		}
	}

	pub unsafe fn shrink_in_place(&self, ptr: NonNull<u8>, old_size: usize, new_size: usize) {
		unsafe {
			assert_unchecked(new_size > 0 && new_size < old_size);
		}

		let curr_block: *mut Block<B> = ptr.as_ptr().cast();
		let curr_idx = (curr_block.addr() - self.data.get().addr()) / B;

		let new_idx = curr_idx + new_size;
		let spare_blocks = old_size - new_size;

		unsafe {
			let prev_free_chunk = self.header_before(curr_idx);

			let next_free_idx = (*prev_free_chunk).next as usize;
			let new_chunk = header_in_block(curr_block.add(new_size));

			(*prev_free_chunk).next = new_idx as u16;

			if new_idx + spare_blocks == next_free_idx {
				let next_free_chunk = self.header_at(next_free_idx);
				(*new_chunk).next = (*next_free_chunk).next;
				(*new_chunk).length = spare_blocks as u16 + (*next_free_chunk).length;
			} else {
				(*new_chunk).next = next_free_idx as u16;
				(*new_chunk).length = spare_blocks as u16;
			}

			(*self.base.get()).length = 0;
		}
	}

	pub unsafe fn grow_in_place(
		&self,
		ptr: NonNull<u8>,
		old_size: usize,
		new_size: usize,
	) -> Result<(), AllocError> {
		unsafe {
			assert_unchecked(old_size >= 1 && old_size <= L && new_size > old_size);
		}

		let curr_block: *mut Block<B> = ptr.as_ptr().cast();
		let curr_idx = (curr_block.addr() - self.data.get().addr()) / B;
		let prev_free_chunk = self.header_before(curr_idx);

		unsafe {
			let next_free_idx = (*prev_free_chunk).next as usize;

			if curr_idx + old_size != next_free_idx {
				return Err(AllocError);
			}

			let next_free_chunk = self.header_at(next_free_idx);
			let room_to_grow = (*next_free_chunk).length as usize;

			let needed_blocks = new_size - old_size;
			if needed_blocks > room_to_grow {
				return Err(AllocError);
			}

			let blocks_left_over = room_to_grow - needed_blocks;

			if blocks_left_over > 0 {
				let new_chunk_idx = next_free_idx + needed_blocks;
				let new_chunk_head = self.header_at(new_chunk_idx);

				(*prev_free_chunk).next = new_chunk_idx as u16;
				(*new_chunk_head).next = (*next_free_chunk).next;
				(*new_chunk_head).length = blocks_left_over as u16;
			} else {
				(*prev_free_chunk).next = (*next_free_chunk).next;

				let base = self.base.get();
				if prev_free_chunk.eq(&base) && matches!((*next_free_chunk).next, 0) {
					(*base).length = OOM_MARKER;
				}
			}

			Ok(())
		}
	}

	pub unsafe fn grow_up_to(&self, ptr: NonNull<u8>, old_size: usize, new_size: usize) -> usize {
		unsafe {
			assert_unchecked(old_size >= 1 && old_size <= L && new_size > old_size);
		}

		let curr_block: *mut Block<B> = ptr.as_ptr().cast();
		let curr_idx = (curr_block.addr() - self.data.get().addr()) / B;
		let prev_free_chunk = self.header_before(curr_idx);

		unsafe {
			let next_free_idx = (*prev_free_chunk).next as usize;

			if curr_idx + old_size != next_free_idx {
				return old_size;
			}

			let next_free_chunk = self.header_at(next_free_idx);
			let room_to_grow = (*next_free_chunk).length as usize;

			let needed_blocks = (new_size - old_size).min(room_to_grow);

			let blocks_left_over = room_to_grow - needed_blocks;

			if blocks_left_over > 0 {
				let new_chunk_idx = next_free_idx + needed_blocks;
				let new_chunk_head = self.header_at(new_chunk_idx);

				(*prev_free_chunk).next = new_chunk_idx as u16;
				(*new_chunk_head).next = (*next_free_chunk).next;
				(*new_chunk_head).length = blocks_left_over as u16;
			} else {
				(*prev_free_chunk).next = (*next_free_chunk).next;

				let base = self.base.get();
				if prev_free_chunk.eq(&base) && matches!((*next_free_chunk).next, 0) {
					(*base).length = OOM_MARKER;
				}
			}

			old_size + needed_blocks
		}
	}

	fn index_of(&self, ptr: *mut Header) -> usize {
		(ptr.addr() - self.data.get().addr()) / B
	}

	const unsafe fn block_at(&self, idx: usize) -> *mut Block<B> {
		let root: *mut Block<B> = self.data.get().cast();
		unsafe { root.add(idx) }
	}

	unsafe fn header_at(&self, idx: usize) -> *mut Header {
		header_in_block(unsafe { self.block_at(idx) })
	}

	fn header_before(&self, idx: usize) -> *mut Header {
		let mut ptr = self.base.get();

		unsafe {
			if matches!((*ptr).length, OOM_MARKER) || (*ptr).next as usize >= idx {
				return ptr;
			}

			loop {
				ptr = self.header_at((*ptr).next as usize);
				let next_idx = (*ptr).next as usize;
				if matches!(next_idx, 0) || next_idx >= idx {
					return ptr;
				}
			}
		}
	}

	pub const fn chain<T>(self, next: &T) -> AllocChain<'_, Self, T> {
		AllocChain::new(self, next)
	}
}

#[cfg(feature = "nightly")]
unsafe impl<const L: usize, const B: usize> Allocator for &Stalloc<L, B>
where
	Align<B>: Alignment,
{
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		let size = layout.size().div_ceil(B);
		let align = layout.align().div_ceil(B);

		if matches!(size, 0) {
			let dangling = NonNull::new(layout.align() as _).unwrap();
			return Ok(NonNull::slice_from_raw_parts(dangling, 0));
		}

		unsafe { self.allocate_blocks(size, align) }
			.map(|p| NonNull::slice_from_raw_parts(p, size * B))
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		let size = layout.size().div_ceil(B);

		if matches!(size, 0) {
			return;
		}

		unsafe {
			self.deallocate_blocks(ptr, size);
		}
	}

	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		let ptr = self.allocate(layout)?;

		let ptr = NonNull::slice_from_raw_parts(ptr.cast(), layout.size());

		unsafe { ptr.cast::<u8>().write_bytes(0, ptr.len()) };
		Ok(ptr)
	}

	unsafe fn grow(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		let old_size = old_layout.size().div_ceil(B);
		let new_size = new_layout.size().div_ceil(B);
		let align = new_layout.align().div_ceil(B);

		if new_size == old_size {
			return Ok(NonNull::slice_from_raw_parts(ptr, new_size * B));
		}

		if matches!(old_size, 0) {
			return unsafe {
				self.allocate_blocks(new_size, align)
					.map(|p| NonNull::slice_from_raw_parts(p, new_size * B))
			};
		}

		unsafe {
			if self.grow_in_place(ptr, old_size, new_size).is_ok() {
				Ok(NonNull::slice_from_raw_parts(ptr, new_size * B))
			} else {
				let new = self.allocate_blocks(new_size, align)?;

				ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr().cast(), old_layout.size());

				self.deallocate_blocks(ptr, old_size);

				Ok(NonNull::slice_from_raw_parts(new, new_size * B))
			}
		}
	}

	unsafe fn grow_zeroed(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		unsafe {
			let new_ptr = self.grow(ptr, old_layout, new_layout)?;
			let count = new_ptr.len() - old_layout.size();

			new_ptr
				.cast::<u8>()
				.add(old_layout.size())
				.write_bytes(0, count);

			Ok(new_ptr)
		}
	}

	unsafe fn shrink(
		&self,
		ptr: NonNull<u8>,
		old_layout: Layout,
		new_layout: Layout,
	) -> Result<NonNull<[u8]>, AllocError> {
		let old_size = old_layout.size().div_ceil(B);
		let new_size = new_layout.size().div_ceil(B);

		if matches!(new_size, 0) {
			unsafe {
				if !matches!(old_size, 0) {
					self.deallocate_blocks(ptr, old_size);
				}

				let dangling = NonNull::new_unchecked(new_layout.align() as _);

				return Ok(NonNull::slice_from_raw_parts(dangling, 0));
			}
		}

		if !matches!(ptr.as_ptr().addr() % new_layout.align(), 0) {
			let align = new_layout.align() / B;

			unsafe {
				let new = self.allocate_blocks(new_size, align)?;

				ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), old_layout.size());

				self.deallocate_blocks(ptr, old_size);

				return Ok(NonNull::slice_from_raw_parts(new, new_size * B));
			}
		}

		if old_size == new_size {
			return Ok(NonNull::slice_from_raw_parts(ptr, old_size * B));
		}

		unsafe {
			self.shrink_in_place(ptr, old_size, new_size);
		}

		Ok(NonNull::slice_from_raw_parts(ptr, new_size * B))
	}
}

unsafe impl<const L: usize, const B: usize> ChainableAlloc for Stalloc<L, B>
where
	Align<B>: Alignment,
{
	fn addr_in_bounds(&self, addr: usize) -> bool {
		addr >= self.data.get().addr() && addr < self.data.get().addr() + B * L
	}
}

impl<const L: usize, const B: usize> Default for Stalloc<L, B>
where
	Align<B>: Alignment,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<const L: usize, const B: usize> Debug for Stalloc<L, B>
where
	Align<B>: Alignment,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Stallocator with ")?;
		Display::fmt(&L, f)?;
		f.write_str(" blocks of ")?;
		Display::fmt(&B, f)?;
		f.write_str(" bytes each")?;

		let mut ptr = self.base.get();
		if matches!(unsafe { (*ptr).length }, OOM_MARKER) {
			f.write_str("\n\tNo free blocks (OOM)")?;
			return Ok(());
		}

		loop {
			unsafe {
				let idx = (*ptr).next as usize;
				ptr = self.header_at(idx);

				let length = (*ptr).length as usize;

				f.write_str("\n\tindex ")?;
				Display::fmt(&idx, f)?;
				f.write_str(": ")?;
				Display::fmt(&length, f)?;
				f.write_str(" free block")?;

				if length > 1 {
					f.write_char('s')?;
				}
				// if matches!(length, 1) {
				// 	f.write_str(" free block")?;
				// } else {
				// 	f.write_str(" free blocks")?;
				// }

				if matches!((*ptr).next, 0) {
					return Ok(());
				}
			}
		}
	}
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Header {
	next: u16,
	length: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
union Block<const B: usize>
where
	Align<B>: Alignment,
{
	header: Header,
	bytes: [MaybeUninit<u8>; B],
	_align: Align<B>,
}

fn header_in_block<const B: usize>(ptr: *mut Block<B>) -> *mut Header
where
	Align<B>: Alignment,
{
	unsafe { &raw mut (*ptr).header }
}
