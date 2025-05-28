use core::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PcRelOffset(i32);

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for PcRelOffset {
	fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		Ok(Self(0))
	}
}

impl From<i32> for PcRelOffset {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<PcRelOffset> for i32 {
	fn from(value: PcRelOffset) -> Self {
		value.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct U6(u8);

impl U6 {
	#[must_use]
	pub const fn new(value: u8) -> Option<Self> {
		if value << 2 >> 2 == value {
			Some(Self(value))
		} else {
			None
		}
	}
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for U6 {
	fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		let byte = u.arbitrary::<u8>()?;
		Ok(Self(byte << 2 >> 2))
	}
}

impl Display for U6 {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl From<U6> for u8 {
	fn from(value: U6) -> Self {
		value.0
	}
}
