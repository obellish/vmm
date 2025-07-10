#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::{
	any::TypeId,
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	mem::{self, MaybeUninit},
};

pub struct Discriminant {
	data: MaybeUninit<*mut ()>,
	vtable: &'static DiscriminantVTable,
}

impl Discriminant {
	pub fn of<T>(value: &T) -> Self {
		let discriminant = mem::discriminant(value);
		let data = if small_discriminant::<T>() {
			let mut data = MaybeUninit::<*mut ()>::uninit();
			unsafe {
				data.as_mut_ptr()
					.cast::<mem::Discriminant<T>>()
					.write(discriminant);
			}
			data
		} else {
			MaybeUninit::new(Box::into_raw(Box::new(discriminant)).cast::<()>())
		};

		Self {
			data,
			vtable: &DiscriminantVTable {
				eq: discriminant_eq::<T>,
				hash: discriminant_hash::<T>,
				clone: discriminant_clone::<T>,
				debug: discriminant_debug::<T>,
				drop: discriminant_drop::<T>,
				type_id: typeid::of::<mem::Discriminant<T>>,
			},
		}
	}
}

impl Clone for Discriminant {
	fn clone(&self) -> Self {
		unsafe { (self.vtable.clone)(self) }
	}
}

impl Debug for Discriminant {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		unsafe { (self.vtable.debug)(self, f) }
	}
}

impl Drop for Discriminant {
	fn drop(&mut self) {
		unsafe { (self.vtable.drop)(self) }
	}
}

impl Eq for Discriminant {}

impl Hash for Discriminant {
	fn hash<H: Hasher>(&self, state: &mut H) {
		unsafe { (self.vtable.hash)(self, state) }
	}
}

impl PartialEq for Discriminant {
	fn eq(&self, other: &Self) -> bool {
		unsafe { (self.vtable.eq)(self, other) }
	}
}

struct DiscriminantVTable {
	eq: unsafe fn(this: &Discriminant, other: &Discriminant) -> bool,
	hash: unsafe fn(this: &Discriminant, hasher: &mut dyn Hasher),
	clone: unsafe fn(this: &Discriminant) -> Discriminant,
	debug: unsafe fn(this: &Discriminant, formatter: &mut Formatter<'_>) -> FmtResult,
	drop: unsafe fn(this: &mut Discriminant),
	type_id: fn() -> TypeId,
}

const unsafe fn as_ref<T>(this: &Discriminant) -> &mem::Discriminant<T> {
	unsafe {
		&*if small_discriminant::<T>() {
			this.data.as_ptr().cast::<mem::Discriminant<T>>()
		} else {
			this.data.assume_init().cast::<mem::Discriminant<T>>()
		}
	}
}

unsafe fn discriminant_eq<T>(this: &Discriminant, other: &Discriminant) -> bool {
	(other.vtable.type_id)() == typeid::of::<mem::Discriminant<T>>()
		&& unsafe { as_ref::<T>(this) } == unsafe { as_ref::<T>(other) }
}

unsafe fn discriminant_hash<T>(this: &Discriminant, mut hasher: &mut dyn Hasher) {
	typeid::of::<mem::Discriminant<T>>().hash(&mut hasher);
	unsafe { as_ref::<T>(this) }.hash(&mut hasher);
}

unsafe fn discriminant_clone<T>(this: &Discriminant) -> Discriminant {
	if small_discriminant::<T>() {
		Discriminant {
			data: this.data,
			vtable: this.vtable,
		}
	} else {
		let discriminant = unsafe { *this.data.assume_init().cast::<mem::Discriminant<T>>() };
		Discriminant {
			data: MaybeUninit::new(Box::into_raw(Box::new(discriminant)).cast::<()>()),
			vtable: this.vtable,
		}
	}
}

unsafe fn discriminant_debug<T>(this: &Discriminant, formatter: &mut Formatter<'_>) -> FmtResult {
	Debug::fmt(unsafe { as_ref::<T>(this) }, formatter)
}

unsafe fn discriminant_drop<T>(this: &mut Discriminant) {
	if !small_discriminant::<T>() {
		_ = unsafe { Box::from_raw(this.data.assume_init().cast::<mem::Discriminant<T>>()) };
	}
}

const fn small_discriminant<T>() -> bool {
	mem::size_of::<mem::Discriminant<T>>() <= mem::size_of::<*const ()>()
}

#[cfg(test)]
mod tests {
	extern crate std;

	use alloc::borrow::ToOwned;
	use std::collections::HashSet;

	use super::Discriminant;

	enum Enum<'a> {
		A(#[allow(dead_code)] &'a str),
		B,
	}

	enum DifferentEnum {
		A,
	}

	#[test]
	fn eq() {
		let temporary_string = "...".to_owned();
		let a = Enum::A(&temporary_string);
		let b = Enum::B;
		let a_discriminant = Discriminant::of(&a);
		let b_discriminant = Discriminant::of(&b);
		drop(temporary_string);
		assert_eq!(a_discriminant, a_discriminant);
		assert_ne!(a_discriminant, b_discriminant);

		let different_discriminant = Discriminant::of(&DifferentEnum::A);
		assert_ne!(a_discriminant, different_discriminant);
		assert_ne!(b_discriminant, different_discriminant);
	}

	#[test]
	fn hashset() {
		let mut set = HashSet::new();

		let temporary_string = "...".to_owned();
		set.insert(Discriminant::of(&Enum::A(&temporary_string)));
		set.insert(Discriminant::of(&Enum::B));
		set.insert(Discriminant::of(&DifferentEnum::A));
		drop(temporary_string);
		assert_eq!(set.len(), 3);

		set.insert(Discriminant::of(&Enum::A("other string")));
		set.insert(Discriminant::of(&Enum::B));
		set.insert(Discriminant::of(&DifferentEnum::A));
		assert_eq!(set.len(), 3);

		assert_eq!(set, set.clone());
	}
}
