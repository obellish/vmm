mod bootrom;
mod file_backend;
mod persistent;

pub use self::{bootrom::BootRom, file_backend::FileBackedMemory, persistent::PersistentMemory};

#[cfg(test)]
mod tests {
	use super::{BootRom, PersistentMemory};
	use crate::{
		error::ComponentCreationError,
		vmm_tools::{
			asm::{ExtInstr, Instr, Program},
			debug::{RunConfig, exec_vm},
		},
	};

	#[test]
	fn flash_mem() -> Result<(), ComponentCreationError> {
		let mut program =
			Program::from_iter(ExtInstr::WriteAddrLit(0x1000, 0x0123_4567).to_instr());
		program.extend(ExtInstr::WriteAddrLit(0x1008, 0x89AB_CDEF).to_instr());
		program.push(Instr::Halt.into());

		let (mut vm, state) = exec_vm(
			vec![
				Box::new(BootRom::with_size(program.encode_words(), 0x1000, 0x0)?),
				Box::new(PersistentMemory::new(0x1000, 0x1)?),
			],
			RunConfig::halt_on_exception(),
		);

		assert!(
			state.ex.is_none(),
			"unexpected exception occurred while running the VM"
		);

		let (mut err_a, mut err_b, mut err_c) = (0, 0, 0);

		let (word_a, word_b, word_c) = vm.map(|mem| {
			(
				mem.read(0x1000, &mut err_a),
				mem.read(0x1008, &mut err_b),
				mem.read(0x1010, &mut err_c),
			)
		});

		assert_eq!(
			err_a, 0,
			"hardware exception occurred while reading word at address 0x00001000: {err_a:#008X}"
		);

		assert_eq!(
			err_b, 0,
			"hardware exception occurred while reading word at address 0x00001008: {err_b:#008X}"
		);

		assert_eq!(
			err_c, 0,
			"hardware exception occurred while reading word at address 0x00001008: {err_c:#008X}"
		);

		assert_eq!(
			word_a, 0x0123_4567,
			"expected word at address 0x00001000 to contain 0x01234567 but it actually contains {word_a:#010X}"
		);

		assert_eq!(
			word_b, 0x89AB_CDEF,
			"expected word at address 0x00001008 to contain 0x89ABCDEF but it actually contains {word_b:#010X}"
		);

		assert_eq!(
			word_c, 0x0000_0000,
			"expected word at address 0x00001010 to contain 0x00000000 but it actually contains {word_c:#010X}"
		);

		Ok(())
	}
}
