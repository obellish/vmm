use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, KeyboardType},
	},
};

pub struct SyncLineKeyboard {
	buffer: Vec<u32>,
	capacity: u32,
	handler: Box<dyn FnMut() -> String>,
	hw_id: u64,
}

impl SyncLineKeyboard {
	pub fn new(
		capacity: u32,
		handler: impl FnMut() -> String + 'static,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let _: usize = capacity.try_into()?;

		if matches!(capacity, 0) {
			return Err(ComponentCreationError::CapacityCannotBeZero);
		}

		if !matches!(capacity % 4, 0) {
			return Err(ComponentCreationError::CapacityMustBeAligned);
		}

		let capacity = capacity / 4;

		Ok(Self {
			buffer: vec![0; (capacity - 1) as usize],
			capacity: capacity - 1,
			handler: Box::new(handler),
			hw_id,
		})
	}
}

impl Bus for SyncLineKeyboard {
	fn name(&self) -> &'static str {
		"Synchronous Line Keyboard"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.capacity * 4 + 4,
			DeviceCategory::Keyboard(KeyboardType::ReadLineSynchronous),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, _: &mut u16) -> u32 {
		let addr = addr / 4;
		if addr == self.capacity {
			0
		} else {
			self.buffer[addr as usize]
		}
	}

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		if addr / 4 == self.capacity {
			match word {
				0xAA => {
					let mut word = 0;
					let mut byte_index = 0;
					let mut pos = 0;

					for byte in (self.handler)().bytes() {
						word += u32::from(byte) << ((3 - byte_index) * 8);

						if matches!(byte_index, 3) {
							if pos >= self.buffer.len() {
								eprintln!(
									"warning: input is too long for synchronous keyboard's buffer (max. {} bytes)",
									self.capacity * 4
								);
								return;
							}

							self.buffer[pos] = word;
							pos += 1;
							byte_index = 0;
							word = 0;
						} else {
							byte_index += 1;
						}
					}

					if byte_index > 0 {
						if pos >= self.buffer.len() {
							eprintln!(
								"warning: input is too long for synchronous keyboard's buffer (max. {} bytes)",
								self.capacity * 4
							);
							return;
						}

						self.buffer[pos] = word;
					}
				}
				0xFF => self.reset(),
				code => *ex = AuxHwException::UnknownOperation(code as u8).encode(),
			}
		} else {
			*ex = 31 << 8;
		}
	}

	fn reset(&mut self) {
		self.buffer = vec![0; self.buffer.len()];
	}
}

impl Debug for SyncLineKeyboard {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("SyncLineKeyboard")
			.field("buffer", &self.buffer)
			.field("capacity", &self.capacity)
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}
