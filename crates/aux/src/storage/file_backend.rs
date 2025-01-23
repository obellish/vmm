use std::{
	cmp::Ordering,
	fs::{File, OpenOptions},
	io::{Read, Seek, SeekFrom, Write},
	path::Path,
};

use crate::{
	error::ComponentCreationError,
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{DeviceCategory, DeviceMetadata, StorageType},
	},
};

#[derive(Debug)]
pub struct FileBackedMemory {
	handler: File,
	size: u32,
	real_size: u32,
	writable: bool,
	hw_id: u64,
}

impl FileBackedMemory {
	fn open(
		path: impl AsRef<Path>,
		writable: bool,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let handler = OpenOptions::new().read(true).write(writable).open(path)?;

		let unaligned_real_size: u32 = handler.metadata()?.len().try_into()?;

		let real_size = (unaligned_real_size / 4) * 4;

		if real_size != unaligned_real_size {
			eprintln!(
				"warning: opened unaligned file as aligned (rounded size to nearest lower multiple of 4 bytes)"
			);
		}

		let _: usize = real_size.try_into()?;

		Ok(Self {
			size: real_size,
			real_size,
			handler,
			writable,
			hw_id,
		})
	}

	pub fn writable(path: impl AsRef<Path>, hw_id: u64) -> Result<Self, ComponentCreationError> {
		Self::open(path, true, hw_id)
	}

	pub fn writable_with_size(
		path: impl AsRef<Path>,
		size: u32,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let mut mem = Self::writable(path, hw_id)?;

		match mem.real_size.cmp(&size) {
			Ordering::Greater => mem.size = size,
			Ordering::Less => mem.handler.set_len(size.into())?,
			Ordering::Equal => {}
		}

		Ok(mem)
	}

	pub fn readonly(path: impl AsRef<Path>, hw_id: u64) -> Result<Self, ComponentCreationError> {
		Self::open(path, false, hw_id)
	}

	pub fn readonly_with_size(
		path: impl AsRef<Path>,
		size: u32,
		hw_id: u64,
	) -> Result<Self, ComponentCreationError> {
		let mut mem = Self::readonly(path, hw_id)?;
		mem.size = size;
		Ok(mem)
	}
}

impl Bus for FileBackedMemory {
	fn name(&self) -> &'static str {
		"Persistent Memory"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			self.size * 4,
			DeviceCategory::Storage(StorageType::Persistent),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, ex: &mut u16) -> u32 {
		if addr >= self.real_size {
			return 0;
		}

		let mut buffer = [0; 4];

		if self.handler.seek(SeekFrom::Start(addr.into())).is_err() {
			*ex = AuxHwException::GenericPhysicalReadError.encode();
			return 0;
		}

		if self.handler.read_exact(&mut buffer).is_err() {
			*ex = AuxHwException::GenericPhysicalReadError.encode();
			return 0;
		}

		u32::from_be_bytes(buffer)
	}

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16) {
		if !self.writable {
			*ex = AuxHwException::MemoryNotWritable.into();
		} else if addr < self.real_size {
			self.handler.seek(SeekFrom::Start(addr.into())).unwrap();
			self.handler.write_all(&word.to_be_bytes()).unwrap();
		}
	}

	fn reset(&mut self) {}
}
