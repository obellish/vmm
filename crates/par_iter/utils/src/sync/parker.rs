use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
	sync::atomic::{AtomicUsize, Ordering::SeqCst},
	time::Duration,
};
use std::{
	sync::{Arc, Condvar, Mutex},
	time::Instant,
};

const EMPTY: usize = 0;
const PARKED: usize = 1;
const NOTIFIED: usize = 2;

#[repr(transparent)]
pub struct Parker {
	unparker: Unparker,
	marker: PhantomData<*const ()>,
}

impl Parker {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	pub fn park(&self) {
		self.unparker.inner.park(None);
	}

	#[must_use]
	pub fn park_timeout(&self, timeout: Duration) -> UnparkReason {
		if let Some(deadline) = Instant::now().checked_add(timeout) {
			self.park_deadline(deadline)
		} else {
			self.park();
			UnparkReason::Unparked
		}
	}

	#[must_use]
	pub fn park_deadline(&self, deadline: Instant) -> UnparkReason {
		self.unparker.inner.park(Some(deadline))
	}

	#[must_use]
	pub const fn unparker(&self) -> &Unparker {
		&self.unparker
	}

	#[must_use]
	pub fn into_raw(this: Self) -> *const () {
		Unparker::into_raw(this.unparker)
	}

	#[must_use]
	pub unsafe fn from_raw(ptr: *const ()) -> Self {
		Self {
			unparker: unsafe { Unparker::from_raw(ptr) },
			marker: PhantomData,
		}
	}
}

impl Debug for Parker {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.pad("Parker { .. }")
	}
}

impl Default for Parker {
	fn default() -> Self {
		Self {
			unparker: Unparker {
				inner: Arc::new(Inner {
					state: AtomicUsize::new(EMPTY),
					lock: Mutex::new(()),
					cvar: Condvar::new(),
				}),
			},
			marker: PhantomData,
		}
	}
}

unsafe impl Send for Parker {}

#[repr(transparent)]
pub struct Unparker {
	inner: Arc<Inner>,
}

impl Unparker {
	pub fn unpark(&self) {
		self.inner.unpark();
	}

	#[must_use]
	pub fn into_raw(this: Self) -> *const () {
		Arc::into_raw(this.inner).cast::<()>()
	}

	#[must_use]
	pub unsafe fn from_raw(ptr: *const ()) -> Self {
		Self {
			inner: unsafe { Arc::from_raw(ptr.cast()) },
		}
	}
}

impl Clone for Unparker {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl Debug for Unparker {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.pad("Unparker { .. }")
	}
}

unsafe impl Send for Unparker {}
unsafe impl Sync for Unparker {}

struct Inner {
	state: AtomicUsize,
	lock: Mutex<()>,
	cvar: Condvar,
}

impl Inner {
	fn park(&self, deadline: Option<Instant>) -> UnparkReason {
		if self
			.state
			.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
			.is_ok()
		{
			return UnparkReason::Unparked;
		}

		if let Some(deadline) = deadline {
			if deadline <= Instant::now() {
				return UnparkReason::Timeout;
			}
		}

		let mut m = self.lock.lock().unwrap();

		match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
			Ok(_) => {}
			Err(NOTIFIED) => {
				let old = self.state.swap(EMPTY, SeqCst);
				assert_eq!(old, NOTIFIED, "park state changed unexpectedly");
				return UnparkReason::Unparked;
			}
			Err(n) => panic!("inconsistent park_timeout state: {n}"),
		}

		loop {
			m = match deadline {
				None => self.cvar.wait(m).unwrap(),
				Some(deadline) => {
					let now = Instant::now();
					if now < deadline {
						self.cvar.wait_timeout(m, deadline - now).unwrap().0
					} else {
						return match self.state.swap(EMPTY, SeqCst) {
							NOTIFIED => UnparkReason::Unparked,
							PARKED => UnparkReason::Timeout,
							n => panic!("inconsistent park_timeout state: {n}"),
						};
					}
				}
			};

			if self
				.state
				.compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
				.is_ok()
			{
				return UnparkReason::Unparked;
			}
		}
	}

	pub fn unpark(&self) {
		match self.state.swap(NOTIFIED, SeqCst) {
			EMPTY | NOTIFIED => return,
			PARKED => {}
			_ => panic!("inconsistent state in unpark"),
		}

		drop(self.lock.lock().unwrap());
		self.cvar.notify_one();
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnparkReason {
	Unparked,
	Timeout,
}
