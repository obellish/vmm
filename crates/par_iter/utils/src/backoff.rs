use core::{
	cell::Cell,
	fmt::{Debug, Formatter, Result as FmtResult},
	hint,
};

const SPIN_LIMIT: u32 = 6;
const YIELD_LIMIT: u32 = 10;

#[repr(transparent)]
pub struct Backoff {
	step: Cell<u32>,
}

impl Backoff {
	#[must_use]
	pub const fn new() -> Self {
		Self { step: Cell::new(0) }
	}

	pub fn reset(&self) {
		self.step.set(0);
	}

	pub fn spin(&self) {
		for _ in 0..1 << self.step.get().min(SPIN_LIMIT) {
			hint::spin_loop();
		}

		if self.step.get() <= SPIN_LIMIT {
			self.step.set(self.step.get() + 1);
		}
	}

	pub fn snooze(&self) {
		if self.step.get() <= SPIN_LIMIT {
			for _ in 0..1 << self.step.get() {
				hint::spin_loop();
			}
		} else {
			#[cfg(not(feature = "std"))]
			for _ in 0..1 << self.step.get() {
				hint::spin_loop();
			}

			#[cfg(feature = "std")]
			::std::thread::yield_now();
		}

		if self.step.get() <= YIELD_LIMIT {
			self.step.set(self.step.get() + 1);
		}
	}

	pub fn is_completed(&self) -> bool {
		self.step.get() > YIELD_LIMIT
	}
}

impl Debug for Backoff {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("Backoff")
			.field("step", &self.step)
			.field("is_completed", &self.is_completed())
			.finish()
	}
}

impl Default for Backoff {
	fn default() -> Self {
		Self::new()
	}
}
