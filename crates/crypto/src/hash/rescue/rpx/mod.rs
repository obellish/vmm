mod digest;

use core::ops::Range;

use winter_math::StarkField;

pub use self::digest::{RpxDigest, RpxDigestError};
use super::{
	ARK1, ARK2, BINARY_CHUNK_SIZE, CAPACITY_RANGE, DIGEST_RANGE, INPUT1_RANGE, INPUT2_RANGE,
	NUM_ROUNDS, RATE_RANGE, RATE_WIDTH, STATE_WIDTH, add_constants,
	add_constants_and_apply_inv_sbox, add_constants_and_apply_sbox, apply_inv_sbox, apply_sbox,
	mds::{MDS, apply_mds},
};
use crate::{
	CubeExtension, Felt, FieldElement, ZERO,
	hash::{ElementHasher, Hasher},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rpx256;

#[allow(clippy::trivially_copy_pass_by_ref)]
impl Rpx256 {
	pub const ARK1: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = ARK1;
	pub const ARK2: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = ARK2;
	pub const CAPACITY_RANGE: Range<usize> = CAPACITY_RANGE;
	pub const DIGEST_RANGE: Range<usize> = DIGEST_RANGE;
	pub const MDS: [[Felt; STATE_WIDTH]; STATE_WIDTH] = MDS;
	pub const NUM_ROUNDS: usize = NUM_ROUNDS;
	pub const RATE_RANGE: Range<usize> = RATE_RANGE;
	pub const STATE_WIDTH: usize = STATE_WIDTH;

	#[must_use]
	pub fn hash(bytes: &[u8]) -> RpxDigest {
		<Self as Hasher>::hash(bytes)
	}

	#[must_use]
	pub fn merge(values: &[RpxDigest; 2]) -> RpxDigest {
		<Self as Hasher>::merge(values)
	}

	pub fn hash_elements<E>(elements: &[E]) -> RpxDigest
	where
		E: FieldElement<BaseField = Felt>,
	{
		<Self as ElementHasher>::hash_elements(elements)
	}

	#[must_use]
	pub fn merge_in_domain(values: &[RpxDigest; 2], domain: Felt) -> RpxDigest {
		let mut state = [ZERO; STATE_WIDTH];
		let it = RpxDigest::digests_as_elements_iter(values.iter());
		for (i, v) in it.copied().enumerate() {
			state[RATE_RANGE.start + i] = v;
		}

		state[CAPACITY_RANGE.start + 1] = domain;

		Self::apply_permutation(&mut state);
		RpxDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	pub fn apply_permutation(state: &mut [Felt; STATE_WIDTH]) {
		Self::apply_fb_round(state, 0);
		Self::apply_ext_round(state, 1);
		Self::apply_fb_round(state, 2);
		Self::apply_ext_round(state, 3);
		Self::apply_fb_round(state, 4);
		Self::apply_ext_round(state, 5);
		Self::apply_final_round(state, 6);
	}

	pub fn apply_fb_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
		apply_mds(state);
		if !add_constants_and_apply_sbox(state, &ARK1[round]) {
			add_constants(state, &ARK1[round]);
			apply_sbox(state);
		}

		apply_mds(state);
		if !add_constants_and_apply_inv_sbox(state, &ARK2[round]) {
			add_constants(state, &ARK2[round]);
			apply_inv_sbox(state);
		}
	}

	pub fn apply_ext_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
		add_constants(state, &ARK1[round]);

		let [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11] = *state;
		let ext0 = Self::exp7(CubicExtElement::new(s0, s1, s2));
		let ext1 = Self::exp7(CubicExtElement::new(s3, s4, s5));
		let ext2 = Self::exp7(CubicExtElement::new(s6, s7, s8));
		let ext3 = Self::exp7(CubicExtElement::new(s9, s10, s11));

		let arr_ext = [ext0, ext1, ext2, ext3];
		*state = CubicExtElement::slice_as_base_elements(&arr_ext)
			.try_into()
			.expect("shouldn't fail");
	}

	pub fn apply_final_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
		apply_mds(state);
		add_constants(state, &ARK1[round]);
	}

	#[must_use]
	pub fn exp7(x: CubicExtElement) -> CubicExtElement {
		let x2 = x.square();
		let x4 = x2.square();

		let x3 = x2 * x;
		x3 * x4
	}
}

impl ElementHasher for Rpx256 {
	type BaseField = Felt;

	fn hash_elements<E>(elements: &[E]) -> Self::Digest
	where
		E: FieldElement<BaseField = Self::BaseField>,
	{
		let elements = E::slice_as_base_elements(elements);

		let mut state = [ZERO; STATE_WIDTH];
		state[CAPACITY_RANGE.start] = Felt::from((elements.len() % RATE_WIDTH) as u8);

		let mut i = 0;
		for &element in elements {
			state[RATE_RANGE.start + i] = element;
			i += 1;
			if matches!(i % RATE_WIDTH, 0) {
				Self::apply_permutation(&mut state);
				i = 0;
			}
		}

		if i > 0 {
			while i != RATE_WIDTH {
				state[RATE_RANGE.start + i] = ZERO;
				i += 1;
			}
			Self::apply_permutation(&mut state);
		}

		RpxDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}
}

impl Hasher for Rpx256 {
	type Digest = RpxDigest;

	const COLLISION_RESISTANCE: u32 = 128;

	fn hash(bytes: &[u8]) -> Self::Digest {
		let mut state = [ZERO; STATE_WIDTH];

		let num_field_elem = bytes.len().div_ceil(BINARY_CHUNK_SIZE);

		state[CAPACITY_RANGE.start] =
			Felt::from((RATE_WIDTH + (num_field_elem % RATE_WIDTH)) as u8);

		let mut buf = [0u8; 8];

		let mut current_chunk_idx = 0usize;

		let last_chunk_idx = if matches!(num_field_elem, 0) {
			current_chunk_idx
		} else {
			num_field_elem - 1
		};
		let rate_pos = bytes.chunks(BINARY_CHUNK_SIZE).fold(0, |rate_pos, chunk| {
			if current_chunk_idx == last_chunk_idx {
				buf.fill(0);
				buf[..chunk.len()].copy_from_slice(chunk);
				buf[chunk.len()] = 1;
			} else {
				buf[..BINARY_CHUNK_SIZE].copy_from_slice(chunk);
			}
			current_chunk_idx += 1;

			state[RATE_RANGE.start + rate_pos] = Felt::new(u64::from_le_bytes(buf));

			if rate_pos == RATE_WIDTH - 1 {
				Self::apply_permutation(&mut state);
				0
			} else {
				rate_pos + 1
			}
		});

		if !matches!(rate_pos, 0) {
			state[RATE_RANGE.start + rate_pos..RATE_RANGE.end].fill(ZERO);
			Self::apply_permutation(&mut state);
		}

		RpxDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
		let mut state = [ZERO; STATE_WIDTH];
		let it = RpxDigest::digests_as_elements_iter(values.iter());
		for (i, v) in it.copied().enumerate() {
			state[RATE_RANGE.start + i] = v;
		}

		Self::apply_permutation(&mut state);
		RpxDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	fn merge_many(values: &[Self::Digest]) -> Self::Digest {
		Self::hash_elements(RpxDigest::digests_as_elements(values))
	}

	fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
		let mut state = [ZERO; STATE_WIDTH];
		state[INPUT1_RANGE].copy_from_slice(seed.as_elements());
		state[INPUT2_RANGE.start] = Felt::new(value);

		if value < Felt::MODULUS {
			state[CAPACITY_RANGE.start] = Felt::from(5u8);
		} else {
			state[INPUT2_RANGE.start + 1] = Felt::new(value / Felt::MODULUS);
			state[CAPACITY_RANGE.start] = Felt::from(6u8);
		}

		Self::apply_permutation(&mut state);
		RpxDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}
}

pub type CubicExtElement = CubeExtension<Felt>;

#[cfg(test)]
mod tests {
	use alloc::{collections::BTreeSet, vec::Vec};

	use proptest::prelude::*;
	use rand_utils::rand_value;

	use super::{Felt, Hasher, Rpx256, StarkField, ZERO};
	use crate::{ONE, hash::rescue::RpxDigest};

	fn get_random_array<const N: usize>() -> [Felt; N] {
		core::array::from_fn(|_| Felt::new(rand_value()))
	}

	#[test]
	fn hash_elements_vs_merge() {
		let elements = get_random_array::<8>();

		let digests = [
			RpxDigest::new(elements[..4].try_into().unwrap()),
			RpxDigest::new(elements[4..].try_into().unwrap()),
		];

		let m_result = Rpx256::merge(&digests);
		let h_result = Rpx256::hash_elements(&elements);
		assert_eq!(m_result, h_result);
	}

	#[test]
	fn merge_vs_merge_in_domain() {
		let elements = get_random_array::<8>();

		let digests = [
			RpxDigest::new(elements[..4].try_into().unwrap()),
			RpxDigest::new(elements[4..].try_into().unwrap()),
		];
		let merge_result = Rpx256::merge(&digests);

		let domain = ZERO;
		let merge_in_domain_result = Rpx256::merge_in_domain(&digests, domain);
		assert_eq!(merge_result, merge_in_domain_result);

		let domain = ONE;

		let merge_in_domain_result = Rpx256::merge_in_domain(&digests, domain);
		assert_ne!(merge_result, merge_in_domain_result);
	}

	#[test]
	fn hash_elements_vs_merge_with_int() {
		let tmp = get_random_array();
		let seed = RpxDigest::new(tmp);

		let val = Felt::new(rand_value());
		let m_result = Rpx256::merge_with_int(seed, val.as_int());

		let mut elements = seed.as_elements().to_vec();
		elements.push(val);
		let h_result = Rpx256::hash_elements(&elements);

		assert_eq!(m_result, h_result);

		let val = Felt::MODULUS + 2;
		let m_result = Rpx256::merge_with_int(seed, val);

		let mut elements = seed.as_elements().to_vec();
		elements.extend([Felt::new(val), ONE]);
		let h_result = Rpx256::hash_elements(&elements);

		assert_eq!(m_result, h_result);
	}

	#[test]
	fn hash_padding() {
		let r1 = Rpx256::hash(&[1, 2, 3]);
		let r2 = Rpx256::hash(&[1, 2, 3, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpx256::hash(&[1, 2, 3, 4, 5, 6]);
		let r2 = Rpx256::hash(&[1, 2, 3, 4, 5, 6, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpx256::hash(&[1, 2, 3, 4, 5, 6, 7]);
		let r2 = Rpx256::hash(&[1, 2, 3, 4, 5, 6, 7, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpx256::hash(&[1, 2, 3, 4, 5, 6, 7, 0, 0]);
		let r2 = Rpx256::hash(&[1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0]);
		assert_ne!(r1, r2);
	}

	#[test]
	fn hash_elements_padding() {
		let elements = core::array::from_fn::<Felt, 8, _>(|i| Felt::new(i as u64));

		let digests = [
			RpxDigest::new(elements[..4].try_into().unwrap()),
			RpxDigest::new(elements[4..].try_into().unwrap()),
		];

		let m_result = Rpx256::merge(&digests);
		let h_result = Rpx256::hash_elements(&elements);

		assert_eq!(m_result, h_result);
	}

	#[test]
	fn hash_empty() {
		let elements: [Felt; 0] = [];

		let zero_digest = RpxDigest::default();
		let h_result = Rpx256::hash_elements(&elements);
		assert_eq!(h_result, zero_digest);
	}

	#[test]
	fn hash_empty_bytes() {
		let zero_digest = RpxDigest::default();
		let h_result = Rpx256::hash(&[]);

		assert_eq!(h_result, zero_digest);
	}

	#[test]
	fn sponge_bytes_with_remainder_length_wont_panic() {
		_ = Rpx256::hash(&[0; 113]);
	}

	#[test]
	fn sponge_collision_for_wrapped_field_element() {
		let a = Rpx256::hash(&[0; 8]);
		let b = Rpx256::hash(&Felt::MODULUS.to_le_bytes());
		assert_ne!(a, b);
	}

	#[test]
	fn sponge_zero_collision() {
		let mut zeroes = Vec::with_capacity(255);
		let mut set = BTreeSet::new();
		(0..255).for_each(|_| {
			let hash = Rpx256::hash(&zeroes);
			zeroes.push(0);
			assert!(set.insert(hash));
		});
	}

	proptest! {
		#[test]
		fn rpx256_wont_panic_with_arbitrary_input(ref bytes in any::<Vec<u8>>()) {
			_ = Rpx256::hash(bytes);
		}
	}
}
