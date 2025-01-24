#![allow(clippy::trivially_copy_pass_by_ref)]
#[cfg(feature = "serde")]
mod serde;

use alloc::{string::String, vec::Vec};
use core::{
	mem::{size_of, transmute_copy},
	ops::Deref,
	slice::{self, from_raw_parts},
};

use super::{Digest, ElementHasher, Hasher};
use crate::{
	Felt, FieldElement,
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, HexParseError, Serializable,
		bytes_to_hex_string, hex_to_bytes,
	},
};

const DIGEST32_BYTES: usize = 32;
const DIGEST24_BYTES: usize = 24;
const DIGEST20_BYTES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Blake3Digest<const N: usize>([u8; N]);

impl<const N: usize> Blake3Digest<N> {
	#[must_use]
	pub const fn digests_as_bytes(digests: &[Self]) -> &[u8] {
		let p = digests.as_ptr();
		let len = digests.len() * N;
		unsafe { slice::from_raw_parts(p.cast(), len) }
	}
}

impl<const N: usize> Default for Blake3Digest<N> {
	fn default() -> Self {
		Self([0; N])
	}
}

impl<const N: usize> Deref for Blake3Digest<N> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<const N: usize> Deserializable for Blake3Digest<N> {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		source.read_array().map(Self)
	}
}

impl<const N: usize> Digest for Blake3Digest<N> {
	fn as_bytes(&self) -> [u8; 32] {
		assert!(N <= 32, "digest currently only supports 32 bytes");
		expand_bytes(&self.0)
	}
}

impl<const N: usize> From<Blake3Digest<N>> for [u8; N] {
	fn from(value: Blake3Digest<N>) -> Self {
		value.0
	}
}

impl<const N: usize> From<[u8; N]> for Blake3Digest<N> {
	fn from(value: [u8; N]) -> Self {
		Self(value)
	}
}

impl<const N: usize> From<Blake3Digest<N>> for String {
	fn from(value: Blake3Digest<N>) -> Self {
		bytes_to_hex_string(value.as_bytes())
	}
}

impl<const N: usize> Serializable for Blake3Digest<N> {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_bytes(self);
	}
}

impl<const N: usize> TryFrom<&str> for Blake3Digest<N> {
	type Error = HexParseError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		hex_to_bytes(value).map(Into::into)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blake3_256;

impl Blake3_256 {
	#[must_use]
	pub fn hash(bytes: &[u8]) -> Blake3Digest<DIGEST32_BYTES> {
		<Self as Hasher>::hash(bytes)
	}

	#[must_use]
	pub fn merge(values: &[Blake3Digest<DIGEST32_BYTES>; 2]) -> Blake3Digest<DIGEST32_BYTES> {
		<Self as Hasher>::merge(values)
	}

	pub fn hash_elements<E>(elements: &[E]) -> Blake3Digest<DIGEST32_BYTES>
	where
		E: FieldElement<BaseField = Felt>,
	{
		<Self as ElementHasher>::hash_elements(elements)
	}
}

impl ElementHasher for Blake3_256 {
	type BaseField = Felt;

	fn hash_elements<E>(elements: &[E]) -> Self::Digest
	where
		E: FieldElement<BaseField = Self::BaseField>,
	{
		Blake3Digest(hash_elements(elements))
	}
}

impl Hasher for Blake3_256 {
	type Digest = Blake3Digest<32>;

	const COLLISION_RESISTANCE: u32 = 128;

	fn hash(bytes: &[u8]) -> Self::Digest {
		Blake3Digest(blake3::hash(bytes).into())
	}

	fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
		Self::hash(prepare_merge(values))
	}

	fn merge_many(values: &[Self::Digest]) -> Self::Digest {
		Blake3Digest(blake3::hash(Blake3Digest::digests_as_bytes(values)).into())
	}

	fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
		let mut hasher = blake3::Hasher::new();
		hasher.update(&seed.0);
		hasher.update(&value.to_le_bytes());
		Blake3Digest(hasher.finalize().into())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blake3_192;

impl Blake3_192 {
	#[must_use]
	pub fn hash(bytes: &[u8]) -> Blake3Digest<DIGEST24_BYTES> {
		<Self as Hasher>::hash(bytes)
	}

	#[must_use]
	pub fn merge(values: &[Blake3Digest<DIGEST24_BYTES>; 2]) -> Blake3Digest<DIGEST24_BYTES> {
		<Self as Hasher>::merge(values)
	}

	pub fn hash_elements<E>(elements: &[E]) -> Blake3Digest<DIGEST24_BYTES>
	where
		E: FieldElement<BaseField = Felt>,
	{
		<Self as ElementHasher>::hash_elements(elements)
	}
}

impl ElementHasher for Blake3_192 {
	type BaseField = Felt;

	fn hash_elements<E>(elements: &[E]) -> Self::Digest
	where
		E: FieldElement<BaseField = Self::BaseField>,
	{
		Blake3Digest(hash_elements(elements))
	}
}

impl Hasher for Blake3_192 {
	type Digest = Blake3Digest<24>;

	const COLLISION_RESISTANCE: u32 = 96;

	fn hash(bytes: &[u8]) -> Self::Digest {
		Blake3Digest(*shrink_bytes(&blake3::hash(bytes).into()))
	}

	fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
		Self::hash(prepare_merge(values))
	}

	fn merge_many(values: &[Self::Digest]) -> Self::Digest {
		let bytes = values.iter().flat_map(Digest::as_bytes).collect::<Vec<_>>();
		Blake3Digest(*shrink_bytes(&blake3::hash(&bytes).into()))
	}

	fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
		let mut hasher = blake3::Hasher::new();
		hasher.update(&seed.0);
		hasher.update(&value.to_le_bytes());
		Blake3Digest(*shrink_bytes(&hasher.finalize().into()))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blake3_160;

impl Blake3_160 {
	#[must_use]
	pub fn hash(bytes: &[u8]) -> Blake3Digest<DIGEST20_BYTES> {
		<Self as Hasher>::hash(bytes)
	}

	#[must_use]
	pub fn merge(values: &[Blake3Digest<DIGEST20_BYTES>; 2]) -> Blake3Digest<DIGEST20_BYTES> {
		<Self as Hasher>::merge(values)
	}

	pub fn hash_elements<E>(elements: &[E]) -> Blake3Digest<DIGEST20_BYTES>
	where
		E: FieldElement<BaseField = Felt>,
	{
		<Self as ElementHasher>::hash_elements(elements)
	}
}

impl ElementHasher for Blake3_160 {
	type BaseField = Felt;

	fn hash_elements<E>(elements: &[E]) -> Self::Digest
	where
		E: FieldElement<BaseField = Self::BaseField>,
	{
		Blake3Digest(hash_elements(elements))
	}
}

impl Hasher for Blake3_160 {
	type Digest = Blake3Digest<20>;

	const COLLISION_RESISTANCE: u32 = 80;

	fn hash(bytes: &[u8]) -> Self::Digest {
		Blake3Digest(*shrink_bytes(&blake3::hash(bytes).into()))
	}

	fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
		Self::hash(prepare_merge(values))
	}

	fn merge_many(values: &[Self::Digest]) -> Self::Digest {
		let bytes = values.iter().flat_map(Digest::as_bytes).collect::<Vec<_>>();
		Blake3Digest(*shrink_bytes(&blake3::hash(&bytes).into()))
	}

	fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
		let mut hasher = blake3::Hasher::new();
		hasher.update(&seed.0);
		hasher.update(&value.to_le_bytes());
		Blake3Digest(*shrink_bytes(&hasher.finalize().into()))
	}
}

fn shrink_bytes<const M: usize, const N: usize>(bytes: &[u8; M]) -> &[u8; N] {
	assert!(
		M >= N,
		"N should fit in M so it can be safely transmuted into a smaller slice"
	);

	unsafe { &*core::ptr::from_ref::<[u8; M]>(bytes).cast::<[u8; N]>() }
}

fn hash_elements<E, const N: usize>(elements: &[E]) -> [u8; N]
where
	E: FieldElement<BaseField = Felt>,
{
	let digest = if Felt::IS_CANONICAL {
		blake3::hash(E::elements_as_bytes(elements))
	} else {
		let mut hasher = blake3::Hasher::new();

		let mut buf = [0u8; 64];
		let mut chunk_iter = E::slice_as_base_elements(elements).chunks_exact(8);
		for chunk in chunk_iter.by_ref() {
			for i in 0..8 {
				buf[i * 8..(i + 1) * 8].copy_from_slice(&chunk[i].as_int().to_le_bytes());
			}
			hasher.update(&buf);
		}

		for element in chunk_iter.remainder() {
			hasher.update(&element.as_int().to_le_bytes());
		}

		hasher.finalize()
	};

	*shrink_bytes(&digest.into())
}

fn expand_bytes<const M: usize, const N: usize>(bytes: &[u8; M]) -> [u8; N] {
	assert!(M <= N, "M should fit in N so M can be expanded");

	if M == N {
		unsafe { transmute_copy(bytes) }
	} else {
		let mut expanded = [0u8; N];
		expanded[..M].copy_from_slice(bytes);
		expanded
	}
}

fn prepare_merge<D, const N: usize>(args: &[D; N]) -> &[u8]
where
	D: Deref<Target = [u8]>,
{
	assert!(N > 0, "N shouldn't represent an empty slice");
	let values = args.as_ptr().cast::<u8>();
	let len = size_of::<D>() * N;

	let bytes = unsafe { from_raw_parts(values, len) };
	debug_assert_eq!(&*args[0], &bytes[..len / N]);
	bytes
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use proptest::prelude::*;
	use rand_utils::rand_vector;

	use super::*;

	fn compute_expected_element_hash(elements: &[Felt]) -> blake3::Hash {
		let mut bytes = Vec::new();
		for element in elements {
			bytes.extend_from_slice(&element.as_int().to_le_bytes());
		}

		blake3::hash(&bytes)
	}

	#[test]
	fn blake3_hash_elements() {
		let elements = rand_vector::<Felt>(16);
		let expected = compute_expected_element_hash(&elements);
		let actual: [u8; 32] = hash_elements(&elements);
		assert_eq!(&expected, &actual);

		let elements = rand_vector::<Felt>(17);
		let expected = compute_expected_element_hash(&elements);
		let actual: [u8; 32] = hash_elements(&elements);
		assert_eq!(&expected, &actual);
	}

	proptest! {
		#[test]
		fn blake160_wont_panic_with_arbitrary_input(ref vec in any::<Vec<u8>>()) {
			_ = Blake3_160::hash(vec);
		}

		#[test]
		fn blake192_wont_panic_with_arbitrary_input(ref vec in any::<Vec<u8>>()) {
			_ = Blake3_192::hash(vec);
		}

		#[test]
		fn blake256_wont_panic_with_arbitrary_input(ref vec in any::<Vec<u8>>()) {
			_ = Blake3_256::hash(vec);
		}
	}
}
