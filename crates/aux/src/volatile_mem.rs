use super::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::metadata::{DeviceCategory, DeviceMetadata, MemoryType},
};

#[derive(Debug)]
pub struct Ram {
	storage: Vec<u32>,
	size: u32,
	hw_id: u64,
}

impl Ram {
	pub fn new(size: u32, hw_id: u64) -> Result<Self, ComponentCreationError> {
		if matches!(size, 0) {
			return Err(ComponentCreationError::CapacityCannotBeZero);
		}

		if !matches!(size % 4, 0) {
			return Err(ComponentCreationError::CapacityMustBeAligned);
		}

		Ok(Self {
			storage: vec![0; size.try_into()?],
			size: size / 4,
			hw_id,
		})
	}

	pub fn from(storage: Vec<u32>, hw_id: u64) -> Result<Self, ComponentCreationError> {
		let size: u32 = storage.len().try_into()?;

		Ok(Self {
			storage,
			size: size / 4,
			hw_id,
		})
	}

	pub fn from_with_size(
		mut storage: Vec<u32>,
		size: u32,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let _: u32 = storage.len().try_into()?;
		let _: usize = size.try_into()?;

		if storage.len() > size as usize {
			return Err(ComponentCreationError::CannotBeLowerThanInitialSize);
		}

		if matches!(size, 0) {
			return Err(ComponentCreationError::CapacityCannotBeZero);
		}

		if !matches!(size % 4, 0) {
			return Err(ComponentCreationError::CapacityMustBeAligned);
		}

		let size = size / 4;

		storage.resize(size as usize, 0);

		Ok(Self {
			storage,
			size,
			hw_id,
		})
	}

	#[must_use]
	pub const fn size(&self) -> u32 {
		self.size
	}
}

impl Bus for Ram {
	fn name(&self) -> &'static str {
		"RAM"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.size * 4,
			DeviceCategory::Memory(MemoryType::Ram),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, _: &mut u16) -> u32 {
		self.storage[addr as usize / 4]
	}

	fn write(&mut self, addr: u32, word: u32, _: &mut u16) {
		self.storage[addr as usize / 4] = word;
	}

	fn reset(&mut self) {
		self.storage = vec![0; self.storage.len()];
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		error::ComponentCreationError,
		storage::BootRom,
		vmm_tools::{
			asm::{ExtInstr, Instr, Program},
			debug::{RunConfig, exec_vm},
		},
		volatile_mem::Ram,
	};

	#[test]
	fn ram() -> Result<(), ComponentCreationError> {
		let mut program =
			Program::from_iter(ExtInstr::WriteAddrLit(0x1000, 0x0123_4567).to_instr());
		program.extend(ExtInstr::WriteAddrLit(0x1008, 0x89AB_CDEF).to_prog_words());
		program.push(Instr::Halt.into());

		let (mut vm, state) = exec_vm(
			vec![
				Box::new(BootRom::with_size(program.encode_words(), 0x1000, 0x0)?),
				Box::new(Ram::new(0x1000, 0x1)?),
			],
			RunConfig::halt_on_exception(),
		);

		assert!(
			state.ex.is_none(),
			"unexpected exception occurred while running the vm"
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
