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

pub struct CharDisplay {
	handler: Box<dyn FnMut(Result<char, u32>)>,
	hw_id: u64,
}

impl CharDisplay {
	pub fn new(handler: impl FnMut(Result<char, u32>) + 'static, hw_id: u64) -> Self {
		Self {
			handler: Box::new(handler),
			hw_id,
		}
	}

	#[must_use]
	pub fn print_lossy(hw_id: u64) -> Self {
		Self::new(
			|result| {
				print!("{}", result.unwrap_or('�'));

				stdout().flush().expect("failed to flush stdout");
			},
			hw_id,
		)
	}
}

impl Bus for CharDisplay {
	fn name(&self) -> &'static str {
		"Character Display"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			4,
			DeviceCategory::Display(DisplayType::Character),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, _: u32, ex: &mut u16) -> u32 {
		*ex = AuxHwException::MemoryNotReadable.encode();
		0
	}

	fn write(&mut self, _: u32, word: u32, _: &mut u16) {
		(self.handler)(std::char::from_u32(word).ok_or(word));
	}

	fn reset(&mut self) {}
}

impl Debug for CharDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("CharDisplay")
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}
