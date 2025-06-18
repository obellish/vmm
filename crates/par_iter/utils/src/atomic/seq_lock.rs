use core::{
	mem,
	sync::atomic::{self, AtomicUsize, Ordering},
};

use crate::Backoff;

pub(crate) struct SeqLock {
	state: AtomicUsize,
}

impl SeqLock {
	pub const fn new() -> Self {
		Self {
			state: AtomicUsize::new(0),
		}
	}

	pub fn optimistic_read(&self) -> Option<usize> {
		let state = self.state.load(Ordering::SeqCst);
		if matches!(state, 1) {
			None
		} else {
			Some(state)
		}
	}

	pub fn validate_read(&self, stamp: usize) -> bool {
		atomic::fence(Ordering::Acquire);
		self.state.load(Ordering::Relaxed) == stamp
	}

	pub fn write(&'static self) -> SeqLockWriteGuard {
		let backoff = Backoff::new();
		loop {
			let previous = self.state.swap(1, Ordering::Acquire);

			if !matches!(previous, 1) {
				atomic::fence(Ordering::Release);

				return SeqLockWriteGuard {
					lock: self,
					state: previous,
				};
			}

			backoff.snooze();
		}
	}
}

pub(crate) struct SeqLockWriteGuard {
	lock: &'static SeqLock,
	state: usize,
}

impl SeqLockWriteGuard {
	pub fn abort(self) {
		self.lock.state.store(self.state, Ordering::Release);

		mem::forget(self);
	}
}

impl Drop for SeqLockWriteGuard {
	fn drop(&mut self) {
		self.lock
			.state
			.store(self.state.wrapping_add(2), Ordering::Release);
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
