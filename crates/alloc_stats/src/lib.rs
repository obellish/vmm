#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::alloc::{GlobalAlloc, Layout};
use core::{
	ops::{Sub, SubAssign},
	sync::atomic::{AtomicIsize, AtomicUsize, Ordering::SeqCst},
};
#[cfg(feature = "std")]
use std::alloc::System;

#[derive(Debug, Default)]
pub struct StatsAlloc<T> {
	allocations: AtomicUsize,
	deallocations: AtomicUsize,
	reallocations: AtomicUsize,
	bytes_allocated: AtomicUsize,
	bytes_deallocated: AtomicUsize,
	bytes_reallocated: AtomicIsize,
	inner: T,
}

impl<T> StatsAlloc<T> {
	pub const fn new(inner: T) -> Self {
		Self {
			allocations: AtomicUsize::new(0),
			deallocations: AtomicUsize::new(0),
			reallocations: AtomicUsize::new(0),
			bytes_allocated: AtomicUsize::new(0),
			bytes_deallocated: AtomicUsize::new(0),
			bytes_reallocated: AtomicIsize::new(0),
			inner,
		}
	}

	pub fn stats(&self) -> Stats {
		Stats {
			allocations: self.allocations.load(SeqCst),
			deallocations: self.deallocations.load(SeqCst),
			reallocations: self.reallocations.load(SeqCst),
			bytes_allocated: self.bytes_allocated.load(SeqCst),
			bytes_deallocated: self.bytes_deallocated.load(SeqCst),
			bytes_reallocated: self.bytes_reallocated.load(SeqCst),
		}
	}
}

#[cfg(feature = "std")]
impl StatsAlloc<System> {
	#[must_use]
	pub const fn system() -> Self {
		Self::new(System)
	}
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for StatsAlloc<T> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.allocations.fetch_add(1, SeqCst);
		self.bytes_allocated.fetch_add(layout.size(), SeqCst);
		unsafe { self.inner.alloc(layout) }
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.deallocations.fetch_add(1, SeqCst);
		self.bytes_deallocated.fetch_add(layout.size(), SeqCst);
		unsafe { self.inner.dealloc(ptr, layout) };
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		self.reallocations.fetch_add(1, SeqCst);
		if new_size > layout.size() {
			let difference = new_size - layout.size();
			self.bytes_allocated.fetch_add(difference, SeqCst);
		} else if new_size < layout.size() {
			let difference = layout.size() - new_size;
			self.bytes_deallocated.fetch_add(difference, SeqCst);
		}

		self.bytes_reallocated
			.fetch_add(new_size.wrapping_sub(layout.size()) as isize, SeqCst);
		unsafe { self.inner.realloc(ptr, layout, new_size) }
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stats {
	pub allocations: usize,
	pub deallocations: usize,
	pub reallocations: usize,
	pub bytes_allocated: usize,
	pub bytes_deallocated: usize,
	pub bytes_reallocated: isize,
}

impl Sub for Stats {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			allocations: Sub::sub(self.allocations, rhs.allocations),
			deallocations: Sub::sub(self.deallocations, rhs.deallocations),
			reallocations: Sub::sub(self.reallocations, rhs.reallocations),
			bytes_allocated: Sub::sub(self.bytes_allocated, rhs.bytes_allocated),
			bytes_deallocated: Sub::sub(self.bytes_deallocated, rhs.bytes_deallocated),
			bytes_reallocated: Sub::sub(self.bytes_reallocated, rhs.bytes_reallocated),
		}
	}
}

impl SubAssign for Stats {
	fn sub_assign(&mut self, rhs: Self) {
		*self = Sub::sub(*self, rhs);
	}
}

pub struct Region<'a, T> {
	alloc: &'a StatsAlloc<T>,
	initial_stats: Stats,
}

impl<'a, T> Region<'a, T> {
	pub fn new(alloc: &'a StatsAlloc<T>) -> Self {
		Self {
			alloc,
			initial_stats: alloc.stats(),
		}
	}

	#[must_use]
	pub const fn initial_stats(&self) -> Stats {
		self.initial_stats
	}

	#[must_use]
	pub fn current_stats(&self) -> Stats {
		self.alloc.stats()
	}

	#[must_use]
	pub fn stat_diff(&self) -> Stats {
		self.current_stats() - self.initial_stats()
	}

	pub fn reset(&mut self) {
		self.initial_stats = self.current_stats();
	}
}
