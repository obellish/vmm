use alloc::boxed::Box;
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	ops::Deref,
	ptr,
	sync::atomic::{AtomicPtr, Ordering},
};

pub struct RacyLock<T, F = fn() -> T>
where
	F: Fn() -> T,
{
	inner: AtomicPtr<T>,
	f: F,
}

impl<T, F> RacyLock<T, F>
where
	F: Fn() -> T,
{
	pub const fn new(f: F) -> Self {
		Self {
			inner: AtomicPtr::new(ptr::null_mut()),
			f,
		}
	}

	pub fn force(this: &Self) -> &T {
		let mut ptr = this.inner.load(Ordering::Acquire);

		if ptr.is_null() {
			let val = (this.f)();
			ptr = Box::into_raw(Box::new(val));

			let exchange = this.inner.compare_exchange(
				ptr::null_mut(),
				ptr,
				Ordering::AcqRel,
				Ordering::Acquire,
			);

			if let Err(old) = exchange {
				drop(unsafe { Box::from_raw(ptr) });
				ptr = old;
			}
		}

		unsafe { &*ptr }
	}
}

impl<T: Debug, F> Debug for RacyLock<T, F>
where
	F: Fn() -> T,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "RacyLock({:?})", self.inner.load(Ordering::Relaxed))
	}
}

impl<T: Default> Default for RacyLock<T> {
	fn default() -> Self {
		Self::new(T::default)
	}
}

impl<T, F> Deref for RacyLock<T, F>
where
	F: Fn() -> T,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		Self::force(self)
	}
}

impl<T, F> Drop for RacyLock<T, F>
where
	F: Fn() -> T,
{
	fn drop(&mut self) {
		let ptr = *self.inner.get_mut();
		if !ptr.is_null() {
			drop(unsafe { Box::from_raw(ptr) });
		}
	}
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use static_assertions::assert_impl_all;

	use super::RacyLock;

	assert_impl_all!(RacyLock<Vec<i32>>: Send, Sync);

	#[test]
	fn deref_default() {
		let lock: RacyLock<i32> = RacyLock::default();
		assert_eq!(*lock, 0);
	}

	#[test]
	fn deref_copy() {
		let lock = RacyLock::new(|| 42);
		assert_eq!(*lock, 42);
	}

	#[test]
	fn deref_clone() {
		let lock = RacyLock::new(|| Vec::from([1, 2, 3]));

		let mut v = lock.clone();
		v.push(4);

		assert_eq!(v, Vec::from([1, 2, 3, 4]));
	}

	#[test]
	fn deref_static() {
		static VEC: RacyLock<Vec<i32>> = RacyLock::new(|| Vec::from([1, 2, 3]));

		let addr = &raw const *VEC;
		for _ in 0..5 {
			assert_eq!(*VEC, [1, 2, 3]);
			assert_eq!(addr, &raw const (*VEC));
		}
	}

	#[test]
	fn type_inference() {
		_ = RacyLock::new(|| ());
	}
}
