use std::borrow::Cow;

use super::ToVasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HwInfo {
	Count,
	UidUpper,
	UidLower,
	NameLength = 0x10,
	NameW1,
	NameW2,
	NameW3,
	NameW4,
	NameW5,
	NameW6,
	NameW7,
	NameW8,
	DevSize = 0x20,
	Category,
	Type,
	Model,
	DataUpper,
	DataLower,
	IsMapped = 0xA0,
	MapStart,
	MapEnd,
}

impl HwInfo {
	#[must_use]
	pub const fn decode(code: u8) -> Option<Self> {
		Some(match code {
			0x00 => Self::Count,
			0x01 => Self::UidUpper,
			0x02 => Self::UidLower,
			0x10 => Self::NameLength,
			0x11 => Self::NameW1,
			0x12 => Self::NameW2,
			0x13 => Self::NameW3,
			0x14 => Self::NameW4,
			0x15 => Self::NameW5,
			0x16 => Self::NameW6,
			0x17 => Self::NameW7,
			0x18 => Self::NameW8,
			0x20 => Self::DevSize,
			0x21 => Self::Category,
			0x22 => Self::Type,
			0x23 => Self::Model,
			0x24 => Self::DataUpper,
			0x25 => Self::DataLower,
			0xA0 => Self::IsMapped,
			0xA1 => Self::MapStart,
			0xA2 => Self::MapEnd,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn code(self) -> u8 {
		self as u8
	}
}

impl From<HwInfo> for u8 {
	fn from(value: HwInfo) -> Self {
		value.code()
	}
}

impl ToVasm for HwInfo {
	fn to_vasm(&self) -> Cow<'static, str> {
		Cow::Borrowed(match self {
			Self::Count => "HWD_COUNT",
			Self::UidUpper => "HWD_UID_UPPER",
			Self::UidLower => "HWD_UID_LOWER",
			Self::NameLength => "HWD_NAME_LEN",
			Self::NameW1 => "HWD_NAME_W1",
			Self::NameW2 => "HWD_NAME_W2",
			Self::NameW3 => "HWD_NAME_W3",
			Self::NameW4 => "HWD_NAME_W4",
			Self::NameW5 => "HWD_NAME_W5",
			Self::NameW6 => "HWD_NAME_W6",
			Self::NameW7 => "HWD_NAME_W7",
			Self::NameW8 => "HWD_NAME_W8",
			Self::DevSize => "HWD_SIZE",
			Self::Category => "HWD_CAT",
			Self::Type => "HWD_TYPE",
			Self::Model => "HWD_MODEL",
			Self::DataUpper => "HWD_DATA_UPPER",
			Self::DataLower => "HWD_DATA_LOWER",
			Self::IsMapped => "HWD_IS_MAPPED",
			Self::MapStart => "HWD_MAP_START",
			Self::MapEnd => "HWD_MAP_END",
		})
	}
}
