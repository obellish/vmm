use core::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
};

pub trait Pipe {
	fn pipe<R>(self, func: impl FnOnce(Self) -> R) -> R
	where
		Self: Sized,
	{
		func(self)
	}

	fn pipe_ref<'a, R>(&'a self, func: impl FnOnce(&'a Self) -> R) -> R {
		func(self)
	}

	fn pipe_mut<'a, R>(&'a mut self, func: impl FnOnce(&'a mut Self) -> R) -> R {
		func(self)
	}

	fn pipe_borrowed<'a, B, R>(&'a self, func: impl FnOnce(&'a B) -> R) -> R
	where
		Self: Borrow<B>,
		B: ?Sized + 'a,
	{
		func(Borrow::borrow(self))
	}

	fn pipe_borrowed_mut<'a, B, R>(&'a mut self, func: impl FnOnce(&'a mut B) -> R) -> R
	where
		Self: BorrowMut<B>,
		B: ?Sized + 'a,
	{
		func(BorrowMut::borrow_mut(self))
	}

	fn pipe_as_ref<'a, U, R>(&'a self, func: impl FnOnce(&'a U) -> R) -> R
	where
		Self: AsRef<U>,
		U: ?Sized + 'a,
	{
		func(AsRef::as_ref(self))
	}

	fn pipe_as_mut<'a, U, R>(&'a mut self, func: impl FnOnce(&'a mut U) -> R) -> R
	where
		Self: AsMut<U>,
		U: ?Sized + 'a,
	{
		func(AsMut::as_mut(self))
	}

	fn pipe_deref<'a, T, R>(&'a self, func: impl FnOnce(&'a T) -> R) -> R
	where
		Self: Deref<Target = T>,
		T: ?Sized + 'a,
	{
		func(&**self)
	}

	fn pipe_deref_mut<'a, T, R>(&'a mut self, func: impl FnOnce(&'a mut T) -> R) -> R
	where
		Self: DerefMut<Target = T>,
		T: ?Sized + 'a,
	{
		func(&mut **self)
	}
}

impl<T: ?Sized> Pipe for T {}
