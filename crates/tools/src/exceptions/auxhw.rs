use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum AuxHwException {
	#[error("unknown error")]
	UnknownError,
	#[error("unspecified synchronization error")]
	UnspecifiedSyncError,
	#[error("time synchronization error")]
	TimeSynchronizationError,
	#[error("unknown operation: {0:#004X}")]
	UnknownOperation(u8),
	#[error("unsupported operation")]
	UnsupportedOperation,
	#[error("generic physical read error")]
	GenericPhysicalReadError,
	#[error("memory address is not readable")]
	MemoryNotReadable,
	#[error("generic physical write error")]
	GenericPhysicalWriteError,
	#[error("memory address is not writable")]
	MemoryNotWritable,
}

impl AuxHwException {
	#[must_use]
	pub fn decode(ex: u16) -> Option<Self> {
		let code = (ex >> 8) as u8;
		let data = ex as u8;

		Self::decode_parts(code, Some(data))
	}

	#[must_use]
	pub fn decode_parts(code: u8, data: Option<u8>) -> Option<Self> {
		Some(match code {
			0x00 => Self::UnknownError,
			0x01 => Self::UnspecifiedSyncError,
			0x02 => Self::TimeSynchronizationError,
			0x10 => Self::UnknownOperation(data?),
			0x11 => Self::UnsupportedOperation,
			0x20 => Self::GenericPhysicalReadError,
			0x21 => Self::MemoryNotReadable,
			0x30 => Self::GenericPhysicalWriteError,
			0x31 => Self::MemoryNotWritable,
			_ => return None,
		})
	}

	#[must_use]
	pub const fn code(self) -> u8 {
		match self {
			Self::UnknownError => 0x00,
			Self::UnspecifiedSyncError => 0x01,
			Self::TimeSynchronizationError => 0x02,
			Self::UnknownOperation(_) => 0x10,
			Self::UnsupportedOperation => 0x11,
			Self::GenericPhysicalReadError => 0x20,
			Self::MemoryNotReadable => 0x21,
			Self::GenericPhysicalWriteError => 0x30,
			Self::MemoryNotWritable => 0x31,
		}
	}

	#[must_use]
	pub const fn associated_data(self) -> Option<u8> {
		if let Self::UnknownOperation(op) = self {
			Some(op)
		} else {
			None
		}
	}

	#[must_use]
	pub fn encode(self) -> u16 {
		(u16::from(self.code()) << 8) + u16::from(self.associated_data().unwrap_or(0))
	}
}

impl From<AuxHwException> for u16 {
	fn from(value: AuxHwException) -> Self {
		value.encode()
	}
}
