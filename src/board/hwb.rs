use std::{cell::RefCell, rc::Rc};

use super::Bus;

#[derive(Debug)]
struct AuxWithCache {
	shared_bus: Rc<RefCell<Box<dyn Bus>>>,
	cache: AuxCache,
}

impl AuxWithCache {
	pub fn create_from_aux(id: usize, shared_bus: Rc<RefCell<Box<dyn Bus>>>) -> Self {
		let bus = shared_bus.borrow();

		let mut name = bus.name().to_owned();

		while name.len() > 32 {
			name.pop();
		}

		let metadata = bus.metadata();
		let hw_id = (u64::from(metadata[0]) << 32) + u64::from(metadata[1]);
		let size = metadata[2];

		std::mem::drop(bus);

		Self {
			shared_bus,
			cache: AuxCache {
				id,
				hw_id,
				name,
				metadata,
				size,
			},
		}
	}
}

#[derive(Debug, Clone)]
pub struct AuxCache {
	pub id: usize,
	pub hw_id: u64,
	pub name: String,
	pub metadata: [u32; 8],
	pub size: u32,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct HardwareBridge {
	aux: Vec<AuxWithCache>,
}

#[allow(clippy::needless_pass_by_ref_mut)]
impl HardwareBridge {
	pub fn count(&self) -> usize {
		self.aux.len()
	}

	pub fn cache_of(&self, aux_id: usize) -> Option<&AuxCache> {
		self.aux.get(aux_id).map(|entry| &entry.cache)
	}

	pub fn name_of(&self, aux_id: usize) -> Option<&str> {
		self.cache_of(aux_id).map(|cache| cache.name.as_str())
	}

	pub fn metadata_of(&self, aux_id: usize) -> Option<[u32; 8]> {
		self.cache_of(aux_id).map(|cache| cache.metadata)
	}

	pub fn hw_id_of(&self, aux_id: usize) -> Option<u64> {
		self.cache_of(aux_id).map(|cache| cache.hw_id)
	}

	pub fn size_of(&self, aux_id: usize) -> Option<u32> {
		self.cache_of(aux_id).map(|cache| cache.size)
	}

	pub fn read(&mut self, aux_id: usize, addr: u32, ex: &mut u16) -> Option<u32> {
		assert_eq!(
			addr % 4,
			0,
			"hardware bridge does not support reading from unaligned addresses"
		);

		self.aux
			.get(aux_id)
			.map(|aux| aux.shared_bus.borrow_mut().read(addr, ex))
	}

	pub fn write(&mut self, aux_id: usize, addr: u32, word: u32, ex: &mut u16) -> Option<()> {
		assert_eq!(
			addr % 4,
			0,
			"hardware bridge does not support writing to unaligned addresses"
		);

		self.aux
			.get(aux_id)
			.map(|aux| aux.shared_bus.borrow_mut().write(addr, word, ex))
	}

	pub fn reset(&mut self, aux_id: usize) -> Option<()> {
		self.aux
			.get(aux_id)
			.map(|aux| aux.shared_bus.borrow_mut().reset())
	}
}

impl FromIterator<Rc<RefCell<Box<dyn Bus>>>> for HardwareBridge {
	fn from_iter<T: IntoIterator<Item = Rc<RefCell<Box<dyn Bus>>>>>(iter: T) -> Self {
		Self {
			aux: iter
				.into_iter()
				.enumerate()
				.map(|(id, shared_bus)| {
					assert!(
						id < u32::MAX as usize,
						"hardware bridge cannot handle more than 2^32 components"
					);

					AuxWithCache::create_from_aux(id, shared_bus)
				})
				.collect(),
		}
	}
}
