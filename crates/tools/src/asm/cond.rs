use std::borrow::Cow;

use super::ToVasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum If2Cond {
	Or = 1,
	And,
	Xor,
	Nor,
	Nand,
	Left,
	Right,
}

impl If2Cond {
	#[must_use]
	pub const fn decode(code: u8) -> Option<Self> {
		match code {
			0x01 => Some(Self::Or),
			0x02 => Some(Self::And),
			0x03 => Some(Self::Xor),
			0x04 => Some(Self::Nor),
			0x05 => Some(Self::Nand),
			0x06 => Some(Self::Left),
			0x07 => Some(Self::Right),
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
			Self::Or => "OR",
			Self::And => "AND",
			Self::Xor => "XOR",
			Self::Nor => "NOR",
			Self::Nand => "NAND",
			Self::Left => "LEFT",
			Self::Right => "RIGHT",
		}
	}
}

impl ToVasm for If2Cond {
	fn to_vasm(&self) -> Cow<'static, str> {
		let mut output = String::from("CMP_");

		output.push_str(self.name());

		Cow::Owned(output)
	}
}
