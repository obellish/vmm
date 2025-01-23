use std::fmt::{Debug, Formatter, Result as FmtResult};

use vmm_tools::{
	exceptions::AuxHwException,
	metadata::{DebugType, DeviceCategory, DeviceMetadata},
};

use crate::vmm::board::Bus;

pub struct BasicDebug {
	hw_id: u64,
	debugger: Box<dyn FnMut(DebugInfo)>,
}

impl BasicDebug {
	pub fn new(debugger: impl FnMut(DebugInfo) + 'static, hw_id: u64) -> Self {
		Self {
			hw_id,
			debugger: Box::new(debugger),
		}
	}

	#[must_use]
	pub fn println(hw_id: u64) -> Self {
		Self::new(
			|info| {
				println!("[debug:basic] {}", match info {
					DebugInfo::UnsignedByteHex(n) => format!("{n:#004X}"),
					DebugInfo::UnsignedHalfWordHex(n) => format!("{n:#006X}"),
					DebugInfo::UnsignedWordHex(n) => format!("{n:#010X}"),
					DebugInfo::SignedByteHex(n) if n < 0 => format!("-{:#004X}", -n),
					DebugInfo::SignedByteHex(n) => format!("{n:#004X}"),
					DebugInfo::SignedHalfWordHex(n) if n < 0 => format!("-{:#006X}", -n),
					DebugInfo::SignedHalfWordHex(n) => format!("{n:#006X}"),
					DebugInfo::SignedWordHex(n) if n < 0 => format!("-{:#010X}", -n),
					DebugInfo::SignedWordHex(n) => format!("{n:#010X}"),
					DebugInfo::UnsignedByteDec(n) => n.to_string(),
					DebugInfo::UnsignedHalfWordDec(n) => n.to_string(),
					DebugInfo::UnsignedWordDec(n) => n.to_string(),
					DebugInfo::SignedByteDec(n) => n.to_string(),
					DebugInfo::SignedHalfWordDec(n) => n.to_string(),
					DebugInfo::SignedWordDec(n) => n.to_string(),
					DebugInfo::Boolean(b) => b.to_string(),
					DebugInfo::Utf8Char(Ok(c)) => c.to_string(),
					DebugInfo::Utf8Char(Err(code)) =>
						format!("<invalid UTF-8 character: {code:#010X}"),
					DebugInfo::EncodingAgnosticChar(c) =>
						format!("<encoding-agnostic character: {c:#010X}>"),
					DebugInfo::DebugMessage => "debug point".to_owned(),
				});
			},
			hw_id,
		)
	}
}

impl Bus for BasicDebug {
	fn name(&self) -> &'static str {
		"Basic Debug Interface"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			32,
			DeviceCategory::Debug(DebugType::Basic),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, _: u32, ex: &mut u16) -> u32 {
		*ex = AuxHwException::MemoryNotReadable.encode();
		0
	}

	fn write(&mut self, addr: u32, word: u32, _: &mut u16) {
		let info = match addr / 4 {
			0 => DebugInfo::UnsignedByteHex(word as u8),
			1 => DebugInfo::UnsignedHalfWordHex(word as u16),
			2 => DebugInfo::UnsignedWordHex(word),
			3 => DebugInfo::SignedByteHex(word as i8),
			4 => DebugInfo::SignedHalfWordHex(word as i16),
			5 => DebugInfo::SignedWordHex(word as i32),
			6 => DebugInfo::UnsignedByteDec(word as u8),
			7 => DebugInfo::UnsignedHalfWordDec(word as u16),
			8 => DebugInfo::UnsignedWordDec(word),
			9 => DebugInfo::SignedByteDec(word as i8),
			10 => DebugInfo::SignedHalfWordDec(word as i16),
			11 => DebugInfo::SignedWordDec(word as i32),
			12 => DebugInfo::Boolean(!matches!(word, 0)),
			13 => DebugInfo::Utf8Char(std::char::from_u32(word).ok_or(word)),
			14 => DebugInfo::EncodingAgnosticChar(word),
			15 => DebugInfo::DebugMessage,
			_ => unreachable!("tried to write to out-of-range address in basic debug interface"),
		};

		(self.debugger)(info);
	}

	fn reset(&mut self) {}
}

impl Debug for BasicDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("BasicDebug")
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum DebugInfo {
	UnsignedByteHex(u8),
	UnsignedHalfWordHex(u16),
	UnsignedWordHex(u32),
	SignedByteHex(i8),
	SignedHalfWordHex(i16),
	SignedWordHex(i32),
	UnsignedByteDec(u8),
	UnsignedHalfWordDec(u16),
	UnsignedWordDec(u32),
	SignedByteDec(i8),
	SignedHalfWordDec(i16),
	SignedWordDec(i32),
	Boolean(bool),
	Utf8Char(Result<char, u32>),
	EncodingAgnosticChar(u32),
	DebugMessage,
}
