use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	io::{Write, stdout},
};

use crate::{
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, DisplayType},
	},
};

pub struct NumberDisplay {
	handler: Box<dyn FnMut(u32, NumberDisplayFormat, bool)>,
	hw_id: u64,
}

impl NumberDisplay {
	pub fn new(handler: impl FnMut(u32, NumberDisplayFormat, bool) + 'static, hw_id: u64) -> Self {
		Self {
			handler: Box::new(handler),
			hw_id,
		}
	}

	#[must_use]
	pub fn print(hw_id: u64) -> Self {
		Self::new(
			|num, format, newline| {
				match format {
					NumberDisplayFormat::Hex => print!("{num:#X}"),
					NumberDisplayFormat::HexLong => print!("{num:#010X}"),
					NumberDisplayFormat::Decimal => print!("{num}"),
					NumberDisplayFormat::DecimalLong => print!("{num:#010}"),
				}

				if newline {
					println!();
				}

				stdout().flush().expect("failed to flush stdout");
			},
			hw_id,
		)
	}
}

impl Bus for NumberDisplay {
	fn name(&self) -> &'static str {
		"Number Display"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			64,
			DeviceCategory::Display(DisplayType::Number),
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
		(self.handler)(
			word,
			match addr % 16 {
				0 => NumberDisplayFormat::Hex,
				4 => NumberDisplayFormat::HexLong,
				8 => NumberDisplayFormat::Decimal,
				12 => NumberDisplayFormat::DecimalLong,
				_ => unreachable!(),
			},
			addr < 16,
		);
	}

	fn reset(&mut self) {}
}

impl Debug for NumberDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("NumberDisplay")
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberDisplayFormat {
	Hex,
	HexLong,
	Decimal,
	DecimalLong,
}
