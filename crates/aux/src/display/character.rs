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

#[cfg(test)]
mod tests {
	use std::sync::{Arc, Mutex};

	use crate::{
		display::CharDisplay,
		error::ComponentCreationError,
		storage::BootRom,
		vmm_tools::{
			asm::{ExtInstr, Instr, Program},
			debug::{RunConfig, exec_vm},
		},
	};

	fn display_prog(character: char, display_addr: u32) -> Program {
		Program::from_iter(ExtInstr::WriteAddrLit(display_addr, character as u32).to_instr())
	}

	#[test]
	fn buffered_display() -> Result<(), ComponentCreationError> {
		let mut prog = display_prog('Z', 0x1000);
		prog.push(Instr::Halt.into());

		let received_message = Arc::new(Mutex::new(false));
		let received_message_closure = Arc::clone(&received_message);

		let (_, state) = exec_vm(
			vec![
				Box::new(BootRom::with_size(prog.encode_words(), 0x1000, 0x0)?),
				Box::new(CharDisplay::new(
					move |message| {
						let mut received_message = received_message_closure.lock().unwrap();

						assert!(
							!*received_message,
							"received a message twice (second message: {})",
							message.map_or_else(
								|_| String::from("<invalid UTF-8 character>"),
								String::from
							)
						);

						let message = message.expect("invalid UTF-8 character received");

						assert_eq!(message, 'Z', "invalid character received: {message}");

						*received_message = true;
					},
					0x1,
				)),
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
