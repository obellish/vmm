use thiserror::Error;

use super::AuxHwException;
use crate::asm::Reg;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NativeException {
	#[error("unknown opcode {0:#004X}")]
	UnknownOpCode(u8),
	#[error("unknown register code {0:#004X}")]
	UnknownRegister(u8),
	#[error("register {} cannot be read in this mode", Reg::from_code(*.0).unwrap())]
	ReadProtectedRegister(u8),
	#[error("register {} cannot be written in this mode", Reg::from_code(*.0).unwrap())]
	WriteProtectedRegister(u8),
	#[error("unaligned memory address (unalignment is {unalignment}")]
	UnalignedMemoryAddress { unalignment: u8 },
	#[error("address cannot be read in this mode (address' weakest bits are {0:#006X})")]
	MmuRefusedRead(u16),
	#[error("address cannot be written in this mode (address' weakest bits are {0:#006X})")]
	MmuRefusedWrite(u16),
	#[error("address cannot be executed in this mode (address' weakest bits are {0:#006X})")]
	MmuRefusedExec(u16),
	#[error("instruction with opcode {0:#004X} cannot be run in userland mode")]
	SupervisorReservedInstruction(u8),
	#[error("cannot perform a division or modulus by zero")]
	DivisionOrModByZero,
	#[error("cannot perform an overflowing division or modulus")]
	OverflowingDivOrMod,
	#[error("invalid IF/IF2 flag provided: {0:#004X}")]
	InvalidCondFlag(u8),
	#[error("invalid IF2 condition mode provided: {0:#004X}")]
	InvalidCondMode(u8),
	#[error("unknown component ID (weakest bits are {0:#006X})")]
	UnknownComponentId(u16),
	#[error("unknown hardware information code: {0:#004X}")]
	UnknownHardwareInformationCode(u8),
	#[error("component with ID {0:#004X} is not mapped")]
	ComponentNotMapped(u16),
	#[error("hardware exception: {0}")]
	HardwareException(#[from] AuxHwException),
	#[error("interruption (code {0:#004X})")]
	Interruption(u8),
}

impl NativeException {
	#[must_use]
	pub fn decode(ex: u32) -> Option<Self> {
		let (this, _) = Self::decode_with_mode(ex)?;

		Some(this)
	}

	#[must_use]
	pub fn decode_with_mode(ex: u32) -> Option<(Self, bool)> {
		let bytes = ex.to_be_bytes();

		let code = bytes[1];
		let associated = u16::from_be_bytes([bytes[2], bytes[3]]);

		Some((
			Self::decode_parts(code, Some(associated))?,
			!matches!(bytes[0], 0),
		))
	}

	#[must_use]
	pub fn decode_parts(code: u8, associated: Option<u16>) -> Option<Self> {
		Some(match code {
			0x01 => Self::UnknownOpCode(associated? as u8),
			0x02 => Self::UnknownRegister(associated? as u8),
			0x03 => Self::ReadProtectedRegister(associated? as u8),
			0x04 => Self::WriteProtectedRegister(associated? as u8),
			0x05 => Self::UnalignedMemoryAddress {
				unalignment: associated? as u8,
			},
			0x06 => Self::MmuRefusedRead(associated?),
			0x07 => Self::MmuRefusedWrite(associated?),
			0x08 => Self::MmuRefusedExec(associated?),
			0x09 => Self::SupervisorReservedInstruction(associated? as u8),
			0x0A => Self::DivisionOrModByZero,
			0x0B => Self::OverflowingDivOrMod,
			0x0C => Self::InvalidCondFlag(associated? as u8),
			0x0D => Self::InvalidCondMode(associated? as u8),
			0x10 => Self::UnknownComponentId(associated?),
			0x11 => Self::UnknownHardwareInformationCode(associated? as u8),
			0x12 => Self::ComponentNotMapped(associated?),
			0xA0 => Self::HardwareException(AuxHwException::decode(associated?)?),
			0xF0 => Self::Interruption(associated? as u8),
			_ => return None,
		})
	}

	#[must_use]
	pub const fn code(self) -> u8 {
		match self {
			Self::UnknownOpCode(_) => 0x01,
			Self::UnknownRegister(_) => 0x02,
			Self::ReadProtectedRegister(_) => 0x03,
			Self::WriteProtectedRegister(_) => 0x04,
			Self::UnalignedMemoryAddress { unalignment: _ } => 0x05,
			Self::MmuRefusedRead(_) => 0x06,
			Self::MmuRefusedWrite(_) => 0x07,
			Self::MmuRefusedExec(_) => 0x08,
			Self::SupervisorReservedInstruction(_) => 0x09,
			Self::DivisionOrModByZero => 0x0A,
			Self::OverflowingDivOrMod => 0x0B,
			Self::InvalidCondFlag(_) => 0x0C,
			Self::InvalidCondMode(_) => 0x0D,
			Self::UnknownComponentId(_) => 0x10,
			Self::UnknownHardwareInformationCode(_) => 0x11,
			Self::ComponentNotMapped(_) => 0x12,
			Self::HardwareException(_) => 0xA0,
			Self::Interruption(_) => 0xF0,
		}
	}

	#[must_use]
	pub fn associated_data(self) -> Option<u16> {
		match self {
			Self::Interruption(code)
			| Self::UnknownHardwareInformationCode(code)
			| Self::InvalidCondFlag(code)
			| Self::InvalidCondMode(code)
			| Self::WriteProtectedRegister(code)
			| Self::UnknownRegister(code)
			| Self::ReadProtectedRegister(code)
			| Self::UnknownOpCode(code)
			| Self::UnalignedMemoryAddress { unalignment: code }
			| Self::SupervisorReservedInstruction(code) => Some(u16::from(code)),
			Self::MmuRefusedRead(addr)
			| Self::MmuRefusedWrite(addr)
			| Self::MmuRefusedExec(addr)
			| Self::UnknownComponentId(addr)
			| Self::ComponentNotMapped(addr) => Some(addr),
			Self::DivisionOrModByZero | Self::OverflowingDivOrMod => None,
			Self::HardwareException(hw_ex) => Some(hw_ex.encode()),
		}
	}

	#[must_use]
	pub fn encode(self) -> u32 {
		(u32::from(self.code()) << 16) + u32::from(self.associated_data().unwrap_or(0))
	}

	#[must_use]
	pub fn encode_with_mode(self, was_sv: bool) -> u32 {
		self.encode() + if was_sv { 1 << 24 } else { 0 }
	}
}
