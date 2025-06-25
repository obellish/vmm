#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(feature = "nightly", feature(min_specialization, dropck_eyepatch))]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod drain;
mod extract_if;
mod into_iter;
mod serde;
mod splice;
#[cfg(test)]
mod tests;

use alloc::{alloc::Layout, boxed::Box, vec, vec::Vec};
use core::{
	borrow::{Borrow, BorrowMut},
	cmp::Ordering,
	error::Error as CoreError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	marker::PhantomData,
	mem::{self, ManuallyDrop, MaybeUninit},
	ops::{Bound, Deref, DerefMut, RangeBounds},
	ptr::{self, NonNull},
};

#[cfg(feature = "bytes")]
use bytes::{BufMut, buf::UninitSlice};
use vmm_utils::InsertOrPush;

pub use self::{drain::*, extract_if::*, into_iter::*, splice::*};

#[macro_export]
macro_rules! smallvec {
	(@one $x:expr) => (1usize);
	() => ($crate::SmallVec::new());
	($elem:expr; $n:expr) => ({
		$crate::SmallVec::from_elem($elem, $n)
	});
	($($x:expr),+$(,)?) => ({
		let count = 0usize $(+ $crate::smallvec!(@one $x))+;
		let mut vec = $crate::SmallVec::new();
		if count <= vec.capacity() {
			$(vec.push($x);)*
			vec
		} else {
			$crate::SmallVec::from_vec($crate::alloc::vec![$($x,)+])
		}
	})
}

#[repr(C)]
pub struct SmallVec<T, const N: usize> {
	len: TaggedLen,
	raw: RawSmallVec<T, N>,
	marker: PhantomData<fn() -> T>,
}

impl<T, const N: usize> SmallVec<T, N> {
	#[inline]
	const fn is_zst() -> bool {
		matches!(mem::size_of::<T>(), 0)
	}

	#[inline]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			len: TaggedLen::new(0, false, Self::is_zst()),
			raw: RawSmallVec::new(),
			marker: PhantomData,
		}
	}

	#[inline]
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		let mut this = Self::new();
		if capacity > Self::inline_size() {
			this.grow(capacity);
		}

		this
	}

	#[inline]
	pub const fn from_buf<const S: usize>(elements: [T; S]) -> Self {
		const {
			assert!(S <= N);
		}

		let mut buf: MaybeUninit<[T; N]> = MaybeUninit::uninit();

		unsafe {
			ptr::copy_nonoverlapping(elements.as_ptr(), buf.as_mut_ptr().cast::<T>(), S);
		}

		mem::forget(elements);

		Self {
			len: TaggedLen::new(S, false, Self::is_zst()),
			raw: RawSmallVec::new_inline(buf),
			marker: PhantomData,
		}
	}

	#[inline]
	pub fn from_buf_and_len(buf: [T; N], len: usize) -> Self {
		assert!(len <= N);

		let mut vec = Self {
			len: TaggedLen::new(len, false, Self::is_zst()),
			raw: RawSmallVec::new_inline(MaybeUninit::new(buf)),
			marker: PhantomData,
		};

		unsafe {
			let remainder_ptr = vec.raw.as_mut_inline_ptr().add(len);
			let remainder_len = N - len;

			ptr::drop_in_place(ptr::slice_from_raw_parts_mut(remainder_ptr, remainder_len));
		}

		vec
	}

	#[inline]
	pub const unsafe fn from_buf_and_len_unchecked(buf: MaybeUninit<[T; N]>, len: usize) -> Self {
		debug_assert!(len <= N);
		Self {
			len: TaggedLen::new(len, false, Self::is_zst()),
			raw: RawSmallVec::new_inline(buf),
			marker: PhantomData,
		}
	}

	#[inline]
	#[must_use]
	pub fn from_vec(vec: Vec<T>) -> Self {
		if matches!(vec.capacity(), 0) {
			return Self::new();
		}

		if Self::is_zst() {
			let mut vec = vec;
			let len = vec.len();

			unsafe {
				vec.set_len(0);
			}
			Self {
				len: TaggedLen::new(len, false, Self::is_zst()),
				raw: RawSmallVec::new(),
				marker: PhantomData,
			}
		} else {
			let mut vec = ManuallyDrop::new(vec);
			let len = vec.len();
			let cap = vec.capacity();

			let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };

			Self {
				len: TaggedLen::new(len, true, Self::is_zst()),
				raw: RawSmallVec::new_heap(ptr, cap),
				marker: PhantomData,
			}
		}
	}

	#[inline]
	const unsafe fn set_on_heap(&mut self) {
		self.len = TaggedLen::new(self.len(), true, Self::is_zst());
	}

	#[inline]
	const unsafe fn set_inline(&mut self) {
		self.len = TaggedLen::new(self.len(), false, Self::is_zst());
	}

	#[inline]
	pub unsafe fn set_len(&mut self, new_len: usize) {
		debug_assert!(new_len <= self.capacity());
		let on_heap = self.len.on_heap(Self::is_zst());
		self.len = TaggedLen::new(new_len, on_heap, Self::is_zst());
	}

	#[inline]
	#[must_use]
	pub const fn inline_size() -> usize {
		if Self::is_zst() { usize::MAX } else { N }
	}

	#[inline]
	pub const fn len(&self) -> usize {
		self.len.value(Self::is_zst())
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		matches!(self.len(), 0)
	}

	#[inline]
	pub const fn capacity(&self) -> usize {
		if self.len.on_heap(Self::is_zst()) {
			unsafe { self.raw.heap.1 }
		} else {
			Self::inline_size()
		}
	}

	#[inline]
	pub const fn spilled(&self) -> bool {
		self.len.on_heap(Self::is_zst())
	}

	#[inline]
	pub fn grow(&mut self, new_capacity: usize) {
		infallible(self.try_grow(new_capacity));
	}

	#[inline]
	#[must_use]
	pub fn split_off(&mut self, at: usize) -> Self {
		let len = self.len();
		assert!(at <= len);

		let other_len = len - at;
		let mut other = Self::with_capacity(other_len);

		unsafe {
			self.set_len(at);
			other.set_len(other_len);

			ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other_len);
		}

		other
	}

	#[cold]
	pub fn try_grow(&mut self, new_capacity: usize) -> Result<(), CollectionAllocError> {
		if Self::is_zst() {
			return Ok(());
		}

		let len = self.len();
		assert!(new_capacity >= len);

		if new_capacity > Self::inline_size() {
			let result = unsafe { self.raw.try_grow_raw(self.len, new_capacity) };

			if result.is_ok() {
				unsafe {
					self.set_on_heap();
				}
			}
			result
		} else {
			if self.spilled() {
				unsafe {
					let (ptr, old_cap) = self.raw.heap;

					ptr::copy_nonoverlapping(ptr.as_ptr(), self.raw.as_mut_inline_ptr(), len);
					drop(DropDealloc {
						ptr,
						capacity: old_cap,
					});
					self.set_inline();
				}
			}

			Ok(())
		}
	}

	#[inline]
	pub const fn as_ptr(&self) -> *const T {
		if self.len.on_heap(Self::is_zst()) {
			unsafe { self.raw.as_heap_ptr() }
		} else {
			self.raw.as_inline_ptr()
		}
	}

	#[inline]
	pub const fn as_mut_ptr(&mut self) -> *mut T {
		if self.len.on_heap(Self::is_zst()) {
			unsafe { self.raw.as_mut_heap_ptr() }
		} else {
			self.raw.as_mut_inline_ptr()
		}
	}

	#[inline]
	pub const fn as_slice(&self) -> &[T] {
		let len = self.len();
		let ptr = self.as_ptr();
		unsafe { core::slice::from_raw_parts(ptr, len) }
	}

	#[inline]
	pub const fn as_mut_slice(&mut self) -> &mut [T] {
		let len = self.len();
		let ptr = self.as_mut_ptr();
		unsafe { core::slice::from_raw_parts_mut(ptr, len) }
	}

	#[inline]
	pub fn truncate(&mut self, len: usize) {
		let old_len = self.len();
		if len < old_len {
			unsafe {
				self.set_len(len);
				ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
					self.as_mut_ptr().add(len),
					old_len - len,
				));
			}
		}
	}

	#[inline]
	pub fn reserve(&mut self, additional: usize) {
		infallible(self.try_reserve(additional));
	}

	#[inline]
	pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocError> {
		if additional > self.capacity() - self.len() {
			let new_capacity = self
				.len()
				.checked_add(additional)
				.and_then(usize::checked_next_power_of_two)
				.ok_or(CollectionAllocError::CapacityOverflow)?;
			self.try_grow(new_capacity)
		} else {
			Ok(())
		}
	}

	#[inline]
	pub fn reserve_exact(&mut self, additional: usize) {
		infallible(self.try_reserve_exact(additional));
	}

	#[inline]
	pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), CollectionAllocError> {
		if additional > self.capacity() - self.len() {
			let new_capacity = self
				.len()
				.checked_add(additional)
				.ok_or(CollectionAllocError::CapacityOverflow)?;
			self.try_grow(new_capacity)
		} else {
			Ok(())
		}
	}

	#[inline]
	pub fn shrink_to_fit(&mut self) {
		if !self.spilled() {
			return;
		}

		let len = self.len();
		if len <= Self::inline_size() {
			unsafe {
				let (ptr, capacity) = self.raw.heap;
				self.raw = RawSmallVec::new_inline(MaybeUninit::uninit());
				ptr::copy_nonoverlapping(ptr.as_ptr(), self.raw.as_mut_inline_ptr(), len);
				self.set_inline();
				alloc::alloc::dealloc(
					ptr.cast().as_ptr(),
					Layout::from_size_align_unchecked(
						capacity * mem::size_of::<T>(),
						mem::align_of::<T>(),
					),
				);
			}
		} else if len < self.capacity() {
			unsafe { infallible(self.raw.try_grow_raw(self.len, len)) }
		}
	}

	#[inline]
	pub fn shrink_to(&mut self, min_capacity: usize) {
		if !self.spilled() {
			return;
		}

		if self.capacity() > min_capacity {
			let len = self.len();
			let target = len.max(min_capacity);
			if target <= Self::inline_size() {
				unsafe {
					let (ptr, capacity) = self.raw.heap;
					self.raw = RawSmallVec::new_inline(MaybeUninit::uninit());
					self.set_inline();
					alloc::alloc::dealloc(
						ptr.cast().as_ptr(),
						Layout::from_size_align_unchecked(
							capacity * mem::size_of::<T>(),
							mem::align_of::<T>(),
						),
					);
				}
			} else if target < self.capacity() {
				unsafe { infallible(self.raw.try_grow_raw(self.len, target)) }
			}
		}
	}

	pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<'_, T, N> {
		let len = self.len();
		let start = match range.start_bound() {
			Bound::Included(&n) => n,
			Bound::Excluded(&n) => n.checked_add(1).expect("range start out of bounds"),
			Bound::Unbounded => 0,
		};

		let end = match range.end_bound() {
			Bound::Included(&n) => n.checked_add(1).expect("range end out of bounds"),
			Bound::Excluded(&n) => n,
			Bound::Unbounded => len,
		};

		assert!(start <= end);
		assert!(end <= len);

		unsafe {
			self.set_len(start);

			let range_slice = core::slice::from_raw_parts(self.as_ptr().add(start), end - start);

			Drain {
				tail_len: len - end,
				tail_start: end,
				iter: range_slice.iter(),
				// Since self is &mut, passing it to a function would invalidate the slice iterator.
				#[allow(clippy::ref_as_ptr)]
				vec: NonNull::new_unchecked(self as *mut _),
			}
		}
	}

	pub fn splice<I>(
		&mut self,
		range: impl RangeBounds<usize>,
		replace_with: I,
	) -> Splice<'_, I::IntoIter, N>
	where
		I: IntoIterator<Item = T>,
	{
		Splice {
			drain: self.drain(range),
			replace_with: replace_with.into_iter(),
		}
	}

	pub fn extract_if<F>(
		&mut self,
		range: impl RangeBounds<usize>,
		filter: F,
	) -> ExtractIf<'_, T, N, F>
	where
		F: FnMut(&mut T) -> bool,
	{
		let old_len = self.len();
		let (start, end) = {
			let len = old_len;

			let start = match range.start_bound() {
				Bound::Included(&start) => start,
				Bound::Excluded(start) => start
					.checked_add(1)
					.expect("attempted to index slice from after maximum usize"),
				Bound::Unbounded => 0,
			};

			let end = match range.end_bound() {
				Bound::Included(end) => end
					.checked_add(1)
					.expect("attempted to index slice up to maximum usize"),
				Bound::Excluded(&end) => end,
				Bound::Unbounded => len,
			};

			assert!(
				(start <= end),
				"slice index starts at {start} but ends at {end}"
			);

			assert!(
				(end <= len),
				"range end index {end} out of range for slice of length {len}"
			);

			(start, end)
		};

		unsafe {
			self.set_len(0);
		}

		ExtractIf {
			vec: self,
			idx: start,
			end,
			del: 0,
			old_len,
			pred: filter,
		}
	}

	#[inline]
	pub fn push(&mut self, value: T) {
		let len = self.len();
		if len == self.capacity() {
			self.reserve(1);
		}

		let ptr = unsafe { self.as_mut_ptr().add(len) };

		unsafe {
			ptr.write(value);
			self.set_len(len + 1);
		}
	}

	#[inline]
	pub fn pop(&mut self) -> Option<T> {
		if self.is_empty() {
			None
		} else {
			let len = self.len() - 1;
			unsafe {
				self.set_len(len);
				let value = self.as_mut_ptr().add(len).read();
				Some(value)
			}
		}
	}

	#[inline]
	pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
		let last = self.last_mut()?;
		if predicate(last) { self.pop() } else { None }
	}

	#[inline]
	pub fn append<const M: usize>(&mut self, other: &mut SmallVec<T, M>) {
		let len = self.len();
		let other_len = other.len();
		let total_len = len + other_len;
		if total_len > self.capacity() {
			self.reserve(other_len);
		}

		unsafe {
			let ptr = self.as_mut_ptr().add(len);
			other.set_len(0);

			ptr::copy_nonoverlapping(other.as_ptr(), ptr, other_len);
			self.set_len(total_len);
		}
	}

	#[inline]
	pub fn swap_remove(&mut self, index: usize) -> T {
		let len = self.len();
		assert!(
			index < len,
			"swap_remove index (is {index}) should be < len (is {len})"
		);
		let new_len = len - 1;
		unsafe {
			let value = ptr::read(self.as_ptr().add(index));
			let base_ptr = self.as_mut_ptr();
			ptr::copy(base_ptr.add(new_len), base_ptr.add(index), 1);
			self.set_len(new_len);
			value
		}
	}

	#[inline]
	pub fn clear(&mut self) {
		unsafe {
			let old_len = self.len();
			self.set_len(0);
			ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), old_len));
		}
	}

	#[inline]
	pub fn remove(&mut self, index: usize) -> T {
		let len = self.len();
		assert!(
			index < len,
			"removal index (is {index}) should be < len (is {len})"
		);
		let new_len = len - 1;
		unsafe {
			self.set_len(new_len);
			let ptr = self.as_mut_ptr();
			let ith = ptr.add(index);
			let ith_item = ith.read();
			ptr::copy(ith.add(1), ith, new_len - index);
			ith_item
		}
	}

	#[inline]
	pub fn insert(&mut self, index: usize, value: T) {
		let len = self.len();
		assert!(
			index <= len,
			"insertion index (is {index}) should be <= len (is {len})"
		);
		self.reserve(1);
		let ptr = self.as_mut_ptr();
		unsafe {
			if index < len {
				ptr::copy(ptr.add(index), ptr.add(index + 1), len - index);
			}

			ptr.add(index).write(value);

			self.set_len(len + 1);
		}
	}

	#[inline]
	pub fn into_vec(self) -> Vec<T> {
		let len = self.len();
		if self.spilled() {
			let this = ManuallyDrop::new(self);

			unsafe {
				let (ptr, cap) = this.raw.heap;
				Vec::from_raw_parts(ptr.as_ptr(), len, cap)
			}
		} else {
			let mut vec = Vec::with_capacity(len);
			let this = ManuallyDrop::new(self);
			unsafe {
				ptr::copy_nonoverlapping(this.raw.as_inline_ptr(), vec.as_mut_ptr(), len);
				vec.set_len(len);
			}

			vec
		}
	}

	#[inline]
	pub fn into_boxed_slice(self) -> Box<[T]> {
		self.into_vec().into_boxed_slice()
	}

	#[inline]
	pub fn into_inner(self) -> Result<[T; N], Self> {
		if self.len() == N {
			let mut this = self;

			unsafe {
				this.set_len(0);
			}

			let ptr = this.as_ptr().cast::<[T; N]>();

			unsafe { Ok(ptr.read()) }
		} else {
			Err(self)
		}
	}

	#[inline]
	pub fn retain(&mut self, mut f: impl FnMut(&T) -> bool) {
		self.retain_mut(|elem| f(elem));
	}

	#[inline]
	pub fn retain_mut(&mut self, mut f: impl FnMut(&mut T) -> bool) {
		let mut del = 0;
		let len = self.len();
		let ptr = self.as_mut_ptr();
		(0..len).for_each(|i| unsafe {
			if !f(&mut *ptr.add(i)) {
				del += 1;
			} else if del > 0 {
				ptr::swap(ptr.add(i), ptr.add(i - del));
			}
		});

		self.truncate(len - del);
	}

	#[inline]
	pub fn dedup(&mut self)
	where
		T: PartialEq,
	{
		self.dedup_by(|a, b| PartialEq::eq(a, b));
	}

	#[inline]
	pub fn dedup_by_key<K: PartialEq>(&mut self, mut key: impl FnMut(&mut T) -> K) {
		self.dedup_by(|a, b| PartialEq::eq(&key(a), &key(b)));
	}

	#[inline]
	pub fn dedup_by(&mut self, mut same_bucket: impl FnMut(&mut T, &mut T) -> bool) {
		let len = self.len();
		if len <= 1 {
			return;
		}

		let ptr = self.as_mut_ptr();
		let mut w = 1usize;

		unsafe {
			(1..len).for_each(|r| {
				let p_r = ptr.add(r);
				let p_wm1 = ptr.add(w - 1);
				if !same_bucket(&mut *p_r, &mut *p_wm1) {
					if r != w {
						let p_w = p_wm1.add(1);
						ptr::swap(p_r, p_w);
					}

					w += 1;
				}
			});
		}

		self.truncate(w);
	}

	#[allow(clippy::comparison_chain)]
	pub fn resize_with(&mut self, new_len: usize, f: impl FnMut() -> T) {
		let old_len = self.len();
		if old_len < new_len {
			let mut f = f;
			let additional = new_len - old_len;
			self.reserve(additional);
			(0..additional).for_each(|_| self.push(f()));
		} else if old_len > new_len {
			self.truncate(new_len);
		}
	}

	#[allow(clippy::missing_const_for_fn)]
	pub fn leak<'a>(self) -> &'a mut [T] {
		let mut me = ManuallyDrop::new(self);
		unsafe { core::slice::from_raw_parts_mut(me.as_mut_ptr(), me.len()) }
	}

	#[inline]
	pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
		unsafe {
			core::slice::from_raw_parts_mut(
				self.as_mut_ptr().add(self.len()).cast::<MaybeUninit<T>>(),
				self.capacity() - self.len(),
			)
		}
	}

	#[inline]
	pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Self {
		assert!(!Self::is_zst());

		let ptr = unsafe {
			debug_assert!(!ptr.is_null(), "called `from_raw_parts` with null pointer");
			NonNull::new_unchecked(ptr)
		};

		Self {
			len: TaggedLen::new(length, true, Self::is_zst()),
			raw: RawSmallVec::new_heap(ptr, capacity),
			marker: PhantomData,
		}
	}

	fn extend_impl(&mut self, iter: impl Iterator<Item = T>) {
		let mut iter = iter.fuse();
		let (lower_bound, _) = iter.size_hint();
		self.reserve(lower_bound);
		let mut capacity = self.capacity();
		let mut ptr = self.as_mut_ptr();
		unsafe {
			loop {
				let mut len = self.len();
				ptr = ptr.add(len);
				let mut guard = DropGuard { ptr, len: 0 };
				iter.by_ref().take(capacity - len).for_each(|item| {
					ptr.add(guard.len).write(item);
					guard.len += 1;
				});

				len += guard.len;
				mem::forget(guard);
				self.set_len(len);
				if let Some(item) = iter.next() {
					self.push(item);
				} else {
					return;
				}

				let (heap_ptr, heap_capacity) = self.raw.heap;
				ptr = heap_ptr.as_ptr();
				capacity = heap_capacity;
			}
		}
	}
}

impl<T: Clone, const N: usize> SmallVec<T, N> {
	pub fn resize(&mut self, len: usize, value: T) {
		let old_len = self.len();
		if len > old_len {
			self.extend(core::iter::repeat_n(value, len - old_len));
		} else {
			self.truncate(len);
		}
	}

	pub fn from_elem(elem: T, n: usize) -> Self {
		if n > Self::inline_size() {
			Self::from_vec(vec![elem; n])
		} else {
			let mut v = Self::new();

			unsafe {
				let ptr = v.raw.as_mut_inline_ptr();
				let mut guard = DropGuard { ptr, len: 0 };

				(0..n).for_each(|i| {
					guard.len = i;
					ptr.add(i).write(elem.clone());
				});

				mem::forget(guard);
				v.set_len(n);
			}

			v
		}
	}
}

impl<T: Copy, const N: usize> SmallVec<T, N> {
	pub fn from_slice(slice: &[T]) -> Self {
		let len = slice.len();
		if len <= Self::inline_size() {
			let mut this = Self::new();
			unsafe {
				let ptr = this.raw.as_mut_inline_ptr();
				ptr::copy_nonoverlapping(slice.as_ptr(), ptr, len);
				this.set_len(len);
			}
			this
		} else {
			let mut this = Vec::with_capacity(len);
			unsafe {
				let ptr = this.as_mut_ptr();
				ptr::copy_nonoverlapping(slice.as_ptr(), ptr, len);
				this.set_len(len);
			}

			Self::from_vec(this)
		}
	}

	pub fn insert_from_slice(&mut self, index: usize, slice: &[T]) {
		let len = self.len();
		let other_len = slice.len();
		assert!(index <= len);
		self.reserve(other_len);
		unsafe {
			let base_ptr = self.as_mut_ptr();
			let ith_ptr = base_ptr.add(index);
			let shifted_ptr = base_ptr.add(index + other_len);
			ptr::copy(ith_ptr, shifted_ptr, len - index);
			ptr::copy_nonoverlapping(slice.as_ptr(), ith_ptr, other_len);

			self.set_len(len + other_len);
		}
	}

	pub fn extend_from_slice(&mut self, slice: &[T]) {
		let len = self.len();
		let other_len = slice.len();
		self.reserve(other_len);

		unsafe {
			let base_ptr = self.as_mut_ptr();
			let end_ptr = base_ptr.add(len);
			ptr::copy_nonoverlapping(slice.as_ptr(), end_ptr, other_len);
			self.set_len(len + other_len);
		}
	}
}

impl<T, const N: usize> AsRef<[T]> for SmallVec<T, N> {
	fn as_ref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T, const N: usize> AsMut<[T]> for SmallVec<T, N> {
	fn as_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<T, const N: usize> Borrow<[T]> for SmallVec<T, N> {
	fn borrow(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T, const N: usize> BorrowMut<[T]> for SmallVec<T, N> {
	fn borrow_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

#[cfg(feature = "bytes")]
unsafe impl<const N: usize> BufMut for SmallVec<u8, N> {
	#[inline]
	fn remaining_mut(&self) -> usize {
		isize::MAX as usize - self.len()
	}

	#[inline]
	unsafe fn advance_mut(&mut self, cnt: usize) {
		let len = self.len();
		let remaining = self.capacity() - len;

		assert!(
			(remaining >= cnt),
			"advance out of bounds: the len is {remaining} but advancing by {cnt}"
		);

		unsafe { self.set_len(len + cnt) };
	}

	#[inline]
	fn chunk_mut(&mut self) -> &mut UninitSlice {
		if self.capacity() == self.len() {
			self.reserve(64);
		}

		let cap = self.capacity();
		let len = self.len();

		let ptr = self.as_mut_ptr();

		unsafe { UninitSlice::from_raw_parts_mut(ptr.add(len), cap - len) }
	}

	#[inline]
	fn put<T: bytes::buf::Buf>(&mut self, mut src: T)
	where
		Self: Sized,
	{
		self.reserve(src.remaining());

		while src.has_remaining() {
			let s = src.chunk();
			let l = s.len();
			self.extend_from_slice(s);
			src.advance(l);
		}
	}

	#[inline]
	fn put_slice(&mut self, src: &[u8]) {
		self.extend_from_slice(src);
	}

	#[inline]
	fn put_bytes(&mut self, val: u8, cnt: usize) {
		let new_len = self.len().saturating_add(cnt);
		self.resize(new_len, val);
	}
}

impl<T: Clone, const N: usize> Clone for SmallVec<T, N> {
	fn clone(&self) -> Self {
		Self::from(self.as_slice())
	}

	fn clone_from(&mut self, source: &Self) {
		self.truncate(source.len());

		let init = unsafe { source.get_unchecked(..self.len()) };
		let tail = unsafe { source.get_unchecked(self.len()..) };

		self.clone_from_slice(init);
		self.extend(tail.iter().cloned());
	}
}

impl<T: Debug, const N: usize> Debug for SmallVec<T, N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_list().entries(self.iter()).finish()
	}
}

impl<T, const N: usize> Default for SmallVec<T, N> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, const N: usize> Deref for SmallVec<T, N> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<T, const N: usize> DerefMut for SmallVec<T, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

#[cfg(feature = "nightly")]
unsafe impl<#[may_dangle] T, const N: usize> Drop for SmallVec<T, N> {
	fn drop(&mut self) {
		let on_heap = self.spilled();
		let len = self.len();
		let ptr = self.as_mut_ptr();

		unsafe {
			let _drop_dealloc = if on_heap {
				let capacity = self.capacity();
				Some(DropDealloc {
					ptr: NonNull::new_unchecked(ptr),
					capacity,
				})
			} else {
				None
			};

			ptr::slice_from_raw_parts_mut(ptr, len).drop_in_place();
		}
	}
}

#[cfg(not(feature = "nightly"))]
impl<T, const N: usize> Drop for SmallVec<T, N> {
	fn drop(&mut self) {
		let on_heap = self.spilled();
		let len = self.len();
		let ptr = self.as_mut_ptr();
		unsafe {
			let _drop_dealloc = if on_heap {
				let capacity = self.capacity();
				Some(DropDealloc {
					ptr: NonNull::new_unchecked(ptr),
					capacity,
				})
			} else {
				None
			};

			ptr::slice_from_raw_parts_mut(ptr, len).drop_in_place();
		}
	}
}

impl<T: Eq, const N: usize> Eq for SmallVec<T, N> {}

impl<T, const N: usize> Extend<T> for SmallVec<T, N> {
	fn extend<I>(&mut self, iter: I)
	where
		I: IntoIterator<Item = T>,
	{
		self.extend_impl(iter.into_iter());
	}
}

impl<T, const N: usize, const M: usize> From<[T; M]> for SmallVec<T, N> {
	fn from(value: [T; M]) -> Self {
		if M > N {
			Self::from(Vec::from(value))
		} else {
			let mut this = Self::new();
			debug_assert!(M <= this.capacity());
			let array = ManuallyDrop::new(value);
			unsafe {
				ptr::copy_nonoverlapping(array.as_ptr(), this.as_mut_ptr(), M);
				this.set_len(M);
			}

			this
		}
	}
}

impl<T, const N: usize> From<Vec<T>> for SmallVec<T, N> {
	fn from(value: Vec<T>) -> Self {
		Self::from_vec(value)
	}
}

#[cfg(feature = "nightly")]
impl<'a, T: Clone, const N: usize> From<&'a [T]> for SmallVec<T, N> {
	fn from(value: &'a [T]) -> Self {
		SpecFrom::spec_from(value)
	}
}

#[cfg(not(feature = "nightly"))]
impl<'a, T: Clone, const N: usize> From<&'a [T]> for SmallVec<T, N> {
	fn from(value: &'a [T]) -> Self {
		value.iter().cloned().collect()
	}
}

impl<T, const N: usize> FromIterator<T> for SmallVec<T, N> {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = T>,
	{
		let mut this = Self::new();
		this.extend_impl(iter.into_iter());
		this
	}
}

impl<T: Hash, const N: usize> Hash for SmallVec<T, N> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_slice().hash(state);
	}
}

impl<T, const N: usize> InsertOrPush<T> for SmallVec<T, N> {
	fn insert_or_push(&mut self, index: usize, value: T) {
		if index >= self.len() {
			self.push(value);
		} else {
			self.insert(index, value);
		}
	}
}

impl<T, const N: usize> IntoIterator for SmallVec<T, N> {
	type IntoIter = IntoIter<T, N>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		unsafe {
			let this = ManuallyDrop::new(self);
			IntoIter {
				raw: (&raw const this.raw).read(),
				begin: 0,
				end: this.len,
				marker: PhantomData,
			}
		}
	}
}

impl<'a, T, const N: usize> IntoIterator for &'a SmallVec<T, N> {
	type IntoIter = core::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, const N: usize> IntoIterator for &'a mut SmallVec<T, N> {
	type IntoIter = core::slice::IterMut<'a, T>;
	type Item = &'a mut T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<T: Ord, const N: usize> Ord for SmallVec<T, N> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_slice().cmp(other.as_slice())
	}
}

impl<T, U, const N: usize, const M: usize> PartialEq<SmallVec<U, M>> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &SmallVec<U, M>) -> bool {
		self.as_slice().eq(other.as_slice())
	}
}

impl<T, U, const N: usize, const M: usize> PartialEq<[U; M]> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &[U; M]) -> bool {
		self[..] == other[..]
	}
}

impl<T, U, const N: usize, const M: usize> PartialEq<&[U; M]> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &&[U; M]) -> bool {
		self[..] == other[..]
	}
}

impl<T, U, const N: usize> PartialEq<[U]> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &[U]) -> bool {
		self[..] == other[..]
	}
}

impl<T, U, const N: usize> PartialEq<&[U]> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &&[U]) -> bool {
		self[..] == other[..]
	}
}

impl<T, U, const N: usize> PartialEq<&mut [U]> for SmallVec<T, N>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &&mut [U]) -> bool {
		self[..] == other[..]
	}
}

impl<T: PartialOrd, const N: usize> PartialOrd for SmallVec<T, N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.as_slice().partial_cmp(other.as_slice())
	}
}

#[expect(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: Send, const N: usize> Send for SmallVec<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for SmallVec<T, N> {}

#[cfg(feature = "std")]
impl<const N: usize> std::io::Write for SmallVec<u8, N> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
		self.extend_from_slice(buf);
		Ok(())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct TaggedLen(usize);

impl TaggedLen {
	#[inline]
	pub const fn new(len: usize, on_heap: bool, is_zst: bool) -> Self {
		if is_zst {
			debug_assert!(!on_heap);
			Self(len)
		} else {
			debug_assert!(len < isize::MAX as usize);
			Self((len << 1) | on_heap as usize)
		}
	}

	#[inline]
	pub const fn on_heap(self, is_zst: bool) -> bool {
		if is_zst {
			false
		} else {
			matches!(self.0 & 1, 1)
		}
	}

	#[inline]
	pub const fn value(self, is_zst: bool) -> usize {
		if is_zst { self.0 } else { self.0 >> 1 }
	}
}

struct DropGuard<T> {
	ptr: *mut T,
	len: usize,
}

impl<T> Drop for DropGuard<T> {
	fn drop(&mut self) {
		unsafe {
			ptr::slice_from_raw_parts_mut(self.ptr, self.len).drop_in_place();
		}
	}
}

struct DropDealloc<T> {
	ptr: NonNull<T>,
	capacity: usize,
}

impl<T> Drop for DropDealloc<T> {
	fn drop(&mut self) {
		unsafe {
			if mem::size_of::<T>() > 0 {
				let layout = Layout::from_size_align_unchecked(
					mem::size_of::<T>() * self.capacity,
					mem::align_of::<T>(),
				);
				alloc::alloc::dealloc(self.ptr.cast().as_ptr(), layout);
			}
		}
	}
}

#[derive(Debug)]
pub enum CollectionAllocError {
	CapacityOverflow,
	Alloc { layout: Layout },
}

impl Display for CollectionAllocError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::CapacityOverflow => f.write_str("capacity overflow"),
			Self::Alloc { layout } => {
				f.write_str("failed to alloc layout ")?;
				Debug::fmt(&layout, f)
			}
		}
	}
}

impl CoreError for CollectionAllocError {}

#[repr(C)]
pub union RawSmallVec<T, const N: usize> {
	inline: ManuallyDrop<MaybeUninit<[T; N]>>,
	heap: (NonNull<T>, usize),
}

impl<T, const N: usize> RawSmallVec<T, N> {
	#[inline]
	const fn is_zst() -> bool {
		matches!(mem::size_of::<T>(), 0)
	}

	#[inline]
	const fn new() -> Self {
		Self::new_inline(MaybeUninit::uninit())
	}

	#[inline]
	const fn new_inline(inline: MaybeUninit<[T; N]>) -> Self {
		Self {
			inline: ManuallyDrop::new(inline),
		}
	}

	#[inline]
	const fn new_heap(ptr: NonNull<T>, capacity: usize) -> Self {
		Self {
			heap: (ptr, capacity),
		}
	}

	#[inline]
	const fn as_inline_ptr(&self) -> *const T {
		(unsafe { ptr::addr_of!(self.inline) }).cast::<T>()
	}

	#[inline]
	const fn as_mut_inline_ptr(&mut self) -> *mut T {
		(unsafe { ptr::addr_of_mut!(self.inline) }).cast::<T>()
	}

	#[inline]
	const unsafe fn as_heap_ptr(&self) -> *const T {
		unsafe { self.heap.0.as_ptr() }
	}

	#[inline]
	const unsafe fn as_mut_heap_ptr(&mut self) -> *mut T {
		unsafe { self.heap.0.as_ptr() }
	}

	unsafe fn try_grow_raw(
		&mut self,
		len: TaggedLen,
		new_capacity: usize,
	) -> Result<(), CollectionAllocError> {
		use alloc::alloc::{alloc, realloc};

		debug_assert!(!Self::is_zst());
		debug_assert!(new_capacity > 0);
		debug_assert!(new_capacity >= len.value(Self::is_zst()));

		let was_on_heap = len.on_heap(Self::is_zst());
		let ptr = if was_on_heap {
			unsafe { self.as_mut_heap_ptr() }
		} else {
			self.as_mut_inline_ptr()
		};

		let len = len.value(Self::is_zst());

		let new_layout =
			Layout::array::<T>(new_capacity).map_err(|_| CollectionAllocError::CapacityOverflow)?;
		if new_layout.size() > isize::MAX as usize {
			return Err(CollectionAllocError::CapacityOverflow);
		}

		let new_ptr = if matches!(len, 0) || !was_on_heap {
			let new_ptr = unsafe { alloc(new_layout).cast::<T>() };
			let new_ptr =
				NonNull::new(new_ptr).ok_or(CollectionAllocError::Alloc { layout: new_layout })?;
			unsafe { ptr::copy_nonoverlapping(ptr, new_ptr.as_ptr(), len) };
			new_ptr
		} else {
			let old_layout = unsafe {
				Layout::from_size_align_unchecked(
					self.heap.1 * mem::size_of::<T>(),
					mem::align_of::<T>(),
				)
			};
			let new_ptr =
				unsafe { realloc(ptr.cast::<u8>(), old_layout, new_layout.size()).cast::<T>() };
			NonNull::new(new_ptr).ok_or(CollectionAllocError::Alloc { layout: new_layout })?
		};

		*self = Self::new_heap(new_ptr, new_capacity);

		Ok(())
	}
}

#[cfg(feature = "nightly")]
trait SpecFrom<T> {
	fn spec_from(slice: &[T]) -> Self;
}

#[cfg(feature = "nightly")]
impl<T: Clone, const N: usize> SpecFrom<T> for SmallVec<T, N> {
	default fn spec_from(slice: &[T]) -> Self {
		slice.iter().cloned().collect()
	}
}

#[cfg(feature = "nightly")]
impl<T: Copy, const N: usize> SpecFrom<T> for SmallVec<T, N> {
	fn spec_from(slice: &[T]) -> Self {
		Self::from_slice(slice)
	}
}

#[inline]
fn infallible<T>(res: Result<T, CollectionAllocError>) -> T {
	match res {
		Ok(x) => x,
		Err(e @ CollectionAllocError::CapacityOverflow) => panic!("{e}"),
		Err(CollectionAllocError::Alloc { layout }) => alloc::alloc::handle_alloc_error(layout),
	}
}
