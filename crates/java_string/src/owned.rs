use std::{
	borrow::{Borrow, BorrowMut, Cow},
	collections::{Bound, TryReserveError},
	convert::Infallible,
	fmt::{Debug, Display, Formatter, Write},
	hash::{Hash, Hasher},
	iter::FusedIterator,
	ops::{
		Add, AddAssign, Deref, DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull,
		RangeInclusive, RangeTo, RangeToInclusive,
	},
	ptr,
	rc::Rc,
	slice,
	str::FromStr,
	sync::Arc,
};

use super::{
	FromUtf8Error, JavaCodePoint, JavaStr, Utf8Error, validations::run_utf8_semi_validation,
};

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct JavaString {
	inner: Vec<u8>,
}

impl JavaString {
	#[must_use]
	pub const fn new() -> Self {
		Self { inner: Vec::new() }
	}

	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			inner: Vec::with_capacity(capacity),
		}
	}

	pub fn from_full_utf8(v: Vec<u8>) -> Result<Self, FromUtf8Error> {
		match std::str::from_utf8(&v) {
			Ok(..) => Ok(Self { inner: v }),
			Err(e) => Err(FromUtf8Error {
				bytes: v,
				error: e.into(),
			}),
		}
	}

	pub fn from_semi_utf8(v: Vec<u8>) -> Result<Self, FromUtf8Error> {
		match run_utf8_semi_validation(&v) {
			Ok(..) => Ok(Self { inner: v }),
			Err(e) => Err(FromUtf8Error { bytes: v, error: e }),
		}
	}
}
