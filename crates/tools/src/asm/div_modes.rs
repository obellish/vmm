use std::{borrow::Cow, fmt::Debug};

use super::{RegOrLit1, ToVasm, cst};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DivMode(pub DivSignMode, pub DivByZeroMode, pub DivOverflowMode);

impl DivMode {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	#[must_use]
	pub fn decode(mode: u8) -> Option<Self> {
		Some(Self(
			DivSignMode::from_mode(mode)?,
			DivByZeroMode::from_mode(mode)?,
			DivOverflowMode::from_mode(mode)?,
		))
	}

	#[must_use]
	pub const fn from_sub_modes(
		sign_mode: DivSignMode,
		zero_mode: DivByZeroMode,
		ovf_mode: DivOverflowMode,
	) -> Self {
		Self(sign_mode, zero_mode, ovf_mode)
	}

	#[must_use]
	pub const fn sign_mode(self) -> DivSignMode {
		self.0
	}

	#[must_use]
	pub const fn zro_mode(self) -> DivByZeroMode {
		self.1
	}

	#[must_use]
	pub const fn ovf_mode(self) -> DivOverflowMode {
		self.2
	}

	#[must_use]
	pub const fn with_sign_mode(mut self, mode: DivSignMode) -> Self {
		self.0 = mode;
		self
	}

	#[must_use]
	pub const fn with_zro_mode(mut self, mode: DivByZeroMode) -> Self {
		self.1 = mode;
		self
	}

	#[must_use]
	pub const fn with_ovf_mode(mut self, mode: DivOverflowMode) -> Self {
		self.2 = mode;
		self
	}

	#[must_use]
	pub fn encode(self) -> u8 {
		self.0.encode() | self.1.encode() | self.2.encode()
	}

	#[must_use]
	pub fn to_val(self) -> RegOrLit1 {
		self.encode().into()
	}
}

impl From<DivMode> for u8 {
	fn from(value: DivMode) -> Self {
		value.encode()
	}
}

impl ToVasm for DivMode {
	fn to_vasm(&self) -> Cow<'static, str> {
		let mut modes = Vec::new();

		if self.0 != DivSignMode::default() {
			modes.push(self.0.to_vasm());
		}

		if self.1 != DivByZeroMode::default() {
			modes.push(self.1.to_vasm());
		}

		if self.2 != DivOverflowMode::default() {
			modes.push(self.2.to_vasm());
		}

		if modes.is_empty() {
			Cow::Borrowed("0")
		} else {
			Cow::Owned(modes.join(" | "))
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivSignMode {
	#[default]
	Unsigned,
	Signed,
}

impl DivSignMode {
	#[must_use]
	pub const fn from_is_signed(signed: bool) -> Self {
		if signed { Self::Signed } else { Self::Unsigned }
	}

	#[must_use]
	pub const fn is_signed(self) -> bool {
		matches!(self, Self::Signed)
	}
}

impl DivSubMode for DivSignMode {
	const MASK: u8 = cst::DIV_SIGN_MODE_MASK;

	fn decode(sub_mode: u8) -> Option<Self> {
		match sub_mode {
			cst::DIV_USG => Some(Self::Unsigned),
			cst::DIV_SIG => Some(Self::Signed),
			_ => None,
		}
	}

	fn encode(self) -> u8 {
		match self {
			Self::Signed => cst::DIV_SIG,
			Self::Unsigned => cst::DIV_USG,
		}
	}
}

impl From<DivSignMode> for u8 {
	fn from(value: DivSignMode) -> Self {
		value.encode()
	}
}

impl ToVasm for DivSignMode {
	fn to_vasm(&self) -> Cow<'static, str> {
		Cow::Borrowed(match self {
			Self::Signed => "DIV_SIG",
			Self::Unsigned => "DIV_USG",
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivByZeroMode {
	#[default]
	Forbid,
	EqToMin,
	EqToZero,
	EqToMax,
}

impl DivByZeroMode {
	#[must_use]
	pub const fn result(self) -> u32 {
		match self {
			Self::Forbid | Self::EqToZero => 0,
			Self::EqToMin => i32::MIN as u32,
			Self::EqToMax => i32::MAX as u32,
		}
	}
}

impl DivSubMode for DivByZeroMode {
	const MASK: u8 = cst::DIV_ZERO_MODE_MASK;

	fn decode(sub_mode: u8) -> Option<Self> {
		match sub_mode {
			cst::DIV_ZRO_FRB => Some(Self::Forbid),
			cst::DIV_ZRO_MIN => Some(Self::EqToMin),
			cst::DIV_ZRO_ZRO => Some(Self::EqToZero),
			cst::DIV_ZRO_MAX => Some(Self::EqToMax),
			_ => None,
		}
	}

	fn encode(self) -> u8 {
		match self {
			Self::Forbid => cst::DIV_ZRO_FRB,
			Self::EqToMin => cst::DIV_ZRO_MIN,
			Self::EqToZero => cst::DIV_ZRO_ZRO,
			Self::EqToMax => cst::DIV_ZRO_MAX,
		}
	}
}

impl From<DivByZeroMode> for u8 {
	fn from(value: DivByZeroMode) -> Self {
		value.encode()
	}
}

impl ToVasm for DivByZeroMode {
	fn to_vasm(&self) -> Cow<'static, str> {
		Cow::Borrowed(match self {
			Self::Forbid => "DIV_ZRO_FRB",
			Self::EqToMin => "DIV_ZRO_MIN",
			Self::EqToZero => "DIV_ZRO_ZRO",
			Self::EqToMax => "DIV_ZRO_MAX",
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivOverflowMode {
	#[default]
	Forbid,
	EqToMin,
	EqToZero,
	EqToMax,
}

impl DivSubMode for DivOverflowMode {
	const MASK: u8 = cst::DIV_OVFW_MODE_MASK;

	fn decode(sub_mode: u8) -> Option<Self> {
		match sub_mode {
			cst::DIV_OFW_FRB => Some(Self::Forbid),
			cst::DIV_OFW_MIN => Some(Self::EqToMin),
			cst::DIV_OFW_ZRO => Some(Self::EqToZero),
			cst::DIV_OFW_MAX => Some(Self::EqToMax),
			_ => None,
		}
	}

	fn encode(self) -> u8 {
		match self {
			Self::Forbid => cst::DIV_OFW_FRB,
			Self::EqToMin => cst::DIV_OFW_MIN,
			Self::EqToZero => cst::DIV_OFW_ZRO,
			Self::EqToMax => cst::DIV_OFW_MAX,
		}
	}
}

impl From<DivOverflowMode> for u8 {
	fn from(value: DivOverflowMode) -> Self {
		value.encode()
	}
}

impl ToVasm for DivOverflowMode {
	fn to_vasm(&self) -> Cow<'static, str> {
		Cow::Borrowed(match self {
			Self::Forbid => "DIV_OFW_FRB",
			Self::EqToMin => "DIV_OFW_MIN",
			Self::EqToZero => "DIV_OFW_ZRO",
			Self::EqToMax => "DIV_OFW_MAX",
		})
	}
}

pub trait DivSubMode: Clone + Copy + Debug + Eq + Sized + ToVasm {
	const MASK: u8;

	#[must_use]
	fn from_mode(mode: u8) -> Option<Self> {
		Self::decode(mode & Self::MASK)
	}

	fn decode(sub_mode: u8) -> Option<Self>;

	fn encode(self) -> u8;
}
