use core::{
	mem,
	sync::atomic::{self, AtomicUsize, Ordering},
};

use crate::Backoff;

pub(crate) struct SeqLock {
	state_hi: AtomicUsize,
	state_lo: AtomicUsize,
}

impl SeqLock {
	pub const fn new() -> Self {
		Self {
			state_hi: AtomicUsize::new(0),
			state_lo: AtomicUsize::new(0),
		}
	}

	pub fn optimistic_read(&self) -> Option<(usize, usize)> {
		let state_hi = self.state_hi.load(Ordering::Acquire);
		let state_lo = self.state_lo.load(Ordering::Acquire);

		if matches!(state_lo, 1) {
			None
		} else {
			Some((state_hi, state_lo))
		}
	}

	pub fn validate_read(&self, stamp: (usize, usize)) -> bool {
		atomic::fence(Ordering::Acquire);

		let state_lo = self.state_lo.load(Ordering::Acquire);
		let state_hi = self.state_hi.load(Ordering::Relaxed);

		(state_hi, state_lo) == stamp
	}

	pub fn write(&'static self) -> SeqLockWriteGuard {
		let backoff = Backoff::new();
		loop {
			let previous = self.state_lo.swap(1, Ordering::Acquire);

			if !matches!(previous, 1) {
				atomic::fence(Ordering::Release);

				return SeqLockWriteGuard {
					lock: self,
					state_lo: previous,
				};
			}

			backoff.snooze();
		}
	}
}

pub(crate) struct SeqLockWriteGuard {
	lock: &'static SeqLock,
	state_lo: usize,
}

impl SeqLockWriteGuard {
	pub fn abort(self) {
		self.lock.state_lo.store(self.state_lo, Ordering::Release);
		mem::forget(self);
	}
}

impl Drop for SeqLockWriteGuard {
	fn drop(&mut self) {
		let state_lo = self.state_lo.wrapping_add(2);

		if matches!(state_lo, 0) {
			let state_hi = self.lock.state_hi.load(Ordering::Relaxed);
			self.lock
				.state_hi
				.store(state_hi.wrapping_add(1), Ordering::Release);
		}

		self.lock.state_lo.store(state_lo, Ordering::Release);
	}
}

#[cfg(test)]
mod tests {
	use super::SeqLock;

	#[test]
	fn abort() {
		static LK: SeqLock = SeqLock::new();
		let before = LK.optimistic_read().unwrap();
		{
			let guard = LK.write();
			guard.abort();
		}
		let after = LK.optimistic_read().unwrap();
		assert_eq!(before, after);
	}
}
