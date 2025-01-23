use std::borrow::Cow;

use crate::asm::{Instr, ToLasm};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramWord {
	Instr(Instr),
	Raw([u8; 4]),
}

impl ProgramWord {
	#[must_use]
	pub fn decode(bytes: [u8; 4]) -> Self {
		Instr::decode(bytes).map_or(Self::Raw(bytes), Self::Instr)
	}

	#[must_use]
	pub fn decode_word(word: u32) -> Self {
		Self::decode(word.to_be_bytes())
	}

	#[must_use]
	pub const fn is_instr(self) -> bool {
		matches!(self, Self::Instr(_))
	}

	#[must_use]
	pub const fn is_raw(self) -> bool {
		matches!(self, Self::Raw(_))
	}

	#[must_use]
	pub fn encode(self) -> [u8; 4] {
		match self {
			Self::Instr(instr) => instr.encode(),
			Self::Raw(bytes) => bytes,
		}
	}

	#[must_use]
	pub fn encode_word(self) -> u32 {
		match self {
			Self::Instr(instr) => instr.encode_word(),
			Self::Raw(bytes) => u32::from_be_bytes(bytes),
		}
	}
}

impl From<Instr> for ProgramWord {
	fn from(value: Instr) -> Self {
		Self::Instr(value)
	}
}

impl ToLasm for ProgramWord {
	fn to_lasm(&self) -> Cow<'static, str> {
		match self {
			Self::Instr(instr) => instr.to_lasm(),
			Self::Raw(bytes) => Cow::Owned(format!(
				"#d32 0x{:002X}_{:002X}_{:002X}_{:002X}",
				bytes[0], bytes[1], bytes[2], bytes[3]
			)),
		}
	}
}
