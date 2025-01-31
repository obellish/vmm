pub mod sync;

pub mod math {
	pub use winter_math::batch_inversion;
}

use alloc::vec::Vec;
use core::{
	fmt::Debug,
	ops::{Bound, Range},
};

pub use miden_crypto::utils::{
	ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
	collections, uninit_vector,
};
pub use vmm_formatting::hex::{DisplayHex, ToHex, to_hex};
#[cfg(feature = "std")]
pub use winter_utils::ReadAdapter;
pub use winter_utils::group_slice_elements;

use super::Felt;

pub trait IntoBytes<const N: usize> {
	fn into_bytes(self) -> [u8; N];
}

impl IntoBytes<32> for [Felt; 4] {
	fn into_bytes(self) -> [u8; 32] {
		let mut result = [0; 32];

		result[..=7].copy_from_slice(&self[0].as_int().to_le_bytes());
		result[8..=15].copy_from_slice(&self[1].as_int().to_le_bytes());
		result[16..=23].copy_from_slice(&self[2].as_int().to_le_bytes());
		result[24..].copy_from_slice(&self[3].as_int().to_le_bytes());

		result
	}
}

pub trait PushMany<T> {
	fn push_many(&mut self, value: T, n: usize);
}

impl<T: Copy> PushMany<T> for Vec<T> {
	fn push_many(&mut self, value: T, n: usize) {
		let new_len = self.len() + n;
		self.resize(new_len, value);
	}
}

pub trait ToElements {
	fn to_elements(&self) -> Vec<Felt>;
}

impl<const N: usize> ToElements for [u64; N] {
	fn to_elements(&self) -> Vec<Felt> {
		self.as_slice().to_elements()
	}
}

impl ToElements for Vec<u64> {
	fn to_elements(&self) -> Vec<Felt> {
		(**self).to_elements()
	}
}

impl ToElements for [u64] {
	fn to_elements(&self) -> Vec<Felt> {
		self.iter().map(|&v| Felt::new(v)).collect()
	}
}

#[must_use]
pub const fn range(start: usize, len: usize) -> Range<usize> {
	Range {
		start,
		end: start + len,
	}
}

#[must_use]
pub fn bound_into_included_u64<I>(bound: Bound<&I>, is_start: bool) -> u64
where
	I: Clone + Into<u64>,
{
	match bound {
		Bound::Excluded(i) => i.clone().into().saturating_sub(1),
		Bound::Included(i) => i.clone().into(),
		Bound::Unbounded => {
			if is_start {
				0
			} else {
				u64::MAX
			}
		}
	}
}

#[must_use]
pub fn new_array_vec<T: Debug, const N: usize>(capacity: usize) -> [Vec<T>; N] {
	(0..N)
		.map(|_| Vec::with_capacity(capacity))
		.collect::<Vec<_>>()
		.try_into()
		.expect("failed to convert vector to array")
}
