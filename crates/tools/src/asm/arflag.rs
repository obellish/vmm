use super::{RegOrLit1, RegOrLit2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArFlag {
	Zero,
	Carry,
	Overflow,
	Sign,
	Even,
	ZeroUpper,
	ZeroLower,
}

impl ArFlag {
	#[must_use]
	pub const fn decode(code: u8) -> Option<Self> {
		match code {
			0x00 => Some(Self::Zero),
			0x01 => Some(Self::Carry),
			0x02 => Some(Self::Overflow),
			0x03 => Some(Self::Sign),
			0x04 => Some(Self::Even),
			0x05 => Some(Self::ZeroUpper),
			0x06 => Some(Self::ZeroLower),
			_ => None,
		}
	}

	#[must_use]
	pub const fn code(self) -> u8 {
		self as u8
	}

	#[must_use]
	pub const fn name(self) -> &'static str {
		match self {
			Self::Zero => "Zero",
			Self::Carry => "Carry",
			Self::Overflow => "Overflow",
			Self::Sign => "Sign",
			Self::Even => "Even",
			Self::ZeroUpper => "ZeroUpper",
			Self::ZeroLower => "ZeroLower",
		}
	}

	#[must_use]
	pub const fn short_name(self) -> &'static str {
		match self {
			Self::Zero => "ZF",
			Self::Carry => "CF",
			Self::Overflow => "OF",
			Self::Sign => "SF",
			Self::Even => "EF",
			Self::ZeroUpper => "ZUF",
			Self::ZeroLower => "ZLF",
		}
	}

	#[must_use]
	pub const fn to_vasm(self) -> &'static str {
		self.short_name()
	}
}

impl From<ArFlag> for u8 {
	fn from(value: ArFlag) -> Self {
		value.code()
	}
}

impl From<ArFlag> for RegOrLit1 {
	fn from(value: ArFlag) -> Self {
		value.code().into()
	}
}

impl From<ArFlag> for RegOrLit2 {
	fn from(value: ArFlag) -> Self {
		value.code().into()
	}
}
