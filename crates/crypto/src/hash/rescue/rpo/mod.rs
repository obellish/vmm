mod digest;

use core::ops::Range;

use winter_math::StarkField;

pub use self::digest::{RpoDigest, RpoDigestError};
use super::{
	ARK1, ARK2, BINARY_CHUNK_SIZE, CAPACITY_RANGE, DIGEST_RANGE, INPUT1_RANGE, INPUT2_RANGE,
	NUM_ROUNDS, RATE_RANGE, RATE_WIDTH, STATE_WIDTH, add_constants,
	add_constants_and_apply_inv_sbox, add_constants_and_apply_sbox, apply_inv_sbox, apply_sbox,
	mds::{MDS, apply_mds},
};
use crate::{
	Felt, FieldElement, ZERO,
	hash::{ElementHasher, Hasher},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rpo256;

#[allow(clippy::trivially_copy_pass_by_ref)]
impl Rpo256 {
	pub const ARK1: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = ARK1;
	pub const ARK2: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = ARK2;
	pub const CAPACITY_RANGE: Range<usize> = CAPACITY_RANGE;
	pub const DIGEST_RANGE: Range<usize> = DIGEST_RANGE;
	pub const MDS: [[Felt; STATE_WIDTH]; STATE_WIDTH] = MDS;
	pub const NUM_ROUNDS: usize = NUM_ROUNDS;
	pub const RATE_RANGE: Range<usize> = RATE_RANGE;
	pub const STATE_WIDTH: usize = STATE_WIDTH;

	#[must_use]
	pub fn hash(bytes: &[u8]) -> RpoDigest {
		<Self as Hasher>::hash(bytes)
	}

	#[must_use]
	pub fn merge(values: &[RpoDigest; 2]) -> RpoDigest {
		<Self as Hasher>::merge(values)
	}

	pub fn hash_elements<E>(elements: &[E]) -> RpoDigest
	where
		E: FieldElement<BaseField = Felt>,
	{
		<Self as ElementHasher>::hash_elements(elements)
	}

	#[must_use]
	pub fn merge_in_domain(values: &[RpoDigest; 2], domain: Felt) -> RpoDigest {
		let mut state = [ZERO; STATE_WIDTH];
		let it = RpoDigest::digests_as_elements_iter(values.iter());
		for (i, v) in it.copied().enumerate() {
			state[RATE_RANGE.start + i] = v;
		}

		state[CAPACITY_RANGE.start + 1] = domain;

		Self::apply_permutation(&mut state);
		RpoDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	pub fn apply_permutation(state: &mut [Felt; STATE_WIDTH]) {
		for i in 0..NUM_ROUNDS {
			Self::apply_round(state, i);
		}
	}

	pub fn apply_round(state: &mut [Felt; STATE_WIDTH], round: usize) {
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
}

impl ElementHasher for Rpo256 {
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

		RpoDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}
}

impl Hasher for Rpo256 {
	type Digest = RpoDigest;

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

		RpoDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
		let mut state = [ZERO; STATE_WIDTH];
		let it = RpoDigest::digests_as_elements_iter(values.iter());
		for (i, v) in it.copied().enumerate() {
			state[RATE_RANGE.start + i] = v;
		}

		Self::apply_permutation(&mut state);
		RpoDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}

	fn merge_many(values: &[Self::Digest]) -> Self::Digest {
		Self::hash_elements(RpoDigest::digests_as_elements(values))
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
		RpoDigest::new(state[DIGEST_RANGE].try_into().unwrap())
	}
}

#[cfg(test)]
mod tests {
	use alloc::{collections::BTreeSet, vec::Vec};

	use proptest::prelude::*;
	use rand_utils::rand_value;

	use super::{
		super::{ALPHA, INV_ALPHA, apply_inv_sbox, apply_sbox},
		Felt, FieldElement, Hasher, Rpo256, RpoDigest, STATE_WIDTH, StarkField, ZERO,
	};
	use crate::{
		ONE, Word,
		hash::rescue::{BINARY_CHUNK_SIZE, CAPACITY_RANGE, DIGEST_RANGE, RATE_WIDTH},
	};

	const EXPECTED: [Word; 19] = [
		[
			Felt::new(18_126_731_724_905_382_595),
			Felt::new(7_388_557_040_857_728_717),
			Felt::new(14_290_750_514_634_285_295),
			Felt::new(7_852_282_086_160_480_146),
		],
		[
			Felt::new(10_139_303_045_932_500_183),
			Felt::new(2_293_916_558_361_785_533),
			Felt::new(15_496_361_415_980_502_047),
			Felt::new(17_904_948_502_382_283_940),
		],
		[
			Felt::new(17_457_546_260_239_634_015),
			Felt::new(803_990_662_839_494_686),
			Felt::new(10_386_005_777_401_424_878),
			Felt::new(18_168_807_883_298_448_638),
		],
		[
			Felt::new(13_072_499_238_647_455_740),
			Felt::new(10_174_350_003_422_057_273),
			Felt::new(9_201_651_627_651_151_113),
			Felt::new(6_872_461_887_313_298_746),
		],
		[
			Felt::new(2_903_803_350_580_990_546),
			Felt::new(1_838_870_750_730_563_299),
			Felt::new(4_258_619_137_315_479_708),
			Felt::new(17_334_260_395_129_062_936),
		],
		[
			Felt::new(8_571_221_005_243_425_262),
			Felt::new(3_016_595_589_318_175_865),
			Felt::new(13_933_674_291_329_928_438),
			Felt::new(678_640_375_034_313_072),
		],
		[
			Felt::new(16_314_113_978_986_502_310),
			Felt::new(14_587_622_368_743_051_587),
			Felt::new(2_808_708_361_436_818_462),
			Felt::new(10_660_517_522_478_329_440),
		],
		[
			Felt::new(2_242_391_899_857_912_644),
			Felt::new(12_689_382_052_053_305_418),
			Felt::new(235_236_990_017_815_546),
			Felt::new(5_046_143_039_268_215_739),
		],
		[
			Felt::new(5_218_076_004_221_736_204),
			Felt::new(17_169_400_568_680_971_304),
			Felt::new(8_840_075_572_473_868_990),
			Felt::new(12_382_372_614_369_863_623),
		],
		[
			Felt::new(9_783_834_557_155_203_486),
			Felt::new(12_317_263_104_955_018_849),
			Felt::new(3_933_748_931_816_109_604),
			Felt::new(1_843_043_029_836_917_214),
		],
		[
			Felt::new(14_498_234_468_286_984_551),
			Felt::new(16_837_257_669_834_682_387),
			Felt::new(6_664_141_123_711_355_107),
			Felt::new(4_590_460_158_294_697_186),
		],
		[
			Felt::new(4_661_800_562_479_916_067),
			Felt::new(11_794_407_552_792_839_953),
			Felt::new(9_037_742_258_721_863_712),
			Felt::new(6_287_820_818_064_278_819),
		],
		[
			Felt::new(7_752_693_085_194_633_729),
			Felt::new(7_379_857_372_245_835_536),
			Felt::new(9_270_229_380_648_024_178),
			Felt::new(10_638_301_488_452_560_378),
		],
		[
			Felt::new(11_542_686_762_698_783_357),
			Felt::new(15_570_714_990_728_449_027),
			Felt::new(7_518_801_014_067_819_501),
			Felt::new(12_706_437_751_337_583_515),
		],
		[
			Felt::new(9_553_923_701_032_839_042),
			Felt::new(7_281_190_920_209_838_818),
			Felt::new(2_488_477_917_448_393_955),
			Felt::new(5_088_955_350_303_368_837),
		],
		[
			Felt::new(4_935_426_252_518_736_883),
			Felt::new(12_584_230_452_580_950_419),
			Felt::new(8_762_518_969_632_303_998),
			Felt::new(18_159_875_708_229_758_073),
		],
		[
			Felt::new(12_795_429_638_314_178_838),
			Felt::new(14_360_248_269_767_567_855),
			Felt::new(3_819_563_852_436_765_058),
			Felt::new(10_859_123_583_999_067_291),
		],
		[
			Felt::new(2_695_742_617_679_420_093),
			Felt::new(9_151_515_850_666_059_759),
			Felt::new(15_855_828_029_180_595_485),
			Felt::new(17_190_029_785_471_463_210),
		],
		[
			Felt::new(13_205_273_108_219_124_830),
			Felt::new(2_524_898_486_192_849_221),
			Felt::new(14_618_764_355_375_283_547),
			Felt::new(10_615_614_265_042_186_874),
		],
	];

	fn get_random_array<const N: usize>() -> [Felt; N] {
		core::array::from_fn(|_| Felt::new(rand_value()))
	}

	#[test]
	fn sbox() {
		let state = get_random_array();

		let mut expected = state;
		expected.iter_mut().for_each(|v| *v = v.exp(ALPHA));

		let mut actual = state;
		apply_sbox(&mut actual);

		assert_eq!(expected, actual);
	}

	#[test]
	fn inv_sbox() {
		let state: [Felt; STATE_WIDTH] = get_random_array();

		let mut expected = state;
		expected.iter_mut().for_each(|v| *v = v.exp(INV_ALPHA));

		let mut actual = state;
		apply_inv_sbox(&mut actual);

		assert_eq!(expected, actual);
	}

	#[test]
	fn hash_elements_vs_merge() {
		let elements = get_random_array::<8>();

		let digests = [
			RpoDigest::new(elements[..4].try_into().unwrap()),
			RpoDigest::new(elements[4..].try_into().unwrap()),
		];

		let m_result = Rpo256::merge(&digests);
		let h_result = Rpo256::hash_elements(&elements);
		assert_eq!(m_result, h_result);
	}

	#[test]
	fn merge_vs_merge_in_domain() {
		let elements = get_random_array::<8>();

		let digests = [
			RpoDigest::new(elements[..4].try_into().unwrap()),
			RpoDigest::new(elements[4..].try_into().unwrap()),
		];
		let merge_result = Rpo256::merge(&digests);

		let domain = ZERO;

		let merge_in_domain_result = Rpo256::merge_in_domain(&digests, domain);
		assert_eq!(merge_result, merge_in_domain_result);

		let domain = ONE;

		let merge_in_domain_result = Rpo256::merge_in_domain(&digests, domain);
		assert_ne!(merge_result, merge_in_domain_result);
	}

	#[test]
	fn hash_elements_vs_merge_with_int() {
		let tmp = get_random_array();
		let seed = RpoDigest::new(tmp);

		let val = Felt::new(rand_value());
		let m_result = Rpo256::merge_with_int(seed, val.as_int());

		let mut elements = seed.as_elements().to_vec();
		elements.push(val);
		let h_result = Rpo256::hash_elements(&elements);

		assert_eq!(m_result, h_result);

		let val = Felt::MODULUS + 2;
		let m_result = Rpo256::merge_with_int(seed, val);

		let mut elements = seed.as_elements().to_vec();
		elements.extend([Felt::new(val), ONE]);
		let h_result = Rpo256::hash_elements(&elements);

		assert_eq!(m_result, h_result);
	}

	#[test]
	fn hash_padding() {
		let r1 = Rpo256::hash(&[1, 2, 3]);
		let r2 = Rpo256::hash(&[1, 2, 3, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpo256::hash(&[1, 2, 3, 4, 5, 6]);
		let r2 = Rpo256::hash(&[1, 2, 3, 4, 5, 6, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpo256::hash(&[1, 2, 3, 4, 5, 6, 7]);
		let r2 = Rpo256::hash(&[1, 2, 3, 4, 5, 6, 7, 0]);
		assert_ne!(r1, r2);

		let r1 = Rpo256::hash(&[1, 2, 3, 4, 5, 6, 7, 0, 0]);
		let r2 = Rpo256::hash(&[1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0]);
		assert_ne!(r1, r2);
	}

	#[test]
	fn hash_padding_no_extra_permutation_call() {
		let num_bytes = BINARY_CHUNK_SIZE * RATE_WIDTH;
		let mut buffer = vec![0u8; num_bytes];
		*buffer.last_mut().unwrap() = 97;
		let r1 = Rpo256::hash(&buffer);

		let final_chunk = [0u8, 0, 0, 0, 0, 0, 97, 1];
		let mut state = [ZERO; STATE_WIDTH];

		state[CAPACITY_RANGE.start] = Felt::from(RATE_WIDTH as u8);
		*state.last_mut().unwrap() = Felt::new(u64::from_le_bytes(final_chunk));
		Rpo256::apply_permutation(&mut state);

		assert_eq!(&r1[0..4], &state[DIGEST_RANGE]);
	}

	#[test]
	fn hash_elements_padding() {
		let e1 = get_random_array::<2>();
		let e2 = [e1[0], e1[1], ZERO];

		let r1 = Rpo256::hash_elements(&e1);
		let r2 = Rpo256::hash_elements(&e2);

		assert_ne!(r1, r2);
	}

	#[test]
	fn hash_elements() {
		let elements = core::array::from_fn::<Felt, 8, _>(|i| Felt::new(i as u64));

		let digests = [
			RpoDigest::new(elements[..4].try_into().unwrap()),
			RpoDigest::new(elements[4..].try_into().unwrap()),
		];

		let m_result = Rpo256::merge(&digests);
		let h_result = Rpo256::hash_elements(&elements);

		assert_eq!(m_result, h_result);
	}

	#[test]
	fn hash_empty() {
		let elements: [Felt; 0] = [];

		let zero_digest = RpoDigest::default();
		let h_result = Rpo256::hash_elements(&elements);
		assert_eq!(h_result, zero_digest);
	}

	#[test]
	fn hash_empty_bytes() {
		let zero_digest = RpoDigest::default();
		let h_result = Rpo256::hash(&[]);

		assert_eq!(h_result, zero_digest);
	}

	#[test]
	fn hash_test_vectors() {
		let elements = core::array::from_fn::<Felt, 19, _>(|i| Felt::new(i as u64));

		for i in 0..elements.len() {
			let expected = RpoDigest::new(EXPECTED[i]);
			let result = Rpo256::hash_elements(&elements[..=i]);
			assert_eq!(result, expected);
		}
	}

	#[test]
	fn sponge_bytes_with_remainder_length_wont_panic() {
		_ = Rpo256::hash(&[0; 113]);
	}

	#[test]
	fn sponge_collision_for_wrapped_field_element() {
		let a = Rpo256::hash(&[0; 8]);
		let b = Rpo256::hash(&Felt::MODULUS.to_le_bytes());
		assert_ne!(a, b);
	}

	#[test]
	fn sponge_zero_collision() {
		let mut zeroes = Vec::with_capacity(255);
		let mut set = BTreeSet::new();
		(0..255).for_each(|_| {
			let hash = Rpo256::hash(&zeroes);
			zeroes.push(0);
			assert!(set.insert(hash));
		});
	}

	proptest! {
		#[test]
		fn rpo256_wont_panic_with_arbitrary_input(ref bytes in any::<Vec<u8>>()) {
			_ = Rpo256::hash(bytes);
		}
	}
}
