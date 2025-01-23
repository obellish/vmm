mod bus;
mod hwb;

use std::{cell::RefCell, rc::Rc};

pub use self::bus::*;
pub(crate) use self::hwb::*;
use super::{cpu::Cpu, mem::MappedMemory};

#[derive(Debug)]
pub struct MotherBoard {
	aux: Vec<Rc<RefCell<Box<dyn Bus>>>>,
	cpu: Cpu,
}

impl MotherBoard {
	pub fn map<T>(&mut self, mapper: impl FnOnce(&mut MappedMemory) -> T) -> T {
		mapper(&mut self.cpu.mem)
	}

	#[must_use]
	pub const fn cpu(&self) -> &Cpu {
		&self.cpu
	}

	pub fn cpu_mut(&mut self) -> &mut Cpu {
		&mut self.cpu
	}

	pub fn reset(&mut self) {
		self.cpu.reset();

		for aux in &self.aux {
			aux.borrow_mut().reset();
		}
	}

	#[must_use]
	pub fn count(&self) -> usize {
		self.aux.len()
	}
}

impl FromIterator<Box<dyn Bus>> for MotherBoard {
	fn from_iter<T: IntoIterator<Item = Box<dyn Bus>>>(iter: T) -> Self {
		let aux = iter
			.into_iter()
			.map(|cp| Rc::new(RefCell::new(cp)))
			.collect::<Vec<_>>();

		assert!(
			u32::try_from(aux.len()).is_ok(),
			"cannot connect more than 2^32 components"
		);

		let mem = MappedMemory::new(HardwareBridge::from_iter(aux.clone()));

		Self {
			cpu: Cpu::new(HardwareBridge::from_iter(aux.clone()), mem),
			aux,
		}
	}
}
