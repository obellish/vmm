use std::{
	borrow::Cow,
	fmt::{Display, Formatter, Result as FmtResult},
};

use super::ToLasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reg {
	A0,
	A1,
	A2,
	A3,
	A4,
	A5,
	A6,
	A7,
	C0,
	C1,
	Ac0,
	Ac1,
	Ac2,
	Rr0,
	Rr1,
	Rr2,
	Rr3,
	Rr4,
	Rr5,
	Rr6,
	Rr7,
	Avr,
	Af,
	Pc,
	Ssp,
	Usp,
	Et,
	Era,
	Ev,
	Mtt,
	Pda,
	Smt,
}

impl Reg {
	#[must_use]
	pub const fn from_code(code: u8) -> Option<Self> {
		match code {
			0x00 => Some(Self::A0),
			0x01 => Some(Self::A1),
			0x02 => Some(Self::A2),
			0x03 => Some(Self::A3),
			0x04 => Some(Self::A4),
			0x05 => Some(Self::A5),
			0x06 => Some(Self::A6),
			0x07 => Some(Self::A7),
			0x08 => Some(Self::C0),
			0x09 => Some(Self::C1),
			0x0A => Some(Self::Ac0),
			0x0B => Some(Self::Ac1),
			0x0C => Some(Self::Ac2),
			0x0D => Some(Self::Rr0),
			0x0E => Some(Self::Rr1),
			0x0F => Some(Self::Rr2),
			0x10 => Some(Self::Rr3),
			0x11 => Some(Self::Rr4),
			0x12 => Some(Self::Rr5),
			0x13 => Some(Self::Rr6),
			0x14 => Some(Self::Rr7),
			0x15 => Some(Self::Avr),
			0x16 => Some(Self::Pc),
			0x17 => Some(Self::Af),
			0x18 => Some(Self::Ssp),
			0x19 => Some(Self::Usp),
			0x1A => Some(Self::Et),
			0x1B => Some(Self::Era),
			0x1C => Some(Self::Ev),
			0x1D => Some(Self::Mtt),
			0x1E => Some(Self::Pda),
			0x1F => Some(Self::Smt),
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
			Self::A0 => "a0",
			Self::A1 => "a1",
			Self::A2 => "a2",
			Self::A3 => "a3",
			Self::A4 => "a4",
			Self::A5 => "a5",
			Self::A6 => "a6",
			Self::A7 => "a7",
			Self::C0 => "c0",
			Self::C1 => "c1",
			Self::Ac0 => "ac0",
			Self::Ac1 => "ac1",
			Self::Ac2 => "ac2",
			Self::Rr0 => "rr0",
			Self::Rr1 => "rr1",
			Self::Rr2 => "rr2",
			Self::Rr3 => "rr3",
			Self::Rr4 => "rr4",
			Self::Rr5 => "rr5",
			Self::Rr6 => "rr6",
			Self::Rr7 => "rr7",
			Self::Avr => "avr",
			Self::Pc => "pc",
			Self::Af => "af",
			Self::Ssp => "ssp",
			Self::Usp => "usp",
			Self::Et => "et",
			Self::Era => "era",
			Self::Ev => "ev",
			Self::Mtt => "mtt",
			Self::Pda => "pda",
			Self::Smt => "smt",
		}
	}
}

impl Display for Reg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self.name())
	}
}

impl From<Reg> for u8 {
	fn from(value: Reg) -> Self {
		value.code()
	}
}

impl ToLasm for Reg {
	fn to_lasm(&self) -> Cow<'static, str> {
		Cow::Borrowed(self.name())
	}
}
