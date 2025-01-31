#[cfg(not(loom))]
use core::{
	hint,
	sync::atomic::{AtomicUsize, Ordering},
};

use lock_api::RawRwLock;
#[cfg(loom)]
use loom::{
	hint,
	sync::atomic::{AtomicUsize, Ordering},
};

pub struct SpinLock {
	state: AtomicUsize,
	writer_wake_counter: AtomicUsize,
}

impl SpinLock {
	#[cfg(not(loom))]
	#[must_use]
	pub const fn new() -> Self {
		Self {
			state: AtomicUsize::new(0),
			writer_wake_counter: AtomicUsize::new(0),
		}
	}

	#[cfg(loom)]
	#[must_use]
	pub fn new() -> Self {
		Self {
			state: AtomicUsize::new(0),
			writer_wake_counter: AtomicUsize::new(0),
		}
	}
}

impl Default for SpinLock {
	fn default() -> Self {
		Self::new()
	}
}

unsafe impl RawRwLock for SpinLock {
	type GuardMarker = lock_api::GuardSend;

	#[cfg(loom)]
	const INIT: Self = unimplemented!();
	#[cfg(not(loom))]
	#[allow(clippy::declare_interior_mutable_const)]
	const INIT: Self = Self::new();

	fn lock_shared(&self) {
		let mut s = self.state.load(Ordering::Relaxed);
		loop {
			if matches!(s & 1, 0) {
				match self.state.compare_exchange_weak(
					s,
					s + 2,
					Ordering::Acquire,
					Ordering::Relaxed,
				) {
					Ok(_) => return,
					Err(e) => s = e,
				}
			}

			if matches!(s & 1, 1) {
				loop {
					let next = self.state.load(Ordering::Relaxed);
					if s == next {
						hint::spin_loop();
						continue;
					}
					s = next;
					break;
				}
			}
		}
	}

	fn try_lock_shared(&self) -> bool {
		let s = self.state.load(Ordering::Relaxed);
		if matches!(s & 1, 0) {
			self.state
				.compare_exchange_weak(s, s + 2, Ordering::Acquire, Ordering::Relaxed)
				.is_ok()
		} else {
			false
		}
	}

	unsafe fn unlock_shared(&self) {
		if matches!(self.state.fetch_sub(2, Ordering::Release), 3) {
			self.writer_wake_counter.fetch_add(1, Ordering::Release);
		}
	}

	fn lock_exclusive(&self) {
		let mut s = self.state.load(Ordering::Relaxed);
		loop {
			if s <= 1 {
				match self.state.compare_exchange(
					s,
					usize::MAX,
					Ordering::Acquire,
					Ordering::Relaxed,
				) {
					Ok(_) => return,
					Err(e) => {
						s = e;
						hint::spin_loop();
						continue;
					}
				}
			}

			if matches!(s & 1, 0) {
				if let Err(e) =
					self.state
						.compare_exchange(s, s + 1, Ordering::Relaxed, Ordering::Relaxed)
				{
					s = e;
					continue;
				}
			}

			let w = self.writer_wake_counter.load(Ordering::Acquire);
			s = self.state.load(Ordering::Relaxed);

			if s >= 2 {
				while self.writer_wake_counter.load(Ordering::Acquire) == w {
					hint::spin_loop();
				}
				s = self.state.load(Ordering::Relaxed);
			}
		}
	}

	fn try_lock_exclusive(&self) -> bool {
		let s = self.state.load(Ordering::Relaxed);
		if s <= 1 {
			self.state
				.compare_exchange(s, usize::MAX, Ordering::Acquire, Ordering::Relaxed)
				.is_ok()
		} else {
			false
		}
	}

	unsafe fn unlock_exclusive(&self) {
		self.state.store(0, Ordering::Release);

		self.writer_wake_counter.fetch_add(1, Ordering::Release);
	}
}

pub type RwLock<T> = lock_api::RwLock<SpinLock, T>;

pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, SpinLock, T>;

pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, SpinLock, T>;

#[cfg(all(test, loom))]
mod tests {
	use alloc::vec::Vec;

	use loom::{model::Builder, sync::Arc};

	use super::{RwLock, SpinLock};

	#[test]
	fn rwlock_loom() {
		let mut builder = Builder::default();
		builder.max_duration = Some(core::time::Duration::from_secs(60));
		builder.log = true;
		builder.check(|| {
			let raw_rwlock = SpinLock::new();
			let n = Arc::new(RwLock::from_raw(raw_rwlock, 0usize));
			let mut readers = Vec::new();
			let mut writers = Vec::new();

			let num_readers = 2;
			let num_writers = 2;
			let num_iterations = 2;

			for _ in 0..num_readers {
				let n0 = n.clone();
				let t = loom::thread::spawn(move || {
					for _ in 0..num_iterations {
						let guard = n0.read();
						assert_eq!(*guard, 0);
					}
				});

				readers.push(t);
			}

			for _ in 0..num_writers {
				let n0 = n.clone();
				let t = loom::thread::spawn(move || {
					for _ in 0..num_iterations {
						let mut guard = n0.write();
						assert_eq!(*guard, 0);
						*guard += 1;
						assert_eq!(*guard, 1);
						*guard -= 1;
						assert_eq!(*guard, 0);
					}
				});

				writers.push(t);
			}

			for t in readers {
				t.join().unwrap();
			}

			for t in writers {
				t.join().unwrap();
			}
		});
	}
}
