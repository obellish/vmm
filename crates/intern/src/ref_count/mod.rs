use std::{
	alloc::{Layout, alloc, dealloc},
	cell::UnsafeCell,
	fmt::{Formatter, Pointer, Result as FmtResult},
	mem,
	ops::Deref,
	ptr::{self, NonNull},
	sync::{Arc, Weak},
};

use parking_lot::lock_api::RawMutex;

mod private;

pub struct Interned<I: Interner> {
	inner: NonNull<RefCounted<I>>,
}

impl<I: Interner> Interned<I> {
	pub(crate) fn ref_count(&self) -> u32 {
		self.lock().refs()
	}

	fn lock(&self) -> Guard<'_, I> {
		unsafe { self.inner.as_ref().state.lock() }
	}

	pub(crate) fn from_box(value: Box<I::T>) -> Self {
		Self {
			inner: RefCounted::from_box(value),
		}
	}

	pub(crate) fn from_sized(value: I::T) -> Self
	where
		I::T: Sized,
	{
		Self {
			inner: RefCounted::from_sized(value),
		}
	}

	#[allow(clippy::needless_pass_by_ref_mut)]
	pub(crate) fn make_hot(&mut self, set: &Arc<I>) {
		let mut state = self.lock();
		*state.cleanup() = Some(Arc::downgrade(set));
	}
}

const MAX_REFCOUNT: u32 = u32::MAX - 2;

impl<I: Interner> Clone for Interned<I> {
	fn clone(&self) -> Self {
		let refs = {
			let mut state = self.lock();
			*state.refs_mut() += 1;
			state.refs()
		};

		assert!((refs <= MAX_REFCOUNT), "too many clones");

		Self { inner: self.inner }
	}
}

impl<I: Interner> Deref for Interned<I> {
	type Target = I::T;

	fn deref(&self) -> &Self::Target {
		&unsafe { self.inner.as_ref() }.value
	}
}

impl<I: Interner> Drop for Interned<I> {
	fn drop(&mut self) {
		let mut state = self.lock();

		*state.refs_mut() -= 1;

		match state.refs() {
			0 => {
				drop(state);
				_ = unsafe { Box::from_raw(self.inner.as_ptr()) };
			}
			1 => {
				if let Some(cleanup) = state.cleanup().take() {
					if let Some(strong) = cleanup.upgrade() {
						drop(state);
						loop {
							let (removed, _) = strong.remove(self);
							if removed {
								break;
							}
							let mut state = self.lock();
							if state.refs() > 1 {
								*state.cleanup() = Some(cleanup);
								break;
							}
							drop(state);
						}
					}
				}
			}
			_ => {}
		}
	}
}

impl<I: Interner> PartialEq for Interned<I> {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self.inner.as_ptr(), other.inner.as_ptr())
	}
}

impl<I: Interner> Pointer for Interned<I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Pointer::fmt(&(&raw const **self), f)
	}
}

unsafe impl<I: Interner> Send for Interned<I> where I::T: Send + Sync + 'static {}
unsafe impl<I: Interner> Sync for Interned<I> where I::T: Send + Sync + 'static {}

#[repr(C)]
struct RefCounted<I: Interner> {
	state: State<I>,
	value: I::T,
}

impl<I: Interner> RefCounted<I> {
	fn from_box(value: Box<I::T>) -> NonNull<Self> {
		let layout = Layout::new::<RefCounted<()>>()
			.extend(Layout::for_value(value.as_ref()))
			.unwrap()
			.0
			.pad_to_align();

		unsafe {
			let ptr = alloc(layout);

			let b = Box::leak(value) as *mut I::T;

			let ptr = {
				let mut temp = b as *mut Self;

				ptr::write((&raw mut temp).cast::<*mut u8>(), ptr);
				temp
			};

			ptr::write(&mut (*ptr).state, State::new());
			let num_bytes = mem::size_of_val(&*b);
			if num_bytes > 0 {
				ptr::copy_nonoverlapping(
					b as *const u8,
					(&raw mut (*ptr).value).cast::<u8>(),
					num_bytes,
				);

				dealloc(b.cast::<u8>(), Layout::for_value(&*b));
			}

			NonNull::new_unchecked(ptr)
		}
	}

	fn from_sized(value: I::T) -> NonNull<Self>
	where
		I::T: Sized,
	{
		let b = Box::new(Self {
			state: State::new(),
			value,
		});

		NonNull::from(Box::leak(b))
	}
}

struct State<I> {
	mutex: parking_lot::RawMutex,
	refs: UnsafeCell<u32>,
	cleanup: UnsafeCell<Option<Weak<I>>>,
}

impl<I: Interner> State<I> {
	pub const fn new() -> Self {
		Self {
			mutex: parking_lot::RawMutex::INIT,
			refs: UnsafeCell::new(1),
			cleanup: UnsafeCell::new(None),
		}
	}

	pub fn lock(&self) -> Guard<'_, I> {
		self.mutex.lock();
		Guard(self)
	}
}

struct Guard<'a, I>(&'a State<I>);

impl<I> Guard<'_, I> {
	pub fn refs(&self) -> u32 {
		unsafe { *self.0.refs.get() }
	}

	pub fn refs_mut(&mut self) -> &mut u32 {
		unsafe { &mut *self.0.refs.get() }
	}

	pub fn cleanup(&mut self) -> &mut Option<Weak<I>> {
		unsafe { &mut *self.0.cleanup.get() }
	}
}

impl<I> Drop for Guard<'_, I> {
	fn drop(&mut self) {
		unsafe {
			self.0.mutex.unlock();
		}
	}
}

pub trait Interner: self::private::Sealed + Sized {
	type T: ?Sized;

	fn remove(&self, value: &Interned<Self>) -> (bool, Option<Interned<Self>>);
}

impl Interner for () {
	type T = ();

	fn remove(&self, _: &Interned<Self>) -> (bool, Option<Interned<Self>>) {
		(false, None)
	}
}
