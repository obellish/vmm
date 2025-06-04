use std::{
	alloc::{Layout, alloc, dealloc},
	cell::UnsafeCell,
	mem,
	ptr::{self, NonNull},
	sync::Weak,
};

use parking_lot::lock_api::RawMutex;

mod private;

pub struct Interned<I: Interner> {
	inner: NonNull<RefCounted<I>>,
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

	fn from_slice(value: I::T) -> NonNull<Self>
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
