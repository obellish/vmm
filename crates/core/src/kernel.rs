use alloc::vec::Vec;
use core::ops::Deref;

use miden_crypto::hash::rpo::RpoDigest;

use crate::{
	errors::KernelError,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Kernel(Vec<RpoDigest>);

impl Kernel {
	pub const MAX_NUM_PROCEDURES: usize = u8::MAX as usize;

	pub fn new(proc_hashes: &[RpoDigest]) -> Result<Self, KernelError> {
		if proc_hashes.len() > Self::MAX_NUM_PROCEDURES {
			Err(KernelError::TooManyProcedures(
				Self::MAX_NUM_PROCEDURES,
				proc_hashes.len(),
			))
		} else {
			let mut hashes = proc_hashes.to_vec();
			hashes.sort_by_key(RpoDigest::as_bytes);

			let duplicated = hashes.windows(2).any(|data| data[0] == data[1]);

			if duplicated {
				Err(KernelError::DuplicatedProcedures)
			} else {
				Ok(Self(hashes))
			}
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[must_use]
	pub fn contains_proc(&self, proc_hash: RpoDigest) -> bool {
		self.0.binary_search(&proc_hash).is_ok()
	}

	#[must_use]
	pub fn proc_hashes(&self) -> &[RpoDigest] {
		self
	}
}

impl Deref for Kernel {
	type Target = [RpoDigest];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Deserializable for Kernel {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let len = source.read_u8()? as usize;
		let kernel = source.read_many::<RpoDigest>(len)?;
		Ok(Self(kernel))
	}
}

impl Serializable for Kernel {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_u8(self.0.len().try_into().expect("too many kernel procedures"));
		target.write_many(&self.0);
	}
}
