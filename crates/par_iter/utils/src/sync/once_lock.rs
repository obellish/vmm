use core::{cell::UnsafeCell, mem::MaybeUninit};
use std::sync::Once;

pub(crate) struct OnceLock<T> {
	once: Once,
	value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceLock<T> {
	pub const fn new() -> Self {
		Self {
			once: Once::new(),
			value: UnsafeCell::new(MaybeUninit::uninit()),
		}
	}

	pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
		if self.once.is_completed() {
			return unsafe { self.get_unchecked() };
		}

		self.initialize(f);

		unsafe { self.get_unchecked() }
	}

	#[cold]
	fn initialize(&self, f: impl FnOnce() -> T) {
		let slot = self.value.get();

		self.once.call_once(|| {
			let value = f();
			unsafe { slot.write(MaybeUninit::new(value)) }
		});
	}

	unsafe fn get_unchecked(&self) -> &T {
		debug_assert!(self.once.is_completed());
		unsafe { (*self.value.get()).assume_init_ref() }
	}
}

impl<T> Drop for OnceLock<T> {
	fn drop(&mut self) {
		if self.once.is_completed() {
			unsafe {
				self.value.get().cast::<T>().drop_in_place();
			}
		}
	}
}

unsafe impl<T: Send> Send for OnceLock<T> {}
unsafe impl<T> Sync for OnceLock<T> where T: Send + Sync {}
