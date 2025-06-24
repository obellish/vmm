use core::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
};

pub trait Tap
where
	Self: Sized,
{
	fn tap(self, func: impl FnOnce(&Self)) -> Self {
		func(&self);
		self
	}

	fn tap_mut(mut self, func: impl FnOnce(&mut Self)) -> Self {
		func(&mut self);
		self
	}

	fn tap_borrowed<B: ?Sized>(self, func: impl FnOnce(&B)) -> Self
	where
		Self: Borrow<B>,
	{
		func(Borrow::borrow(&self));
		self
	}

	fn tap_borrowed_mut<B: ?Sized>(mut self, func: impl FnOnce(&mut B)) -> Self
	where
		Self: BorrowMut<B>,
	{
		func(BorrowMut::borrow_mut(&mut self));
		self
	}

	fn tap_as_ref<R: ?Sized>(self, func: impl FnOnce(&R)) -> Self
	where
		Self: AsRef<R>,
	{
		func(AsRef::as_ref(&self));
		self
	}

	fn tap_as_mut<R: ?Sized>(mut self, func: impl FnOnce(&mut R)) -> Self
	where
		Self: AsMut<R>,
	{
		func(AsMut::as_mut(&mut self));
		self
	}

	fn tap_deref<T: ?Sized>(self, func: impl FnOnce(&T)) -> Self
	where
		Self: Deref<Target = T>,
	{
		func(Deref::deref(&self));
		self
	}

	fn tap_deref_mut<T: ?Sized>(mut self, func: impl FnOnce(&mut T)) -> Self
	where
		Self: DerefMut<Target = T>,
	{
		func(DerefMut::deref_mut(&mut self));
		self
	}

	fn debug_tap(self, func: impl FnOnce(&Self)) -> Self {
		if cfg!(debug_assertions) {
			self.tap(func)
		} else {
			self
		}
	}

	fn debug_tap_mut(self, func: impl FnOnce(&mut Self)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_mut(func)
		} else {
			self
		}
	}

	fn debug_tap_borrowed<B: ?Sized>(self, func: impl FnOnce(&B)) -> Self
	where
		Self: Borrow<B>,
	{
		if cfg!(debug_assertions) {
			self.tap_borrowed(func)
		} else {
			self
		}
	}

	fn debug_tap_borrowed_mut<B: ?Sized>(self, func: impl FnOnce(&mut B)) -> Self
	where
		Self: BorrowMut<B>,
	{
		if cfg!(debug_assertions) {
			self.tap_borrowed_mut(func)
		} else {
			self
		}
	}

	fn debug_tap_as_ref<R: ?Sized>(self, func: impl FnOnce(&R)) -> Self
	where
		Self: AsRef<R>,
	{
		if cfg!(debug_assertions) {
			self.tap_as_ref(func)
		} else {
			self
		}
	}

	fn debug_tap_as_mut<R: ?Sized>(self, func: impl FnOnce(&mut R)) -> Self
	where
		Self: AsMut<R>,
	{
		if cfg!(debug_assertions) {
			self.tap_as_mut(func)
		} else {
			self
		}
	}

	fn debug_tap_deref<T: ?Sized>(self, func: impl FnOnce(&T)) -> Self
	where
		Self: Deref<Target = T>,
	{
		if cfg!(debug_assertions) {
			self.tap_deref(func)
		} else {
			self
		}
	}

	fn debug_tap_deref_mut<T: ?Sized>(self, func: impl FnOnce(&mut T)) -> Self
	where
		Self: DerefMut<Target = T>,
	{
		if cfg!(debug_assertions) {
			self.tap_deref_mut(func)
		} else {
			self
		}
	}
}

impl<T> Tap for T {}

pub trait FallibleTap<T: ?Sized, E: ?Sized>
where
	Self: Sized,
{
	fn tap_ok(self, func: impl FnOnce(&T)) -> Self;

	fn tap_ok_mut(self, func: impl FnOnce(&mut T)) -> Self;

	fn tap_err(self, func: impl FnOnce(&E)) -> Self;

	fn tap_err_mut(self, func: impl FnOnce(&mut E)) -> Self;

	fn debug_tap_ok(self, func: impl FnOnce(&T)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_ok(func)
		} else {
			self
		}
	}

	fn debug_tap_ok_mut(self, func: impl FnOnce(&mut T)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_ok_mut(func)
		} else {
			self
		}
	}

	fn debug_tap_err(self, func: impl FnOnce(&E)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_err(func)
		} else {
			self
		}
	}

	fn debug_tap_err_mut(self, func: impl FnOnce(&mut E)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_err_mut(func)
		} else {
			self
		}
	}
}

impl<T, E> FallibleTap<T, E> for Result<T, E> {
	fn tap_ok(self, func: impl FnOnce(&T)) -> Self {
		SpecFallibleTap::spec_tap_ok(self, func)
	}

	fn tap_ok_mut(self, func: impl FnOnce(&mut T)) -> Self {
		SpecFallibleTap::spec_tap_ok_mut(self, func)
	}

	fn tap_err(self, func: impl FnOnce(&E)) -> Self {
		SpecFallibleTap::spec_tap_err(self, func)
	}

	fn tap_err_mut(self, func: impl FnOnce(&mut E)) -> Self {
		SpecFallibleTap::spec_tap_err_mut(self, func)
	}
}

pub trait OptionalTap<T: ?Sized>
where
	Self: Sized,
{
	fn tap_some(self, func: impl FnOnce(&T)) -> Self;

	fn tap_some_mut(self, func: impl FnOnce(&mut T)) -> Self;

	fn tap_none(self, func: impl FnOnce()) -> Self;

	fn debug_tap_some(self, func: impl FnOnce(&T)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_some(func)
		} else {
			self
		}
	}

	fn debug_tap_some_mut(self, func: impl FnOnce(&mut T)) -> Self {
		if cfg!(debug_assertions) {
			self.tap_some_mut(func)
		} else {
			self
		}
	}

	fn debug_tap_none(self, func: impl FnOnce()) -> Self {
		if cfg!(debug_assertions) {
			self.tap_none(func)
		} else {
			self
		}
	}
}

impl<T> OptionalTap<T> for Option<T> {
	fn tap_some(self, func: impl FnOnce(&T)) -> Self {
		SpecOptionalTap::spec_tap_some(self, func)
	}

	fn tap_some_mut(self, func: impl FnOnce(&mut T)) -> Self {
		SpecOptionalTap::spec_tap_some_mut(self, func)
	}

	fn tap_none(self, func: impl FnOnce()) -> Self {
		SpecOptionalTap::spec_tap_none(self, func)
	}
}

trait SpecOptionalTap<T: ?Sized> {
	fn spec_tap_some(self, func: impl FnOnce(&T)) -> Self;

	fn spec_tap_some_mut(self, func: impl FnOnce(&mut T)) -> Self;

	fn spec_tap_none(self, func: impl FnOnce()) -> Self;
}

#[cfg(not(feature = "nightly"))]
impl<T> SpecOptionalTap<T> for Option<T> {
	fn spec_tap_some(self, func: impl FnOnce(&T)) -> Self {
		if let Some(ref val) = self {
			func(val);
		}

		self
	}

	fn spec_tap_some_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		if let Some(ref mut val) = self {
			func(val);
		}

		self
	}

	fn spec_tap_none(self, func: impl FnOnce()) -> Self {
		if self.is_none() {
			func();
		}

		self
	}
}

#[cfg(feature = "nightly")]
impl<T> SpecOptionalTap<T> for Option<T> {
	default fn spec_tap_some(self, func: impl FnOnce(&T)) -> Self {
		if let Some(ref val) = self {
			func(val);
		}

		self
	}

	default fn spec_tap_some_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		if let Some(ref mut val) = self {
			func(val);
		}

		self
	}

	default fn spec_tap_none(self, func: impl FnOnce()) -> Self {
		if self.is_none() {
			func();
		}

		self
	}
}

#[cfg(feature = "nightly")]
impl SpecOptionalTap<!> for Option<!> {
	fn spec_tap_some(self, _: impl FnOnce(&!)) -> Self {
		self
	}

	fn spec_tap_some_mut(self, _: impl FnOnce(&mut !)) -> Self {
		self
	}

	fn spec_tap_none(self, func: impl FnOnce()) -> Self {
		func();
		self
	}
}

#[cfg(feature = "nightly")]
impl SpecOptionalTap<core::convert::Infallible> for Option<core::convert::Infallible> {
	fn spec_tap_some(self, _: impl FnOnce(&core::convert::Infallible)) -> Self {
		self
	}

	fn spec_tap_some_mut(self, _: impl FnOnce(&mut core::convert::Infallible)) -> Self {
		self
	}

	fn spec_tap_none(self, func: impl FnOnce()) -> Self {
		func();
		self
	}
}

trait SpecFallibleTap<T: ?Sized, E: ?Sized> {
	fn spec_tap_ok(self, func: impl FnOnce(&T)) -> Self;

	fn spec_tap_ok_mut(self, func: impl FnOnce(&mut T)) -> Self;

	fn spec_tap_err(self, func: impl FnOnce(&E)) -> Self;

	fn spec_tap_err_mut(self, func: impl FnOnce(&mut E)) -> Self;
}

#[cfg(not(feature = "nightly"))]
impl<T, E> SpecFallibleTap<T, E> for Result<T, E> {
	fn spec_tap_ok(self, func: impl FnOnce(&T)) -> Self {
		if let Ok(ref val) = self {
			func(val);
		}

		self
	}

	fn spec_tap_ok_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		if let Ok(ref mut val) = self {
			func(val);
		}

		self
	}

	fn spec_tap_err(self, func: impl FnOnce(&E)) -> Self {
		if let Err(ref val) = self {
			func(val);
		}

		self
	}

	fn spec_tap_err_mut(mut self, func: impl FnOnce(&mut E)) -> Self {
		if let Err(ref mut val) = self {
			func(val);
		}

		self
	}
}

#[cfg(feature = "nightly")]
impl<T, E> SpecFallibleTap<T, E> for Result<T, E> {
	default fn spec_tap_ok(self, func: impl FnOnce(&T)) -> Self {
		if let Ok(ref val) = self {
			func(val);
		}

		self
	}

	default fn spec_tap_ok_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		if let Ok(ref mut val) = self {
			func(val);
		}

		self
	}

	default fn spec_tap_err(self, func: impl FnOnce(&E)) -> Self {
		if let Err(ref val) = self {
			func(val);
		}

		self
	}

	default fn spec_tap_err_mut(mut self, func: impl FnOnce(&mut E)) -> Self {
		if let Err(ref mut val) = self {
			func(val);
		}

		self
	}
}

#[cfg(feature = "nightly")]
impl<T> SpecFallibleTap<T, !> for Result<T, !> {
	fn spec_tap_ok(self, func: impl FnOnce(&T)) -> Self {
		let Ok(ref val) = self;
		func(val);
		self
	}

	fn spec_tap_ok_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		let Ok(ref mut val) = self;
		func(val);
		self
	}

	fn spec_tap_err(self, _: impl FnOnce(&!)) -> Self {
		self
	}

	fn spec_tap_err_mut(self, _: impl FnOnce(&mut !)) -> Self {
		self
	}
}

#[cfg(feature = "nightly")]
impl<T> SpecFallibleTap<T, core::convert::Infallible> for Result<T, core::convert::Infallible> {
	fn spec_tap_ok(self, func: impl FnOnce(&T)) -> Self {
		let Ok(ref val) = self;
		func(val);
		self
	}

	fn spec_tap_ok_mut(mut self, func: impl FnOnce(&mut T)) -> Self {
		let Ok(ref mut val) = self;
		func(val);
		self
	}

	fn spec_tap_err_mut(self, _: impl FnOnce(&mut core::convert::Infallible)) -> Self {
		self
	}

	fn spec_tap_err(self, _: impl FnOnce(&core::convert::Infallible)) -> Self {
		self
	}
}

#[cfg(test)]
mod tests {
	extern crate alloc;

	use alloc::vec::Vec;

	use super::{FallibleTap, OptionalTap, Tap};

	#[test]
	fn filter_map() {
		let values: &[Result<i32, &str>] = &[Ok(3), Err("foo"), Err("bar"), Ok(8)];

		let mut errors = Vec::new();

		let sum = values
			.iter()
			.filter_map(|r| r.tap_err(|e| errors.push(*e)).ok())
			.sum::<i32>();

		assert_eq!(errors, ["foo", "bar"]);
		assert_eq!(sum, 11);
	}

	#[test]
	fn basic() {
		let mut val = 5;

		if 10.tap(|v| val += *v) > 0 {
			assert_eq!(val, 15);
		}

		let _: Result<i32, i32> = Err(5).tap_err(|e| val = *e);
		assert_eq!(val, 5);

		let _: Option<i32> = None.tap_none(|| val = 10);
		assert_eq!(val, 10);
	}

	#[cfg(feature = "nightly")]
	#[expect(clippy::uninhabited_references)]
	#[test]
	fn spec() {
		let res: Result<i32, !> = Ok(3);

		_ = res.tap_ok(|v| assert_eq!(*v, 3));

		_ = res.tap_err(|v| match *v {});

		let res: Result<i32, core::convert::Infallible> = Ok(5);

		_ = res.tap_ok(|v| assert_eq!(*v, 5));

		_ = res.tap_err(|v| match *v {});
	}
}
