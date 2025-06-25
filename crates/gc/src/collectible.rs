use std::{
	mem,
	ptr::{self, NonNull},
	sync::atomic::{AtomicPtr, AtomicUsize, Ordering::Relaxed},
};

#[derive(Debug, Default)]
pub struct Link {
	data: (AtomicUsize, AtomicPtr<usize>),
}

impl Link {
	pub(super) const fn shared() -> Self {
		Self {
			data: (AtomicUsize::new(1), AtomicPtr::new(ptr::null_mut())),
		}
	}

	pub(super) const fn unique() -> Self {
		Self {
			data: (AtomicUsize::new(0), AtomicPtr::new(ptr::null_mut())),
		}
	}

	pub(super) const fn reference_count(&self) -> &AtomicUsize {
		&self.data.0
	}
}

#[allow(clippy::transmute_undefined_repr)]
impl Collectible for Link {
	fn next_ptr(&self) -> Option<NonNull<dyn Collectible>> {
		let fat_ptr: (*mut usize, *mut usize) = (
			self.data.0.load(Relaxed) as *mut usize,
			self.data.1.load(Relaxed),
		);

		unsafe { mem::transmute(fat_ptr) }
	}

	fn set_next_ptr(&self, next_ptr: Option<NonNull<dyn Collectible>>) {
		let data: (*mut usize, *mut usize) = next_ptr.map_or_else(
			|| (ptr::null_mut(), ptr::null_mut()),
			|p| unsafe { mem::transmute(p) },
		);

		self.data.0.store(data.0 as usize, Relaxed);
		self.data.1.store(data.1, Relaxed);
	}
}

pub(super) struct DeferredClosure<F>
where
	F: FnOnce() + 'static,
{
	closure: Option<F>,
	link: Link,
}

impl<F> DeferredClosure<F>
where
	F: FnOnce() + 'static,
{
	pub fn new(closure: F) -> Self {
		Self {
			closure: Some(closure),
			link: Link::default(),
		}
	}
}

impl<F> Collectible for DeferredClosure<F>
where
	F: FnOnce() + 'static,
{
	fn next_ptr(&self) -> Option<NonNull<dyn Collectible>> {
		self.link.next_ptr()
	}

	fn set_next_ptr(&self, next_ptr: Option<NonNull<dyn Collectible>>) {
		self.link.set_next_ptr(next_ptr);
	}
}

impl<F> Drop for DeferredClosure<F>
where
	F: FnOnce() + 'static,
{
	fn drop(&mut self) {
		if let Some(f) = self.closure.take() {
			f();
		}
	}
}

pub(super) trait Collectible {
	fn next_ptr(&self) -> Option<NonNull<dyn Collectible>>;

	fn set_next_ptr(&self, next_ptr: Option<NonNull<dyn Collectible>>);
}
