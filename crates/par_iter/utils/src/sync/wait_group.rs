use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	sync::{Arc, Condvar, Mutex},
};

#[repr(transparent)]
pub struct WaitGroup {
	inner: Arc<Inner>,
}

impl WaitGroup {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	pub fn wait(self) {
		if matches!(*self.inner.count.lock().unwrap(), 1) {
			return;
		}

		let inner = self.inner.clone();
		drop(self);

		let mut count = inner.count.lock().unwrap();
		while *count > 0 {
			count = inner.cvar.wait(count).unwrap();
		}
	}
}

impl Clone for WaitGroup {
	fn clone(&self) -> Self {
		let mut count = self.inner.count.lock().unwrap();
		*count += 1;

		Self {
			inner: self.inner.clone(),
		}
	}
}

impl Debug for WaitGroup {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let count: &usize = &self.inner.count.lock().unwrap();
		f.debug_struct("WaitGroup").field("count", count).finish()
	}
}

impl Default for WaitGroup {
	fn default() -> Self {
		Self {
			inner: Arc::new(Inner {
				cvar: Condvar::new(),
				count: Mutex::new(1),
			}),
		}
	}
}

impl Drop for WaitGroup {
	fn drop(&mut self) {
		let mut count = self.inner.count.lock().unwrap();
		*count -= 1;

		if matches!(*count, 0) {
			self.inner.cvar.notify_all();
		}
	}
}

struct Inner {
	cvar: Condvar,
	count: Mutex<usize>,
}
