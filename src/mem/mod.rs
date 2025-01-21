mod mappings;

pub use self::mappings::*;
use super::board::HardwareBridge;

pub struct MappedMemory {
	bridge: HardwareBridge,
	mappings: Vec<Mapping>,
}

impl MappedMemory {
	#[must_use]
	pub const fn new(bridge: HardwareBridge) -> Self {
		Self {
			bridge,
			mappings: Vec::new(),
		}
	}

	pub fn map(&mut self, addr: u32, aux_id: usize) -> Result<MappingRange, MappingError> {
		self.internal_map(addr, None, aux_id)
	}

	pub fn map_abs(
		&mut self,
		addr: u32,
		addr_end: u32,
		aux_id: usize,
	) -> Result<MappingRange, MappingError> {
		self.internal_map(addr, Some(addr_end), aux_id)
	}

	pub fn map_contiguous(
		&mut self,
		addr: u32,
		aux_ids: impl AsRef<[usize]>,
	) -> ContiguousMappingResult {
		let mut aux_mapping = Vec::new();
		let mut failed = Vec::new();
		let mut last_addr = addr;
		let mut max_addr = addr;

		for aux_id in aux_ids.as_ref() {
			let result = self.map(last_addr, *aux_id);

			match result {
				Ok(MappingRange { end_addr, .. }) => {
					if end_addr > max_addr {
						max_addr = end_addr;
					}

					last_addr = end_addr + 4;
				}
				Err(err) => failed.push((*aux_id, err)),
			}

			aux_mapping.push(AuxMappingStatus {
				aux_id: *aux_id,
				aux_hw_id: self.bridge.hw_id_of(*aux_id).unwrap(),
				aux_name: self.bridge.name_of(*aux_id).unwrap().to_owned(),
				aux_mapping: result,
			});
		}

		ContiguousMappingResult {
			mapping: if failed.is_empty() {
				Ok(MappingRange {
					start_addr: addr,
					end_addr: max_addr,
				})
			} else {
				Err(failed)
			},
			aux_mapping,
		}
	}

	pub fn read(&mut self, addr: u32, ex: &mut u16) -> u32 {
		assert_eq!(
			addr % 4,
			0,
			"memory does not support reading from unaligned addresses"
		);

		if let Some(mapping) = self
			.mappings
			.iter()
			.find(|mapping| mapping.addr <= addr && addr <= mapping.end_addr())
		{
			self.bridge
				.read(mapping.aux_id, addr - mapping.addr, ex)
				.unwrap()
		} else {
			if cfg!(debug_assertions) {
				eprintln!("warning: tried to read non-mapped memory at address {addr:#010X}");
			}

			0
		}
	}

	pub fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		assert_eq!(
			addr % 4,
			0,
			"memory does not support writing to unaligned addresses"
		);

		if let Some(mapping) = self
			.mappings
			.iter()
			.find(|mapping| mapping.addr <= addr && addr <= mapping.end_addr())
		{
			self.bridge
				.write(mapping.aux_id, addr - mapping.addr, word, ex)
				.unwrap();
		} else if cfg!(debug_assertions) {
			eprintln!("warning: tried to write non-mapped memory at address {addr:#010X}");
		}
	}

	#[must_use]
	pub fn get_mapping(&self, aux_id: usize) -> Option<&Mapping> {
		self.mappings
			.iter()
			.find(|mapping| mapping.aux_id == aux_id)
	}

	fn internal_map(
		&mut self,
		start_addr: u32,
		end_addr: Option<u32>,
		aux_id: usize,
	) -> Result<MappingRange, MappingError> {
		let aux_size = self
			.bridge
			.size_of(aux_id)
			.ok_or(MappingError::UnknownComponent)?;

		let end_addr = end_addr.unwrap_or(start_addr + aux_size - 4);

		if !matches!(start_addr % 4, 0) {
			return Err(MappingError::UnalignedStartAddress);
		}

		if matches!(aux_size, 0) {
			return Err(MappingError::NullBusSize);
		}

		if !matches!(aux_size % 4, 0) {
			return Err(MappingError::UnalignedBusSize);
		}

		if !matches!(end_addr % 4, 0) {
			return Err(MappingError::UnalignedEndAddress);
		}

		if start_addr == end_addr + 4 || start_addr > end_addr {
			return Err(MappingError::NullOrNegAddressRange);
		}

		if end_addr - start_addr > aux_size {
			return Err(MappingError::MappingTooLarge { aux_size });
		}

		if self.get_mapping(aux_id).is_some() {
			return Err(MappingError::AlreadyMapped);
		}

		if let Some(mapping) = self
			.mappings
			.iter()
			.find(|mapping| mapping.addr <= end_addr && start_addr <= mapping.end_addr())
			.copied()
		{
			Err(MappingError::AddressOverlaps(mapping))
		} else {
			self.mappings.push(Mapping {
				aux_id,
				aux_hw_id: self.bridge.hw_id_of(aux_id).expect(
					"internal error: failed to get HW ID of component after mapping validation",
				),
				addr: start_addr,
				size: aux_size,
			});

			Ok(MappingRange {
				start_addr,
				end_addr,
			})
		}
	}
}
