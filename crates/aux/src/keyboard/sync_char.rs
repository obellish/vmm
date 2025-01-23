use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, KeyboardType},
	},
};

pub struct SyncCharKeyboard {
	buffer: char,
	handler: Box<dyn FnMut() -> char>,
	hw_id: u64,
}

impl SyncCharKeyboard {
	pub fn new(handler: impl FnMut() -> char + 'static, hw_id: u64) -> Self {
		Self {
			buffer: 0 as char,
			handler: Box::new(handler),
			hw_id,
		}
	}
}

impl Bus for SyncCharKeyboard {
	fn name(&self) -> &'static str {
		"Synchronous Character Keyboard"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			8,
			DeviceCategory::Keyboard(KeyboardType::ReadCharSynchronous),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, ex: &mut u16) -> u32 {
		if matches!(addr, 0) {
			self.buffer as u32
		} else if matches!(addr, 4) {
			*ex = AuxHwException::MemoryNotReadable.encode();
			0
		} else {
			unreachable!()
		}
	}

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		if matches!(addr, 0) {
			*ex = 0x31 << 8;
		} else if matches!(addr, 4) {
			match word {
				0x01 => self.buffer = (self.handler)(),
				0x02 => self.reset(),
				code => *ex = AuxHwException::UnknownOperation(code as u8).encode(),
			}
		} else {
			unreachable!()
		}
	}

	fn reset(&mut self) {
		self.buffer = 0 as char;
	}
}

impl Debug for SyncCharKeyboard {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("SyncCharKeyboard")
			.field("buffer", &self.buffer)
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}
