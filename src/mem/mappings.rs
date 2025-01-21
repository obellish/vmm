#[derive(Debug, Clone, Copy)]
pub struct Mapping {
	pub aux_id: usize,
	pub aux_hw_id: u64,
	pub addr: u32,
	pub size: u32,
}

impl Mapping {
	#[must_use]
	pub const fn end_addr(self) -> u32 {
		self.addr + self.size - 1
	}
}

#[derive(Debug)]
pub struct MappingRange {
	pub start_addr: u32,
	pub end_addr: u32,
}

#[derive(Debug)]
pub struct ContiguousMappingResult {
	pub mapping: Result<MappingRange, Vec<(usize, MappingError)>>,
	pub aux_mapping: Vec<AuxMappingStatus>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug)]
pub struct AuxMappingStatus {
	pub aux_id: usize,
	pub aux_hw_id: u64,
	pub aux_name: String,
	pub aux_mapping: Result<MappingRange, MappingError>,
}

#[derive(Debug, Clone, Copy)]
pub enum MappingError {
	UnknownComponent,
	UnalignedStartAddress,
	UnalignedBusSize,
	UnalignedEndAddress,
	NullOrNegAddressRange,
	AlreadyMapped,
	NullBusSize,
	AddressOverlaps(Mapping),
	MappingTooLarge { aux_size: u32 },
}
