use std::sync::{Arc, Mutex};

#[must_use]
#[repr(transparent)]
pub struct Promise<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Promise<T> {
	pub fn new() -> Self {
		Self(Arc::new(Mutex::new(FutureValue::Wait)))
	}

	pub fn start() -> (Self, Future<T>) {
		let promise = Self::new();
		let future = Future::new(&promise);
		(promise, future)
	}

	pub fn set(&self, value: T) {
		let mut future = self.0.lock().unwrap();
		*future = FutureValue::Ready(value);
	}
}

impl<T> Default for Promise<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[must_use]
#[repr(transparent)]
pub struct Future<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Future<T> {
	pub fn expired() -> Self {
		Self(Arc::new(Mutex::new(FutureValue::Expired)))
	}

	pub fn new(promise: &Promise<T>) -> Self {
		Self(promise.0.clone())
	}

	pub fn poll(&self) -> FutureValue<T> {
		let Ok(mut value) = self.0.try_lock() else {
			return FutureValue::Wait;
		};

		if matches!(*value, FutureValue::Wait) {
			return FutureValue::Wait;
		}

		if value.is_expired() {
			return FutureValue::Expired;
		}

		std::mem::replace(&mut *value, FutureValue::Expired)
	}

	pub fn forget(self) {
		drop(self);
	}
}

impl<T> Default for Future<T> {
	fn default() -> Self {
		Self::expired()
	}
}

#[must_use]
#[expect(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, PartialEq)]
pub enum FutureValue<T> {
	Wait,
	Ready(T),
	Expired,
}

impl<T> FutureValue<T> {
	pub const fn is_ready(&self) -> bool {
		matches!(self, Self::Ready(_))
	}

	#[must_use]
	pub const fn is_expired(&self) -> bool {
		matches!(self, Self::Expired)
	}

	pub fn unwrap(self) -> T {
		if let Self::Ready(value) = self {
			value
		} else {
			panic!("future is not ready")
		}
	}
}
