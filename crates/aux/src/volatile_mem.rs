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
