#![allow(unused)]

use core::{
	fmt::{Debug, Formatter, Pointer, Result as FmtResult},
	marker::PhantomData,
	ptr::NonNull,
};

#[repr(transparent)]
pub struct Unique<T: ?Sized> {
	inner: NonNull<T>,
	marker: PhantomData<T>,
}

impl<T: ?Sized> Unique<T> {
	pub const fn new(ptr: *mut T) -> Option<Self> {
		if let Some(inner) = NonNull::new(ptr) {
			Some(Self {
				inner,
				marker: PhantomData,
			})
		} else {
			None
		}
	}

	pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
		unsafe {
			Self {
				inner: NonNull::new_unchecked(ptr),
				marker: PhantomData,
			}
		}
	}

	pub const fn from_non_null(inner: NonNull<T>) -> Self {
		Self {
			inner,
			marker: PhantomData,
		}
	}

	pub const fn as_ptr(self) -> *mut T {
		self.inner.as_ptr()
	}

	pub const fn as_non_null_ptr(self) -> NonNull<T> {
		self.inner
	}

	pub const unsafe fn as_ref(&self) -> &T {
		unsafe { self.inner.as_ref() }
	}

	pub const unsafe fn as_mut(&mut self) -> &mut T {
		unsafe { self.inner.as_mut() }
	}

	pub const fn cast<U>(self) -> Unique<U> {
		Unique {
			inner: self.inner.cast(),
			marker: PhantomData,
		}
	}
}

impl<T> Unique<T> {
	pub const fn dangling() -> Self {
		Self {
			inner: NonNull::dangling(),
			marker: PhantomData,
		}
	}

	#[cfg_attr(miri, track_caller)]
	pub const unsafe fn add(self, count: usize) -> Self {
		unsafe {
			Self {
				inner: self.inner.add(count),
				marker: PhantomData,
			}
		}
	}

	#[cfg_attr(miri, track_caller)]
	pub const unsafe fn read(self) -> T {
		unsafe { self.inner.read() }
	}

	#[cfg_attr(miri, track_caller)]
	pub const unsafe fn write(self, value: T) {
		unsafe { self.inner.write(value) }
	}
}

impl<T: ?Sized> Clone for Unique<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Unique<T> {}

impl<T: ?Sized> Debug for Unique<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Pointer::fmt(&self, f)
	}
}

impl<T: ?Sized> Eq for Unique<T> {}

impl<T: ?Sized> From<&mut T> for Unique<T> {
	fn from(value: &mut T) -> Self {
		Self::from(NonNull::from(value))
	}
}

impl<T: ?Sized> From<NonNull<T>> for Unique<T> {
	fn from(value: NonNull<T>) -> Self {
		Self::from_non_null(value)
	}
}

impl<T: ?Sized> PartialEq for Unique<T> {
	fn eq(&self, other: &Self) -> bool {
		core::ptr::eq(&self.inner.as_ptr(), &other.inner.as_ptr())
	}
}

impl<T: ?Sized> Pointer for Unique<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Pointer::fmt(&self.inner, f)
	}
}

unsafe impl<T> Send for Unique<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for Unique<T> where T: ?Sized + Send {}
