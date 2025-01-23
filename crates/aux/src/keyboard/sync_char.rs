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

#[cfg(test)]
mod tests {
	use std::sync::{Arc, Mutex};

	use crate::{
		error::ComponentCreationError,
		keyboard::SyncCharKeyboard,
		storage::BootRom,
		vmm_tools::{
			asm::{ExtInstr, Instr, Program, Reg},
			debug::{RunConfig, exec_vm},
		},
	};

	static PLACEHOLDER_KEYB_EVENT: char = 'Z';

	fn keyb_prog(input_end_addr: u32) -> Program {
		let mut prog = Program::from_iter(ExtInstr::SetReg(Reg::Ac0, input_end_addr).to_instr());
		prog.extend(ExtInstr::SetReg(Reg::Avr, 0x01).to_prog_words());
		prog.push(Instr::Wea(Reg::Ac0.into(), 0u8.into(), 0u8.into()).into());

		prog
	}

	#[test]
	fn sync_char() -> Result<(), ComponentCreationError> {
		let mut prog = keyb_prog(0x1004);
		prog.push(Instr::Halt.into());

		let received_request = Arc::new(Mutex::new(false));
		let received_request_closure = Arc::clone(&received_request);

		let (mut vm, state) = exec_vm(
			vec![
				Box::new(BootRom::with_size(prog.encode_words(), 0x1000, 0x0)?),
				Box::new(SyncCharKeyboard::new(
					move || {
						let mut received_request = received_request_closure.lock().unwrap();
						assert!(!*received_request, "received a keyboard request twice");
						*received_request = true;

						PLACEHOLDER_KEYB_EVENT
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
			*received_request.lock().unwrap(),
			"no keyboard request was triggered"
		);

		vm.map(|mem| {
			let mut ex = 0;

			let word = mem.read(0x1000, &mut ex);

			assert_eq!(
				ex, 0,
				"exception occurred while reading word at address 0x1000: {ex:#008X}"
			);

			let character = std::char::from_u32(word).unwrap_or_else(|| {
				panic!("got invalid character code from keyboard: {word:#004X}")
			});

			assert_eq!(
				character, PLACEHOLDER_KEYB_EVENT,
				"invalid character from keyboard: {character}"
			);
		});

		Ok(())
	}
}
