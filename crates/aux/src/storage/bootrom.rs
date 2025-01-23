use crate::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, StorageType},
	},
};

#[derive(Debug)]
pub struct BootRom {
	storage: Vec<u32>,
	len: u32,
	size: u32,
	hw_id: u64,
}

impl BootRom {
	pub fn new(storage: Vec<u32>, hw_id: u64) -> Result<Self, ComponentCreationError> {
		let len: u32 = storage.len().try_into()?;

		Ok(Self {
			storage,
			len,
			size: len,
			hw_id,
		})
	}

	pub fn with_size(
		storage: Vec<u32>,
		size: u32,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let len: u32 = storage.len().try_into()?;

		if storage.len() > size as usize {
			return Err(ComponentCreationError::CannotBeLowerThanInitialSize);
		}

		if matches!(size, 0) {
			return Err(ComponentCreationError::CapacityCannotBeZero);
		}

		if !matches!(size % 4, 0) {
			return Err(ComponentCreationError::CapacityMustBeAligned);
		}

		Ok(Self {
			storage,
			len,
			size: size / 4,
			hw_id,
		})
	}

	#[allow(clippy::len_without_is_empty)]
	#[must_use]
	pub const fn len(&self) -> u32 {
		self.len
	}

	#[must_use]
	pub const fn size(&self) -> u32 {
		self.size
	}
}

impl Bus for BootRom {
	fn name(&self) -> &'static str {
		"BootROM"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.size * 4,
			DeviceCategory::Storage(StorageType::Readonly),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, _: &mut u16) -> u32 {
		let addr = addr / 4;

		if addr < self.len {
			self.storage[addr as usize]
		} else {
			0
		}
	}

	fn write(&mut self, _: u32, _: u32, ex: &mut u16) {
		*ex = AuxHwException::MemoryNotWritable.encode();
	}

	fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
	use crate::{
		error::ComponentCreationError,
		storage::BootRom,
		vmm::board::MotherBoard,
		vmm_tools::{
			asm::{Instr, Program, Reg},
			debug::{RunConfig, prepare_vm, run_vm},
			exceptions::{AuxHwException, NativeException},
		},
	};

	fn prepare(instr: Instr) -> Result<MotherBoard, ComponentCreationError> {
		let prog = Program::from_iter([instr, Instr::Halt]);

		Ok(prepare_vm(vec![Box::new(BootRom::with_size(
			prog.encode_words(),
			0x1000,
			0x0,
		)?)]))
	}

	#[test]
	fn read() -> Result<(), ComponentCreationError> {
		let mut vm = prepare(Instr::Cpy(Reg::A0, 0xABCDu16.into()))?;

		let cpu = vm.cpu_mut();

		let status = run_vm(cpu, RunConfig::new());

		assert_eq!(
			status.cycles, 2,
			"CPU was expected to run 2 cycles, {} cycles run instead",
			status.cycles
		);

		assert_eq!(
			cpu.regs.a[0], 0xABCD,
			"registry a0 was expected to contain 0x0000ABCD, contains {:#010X} instead",
			cpu.regs.a[0]
		);

		Ok(())
	}

	#[test]
	fn write() -> Result<(), ComponentCreationError> {
		let mut vm = prepare(Instr::Wea(0u8.into(), 0u8.into(), 0u8.into()))?;
		let ex = run_vm(vm.cpu_mut(), RunConfig::halt_on_exception())
			.ex
			.expect("no exception occurred while writing BootROM");

		match NativeException::decode_with_mode(ex.raw) {
			Some((NativeException::HardwareException(AuxHwException::MemoryNotWritable), _)) => {}
			Some((NativeException::HardwareException(hw_ex), _)) => {
				panic!("wrong hardware exception occurred while writing BootROM: {hw_ex}")
			}
			Some((ex, _)) => panic!(
				"expected hardware exception while writing BootROM, got non-hardware exception {ex}"
			),
			None => panic!("unknown exception occurred while writing BootROM: {ex:?}"),
		}

		Ok(())
	}
}
