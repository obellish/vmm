use std::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	str::{Utf8Error, from_utf8},
};

use crate::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::{
		bytes::words_to_bytes,
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, DisplayType},
	},
};

pub struct BufferedDisplay {
	buffer: Vec<u32>,
	words: u32,
	handler: Box<dyn FnMut(DecodedStr<'_>)>,
	hw_id: u64,
}

impl BufferedDisplay {
	pub fn new(
		capacity: u32,
		handler: impl FnMut(DecodedStr<'_>) + 'static,
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
			words: capacity - 1,
			handler: Box::new(handler),
			hw_id,
		})
	}

	pub fn print_lossy(capacity: u32, hw_id: u64) -> Result<Self, ComponentCreationError> {
		Self::new(
			capacity,
			|message| match message {
				Ok(message) => print!("{message}"),
				Err(e) => print!("{}", String::from_utf8_lossy(e.bytes())),
			},
			hw_id,
		)
	}
}

impl Bus for BufferedDisplay {
	fn name(&self) -> &'static str {
		"Buffered Display"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.words * 4 + 4,
			DeviceCategory::Display(DisplayType::Buffered),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, _: u32, ex: &mut u16) -> u32 {
		*ex = AuxHwException::MemoryNotReadable.encode();
		0
	}

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		let addr = addr / 4;

		if addr < self.words {
			self.buffer[addr as usize] = word;
			return;
		}

		match word {
			0xAA => {
				let bytes = words_to_bytes(&self.buffer);
				(self.handler)(
					from_utf8(bytes.as_slice()).map_err(|source| BufferedUtf8Error {
						source,
						bytes: &bytes,
					}),
				);
			}
			0xBB => {
				let bytes = words_to_bytes(&self.buffer);
				(self.handler)(Ok(&String::from_utf8_lossy(&bytes)));
			}
			0xFF => self.reset(),
			code => *ex = AuxHwException::UnknownOperation(code as u8).encode(),
		}
	}

	fn reset(&mut self) {
		self.buffer = vec![0; self.buffer.len()];
	}
}

impl Debug for BufferedDisplay {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("BufferedDisplay")
			.field("buffer", &self.buffer)
			.field("words", &self.words)
			.field("hw_id", &self.hw_id)
			.finish_non_exhaustive()
	}
}

#[derive(Debug)]
pub struct BufferedUtf8Error<'a> {
	source: Utf8Error,
	bytes: &'a [u8],
}

impl<'a> BufferedUtf8Error<'a> {
	#[must_use]
	pub const fn source(&self) -> &Utf8Error {
		&self.source
	}

	#[must_use]
	pub const fn bytes(&self) -> &'a [u8] {
		self.bytes
	}
}

impl Display for BufferedUtf8Error<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.source, f)
	}
}

impl StdError for BufferedUtf8Error<'_> {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		Some(&self.source)
	}
}

pub type DecodedStr<'a> = Result<&'a str, BufferedUtf8Error<'a>>;

#[cfg(test)]
mod tests {
	use std::sync::{Arc, Mutex};

	use crate::{
		display::BufferedDisplay,
		error::ComponentCreationError,
		storage::BootRom,
		vmm_tools::{
			asm::{ExtInstr, Instr, Program, Reg},
			debug::{RunConfig, exec_vm},
		},
	};

	fn display_prog(text: &str, display_addr: u32, display_final_addr: u32) -> Option<Program> {
		let mut instr = ExtInstr::SetReg(Reg::Ac0, display_addr).to_instr();
		instr.push(Instr::Cpy(Reg::Avr, 0u8.into()));

		let mut byte_index = 0;

		let text_bytes = text.bytes();

		if text_bytes.len() as u64 > u64::from(display_final_addr - display_addr) {
			return None;
		}

		for byte in text_bytes {
			instr.push(Instr::Add(Reg::Avr, byte.into()));
			byte_index += 1;

			if byte_index < 4 {
				instr.push(Instr::Shl(Reg::Avr, 8u8.into()));
			} else {
				instr.extend([
					Instr::Wea(Reg::Ac0.into(), 0u8.into(), 0u8.into()),
					Instr::Add(Reg::Ac0, 4u8.into()),
					Instr::Cpy(Reg::Avr, 0u8.into()),
				]);
				byte_index = 0;
			}
		}

		if !matches!(byte_index, 0) {
			instr.push(Instr::Wea(Reg::Ac0.into(), 0u8.into(), 0u8.into()));
		}

		instr.extend_from_slice(&ExtInstr::WriteAddrLit(display_final_addr, 0xAA).to_instr());

		Some(Program::from_iter(instr))
	}

	#[test]
	fn buffered_display() -> Result<(), ComponentCreationError> {
		let mut prog = display_prog("Hello world!", 0x1000, 0x1100 - 0x04).unwrap();
		prog.push(Instr::Halt.into());

		let received_message = Arc::new(Mutex::new(false));
		let received_message_closure = Arc::clone(&received_message);

		let (_, state) = exec_vm(
			vec![
				Box::new(BootRom::with_size(prog.encode_words(), 0x1000, 0x0)?),
				Box::new(BufferedDisplay::new(
					0x100,
					move |message| {
						let mut received_message = received_message_closure.lock().unwrap();

						assert!(
							!*received_message,
							"received a message twice (second message: {})",
							message.unwrap_or("<invalid UTF-8 string>")
						);

						let message = message
							.expect("invalid UTF-8 message received")
							.trim_end_matches(char::from(0));

						assert_eq!(
							message, "Hello world!",
							"invalid message received: {message}"
						);

						*received_message = true;
					},
					0x1,
				)?),
			],
			RunConfig::halt_on_exception(),
		);

		assert!(
			state.ex.is_none(),
			"unexpected exception occurred while running the vm"
		);

		assert!(
			*received_message.lock().unwrap(),
			"no message received by buffered display"
		);

		Ok(())
	}
}
