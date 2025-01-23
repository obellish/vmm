use std::fmt::{Display, Formatter, Result as FmtResult};

use super::types::{ClockType, DebugType, DisplayType, KeyboardType, MemoryType, StorageType};

#[derive(Debug, Clone, Copy)]
pub enum DeviceCategory {
	Debug(DebugType),
	Clock(ClockType),
	Display(DisplayType),
	Keyboard(KeyboardType),
	Memory(MemoryType),
	Storage(StorageType),
	PlatformSpecific(u32),
	Uncategorized,
}

impl DeviceCategory {
	#[must_use]
	pub fn decode(code: u64) -> Option<Self> {
		let cat = (code >> 32) as u32;
		let typ = (code & 0xFFFF_FFFF) as u32;

		Some(match cat {
			0x0000_0100 => Self::Debug(DebugType::decode(typ)?),
			0x0000_1000 => Self::Clock(ClockType::decode(typ)?),
			0x0001_1000 => Self::Display(DisplayType::decode(typ)?),
			0x0001_6000 => Self::Keyboard(KeyboardType::decode(typ)?),
			0x0002_1000 => Self::Memory(MemoryType::decode(typ)?),
			0x0002_2000 => Self::Storage(StorageType::decode(typ)?),
			0xEEEE_EEEE => Self::PlatformSpecific(typ),
			0xFFFF_FFFF => Self::Uncategorized,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn category_code(self) -> u32 {
		match self {
			Self::Debug(_) => 0x0000_0100,
			Self::Clock(_) => 0x0000_1000,
			Self::Display(_) => 0x0001_1000,
			Self::Keyboard(_) => 0x0001_6000,
			Self::Memory(_) => 0x0002_1000,
			Self::Storage(_) => 0x0002_2000,
			Self::PlatformSpecific(_) => 0xEEEE_EEEE,
			Self::Uncategorized => 0xFFFF_FFFF,
		}
	}

	#[must_use]
	pub const fn type_code(self) -> u32 {
		match self {
			Self::Debug(t) => t.code(),
			Self::Clock(t) => t.code(),
			Self::Display(t) => t.code(),
			Self::Keyboard(t) => t.code(),
			Self::Memory(t) => t.code(),
			Self::Storage(t) => t.code(),
			Self::PlatformSpecific(typ) => typ,
			Self::Uncategorized => 0x0000_0000,
		}
	}

	#[must_use]
	pub const fn encode(self) -> u64 {
		((self.category_code() as u64) << 32) + self.type_code() as u64
	}
}

impl Display for DeviceCategory {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Debug(d) => {
				f.write_str("Debug:")?;
				Display::fmt(d, f)
			}
			Self::Clock(c) => {
				f.write_str("Clock:")?;
				Display::fmt(c, f)
			}
			Self::Display(d) => {
				f.write_str("Display:")?;
				Display::fmt(d, f)
			}
			Self::Keyboard(k) => {
				f.write_str("Keyboard:")?;
				Display::fmt(k, f)
			}
			Self::Memory(m) => {
				f.write_str("Memory:")?;
				Display::fmt(m, f)
			}
			Self::Storage(s) => {
				f.write_str("Storage:")?;
				Display::fmt(s, f)
			}
			Self::PlatformSpecific(code) => write!(f, "PlatformSpecific:(Code={code:#010X})"),
			Self::Uncategorized => f.write_str("Uncategorized"),
		}
	}
}
