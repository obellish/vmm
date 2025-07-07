use core::{cmp::Ordering, num::TryFromIntError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Integer(i128);

impl Integer {
	fn canonical_len(self) -> usize {
		let Self(x) = self;

		if let Ok(x) = u8::try_from(x) {
			if x < 24 { 1 } else { 2 }
		} else if let Ok(x) = i8::try_from(x) {
			if x >= -24i8 { 1 } else { 2 }
		} else if u16::try_from(x).is_ok() || i16::try_from(x).is_ok() {
			3
		} else if u32::try_from(x).is_ok() || i32::try_from(x).is_ok() {
			5
		} else if u64::try_from(x).is_ok() || i64::try_from(x).is_ok() {
			9
		} else {
			x.to_be_bytes().len() + 1
		}
	}

	#[must_use]
	pub fn canonical_cmp(self, other: Self) -> Ordering {
		self.canonical_len()
			.cmp(&other.canonical_len())
			.then_with(|| match (self.0.is_negative(), other.0.is_negative()) {
				(false, true) => Ordering::Less,
				(true, false) => Ordering::Greater,
				(true, true) => self.0.cmp(&other.0).reverse(),
				_ => self.0.cmp(&other.0),
			})
	}
}

impl From<u8> for Integer {
	fn from(value: u8) -> Self {
		Self(value.into())
	}
}

impl From<u16> for Integer {
	fn from(value: u16) -> Self {
		Self(value.into())
	}
}

impl From<u32> for Integer {
	fn from(value: u32) -> Self {
		Self(value.into())
	}
}

impl From<u64> for Integer {
	fn from(value: u64) -> Self {
		Self(value.into())
	}
}

impl From<i8> for Integer {
	fn from(value: i8) -> Self {
		Self(value.into())
	}
}

impl From<i16> for Integer {
	fn from(value: i16) -> Self {
		Self(value.into())
	}
}

impl From<i32> for Integer {
	fn from(value: i32) -> Self {
		Self(value.into())
	}
}

impl From<i64> for Integer {
	fn from(value: i64) -> Self {
		Self(value.into())
	}
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl From<usize> for Integer {
	fn from(value: usize) -> Self {
		Self(value as _)
	}
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl From<isize> for Integer {
	fn from(value: isize) -> Self {
		Self(value as _)
	}
}

impl From<Integer> for i128 {
	fn from(value: Integer) -> Self {
		value.0
	}
}

impl TryFrom<Integer> for u8 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for u16 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for u32 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for u64 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for i8 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for i16 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for i32 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for i64 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for usize {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for isize {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<Integer> for u128 {
	type Error = TryFromIntError;

	fn try_from(value: Integer) -> Result<Self, Self::Error> {
		Self::try_from(value.0)
	}
}

impl TryFrom<i128> for Integer {
	type Error = TryFromIntError;

	fn try_from(value: i128) -> Result<Self, Self::Error> {
		u64::try_from(if value.is_negative() {
			value ^ !0
		} else {
			value
		})?;

		Ok(Self(value))
	}
}

impl TryFrom<u128> for Integer {
	type Error = TryFromIntError;

	fn try_from(value: u128) -> Result<Self, Self::Error> {
		Ok(Self(u64::try_from(value)?.into()))
	}
}
