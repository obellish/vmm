#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod inner;

pub use self::inner::VmString;

#[derive(Debug, Clone, Copy)]
pub struct VmStatus<O, V = ()> {
	pub done: bool,
	pub last_command: O,
	pub value: Option<V>,
}

pub trait Vm<Op, Val, T> {
	type Error;
}

pub trait Logical: Clone + Sized {
	fn and(&self, b: &Self) -> bool;

	fn or(&self, b: &Self) -> bool;

	fn xor(&self, b: &Self) -> bool;

	fn not(&self) -> bool;

	#[must_use]
	fn logical_and(&self, b: &Self) -> Self {
		if self.and(b) { self.clone() } else { b.clone() }
	}

	#[must_use]
	fn logical_or(&self, b: &Self) -> Self {
		if self.or(b) { self.clone() } else { b.clone() }
	}

	#[must_use]
	fn logical_xor(&self, b: &Self) -> Self {
		if self.xor(b) { self.clone() } else { b.clone() }
	}

	#[must_use]
	fn bitwise_reverse(&self) -> Self;
}
