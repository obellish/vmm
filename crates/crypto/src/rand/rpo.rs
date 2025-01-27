use alloc::{string::String, vec::Vec};

use rand::RngCore;
use rand_core::impls;

use super::{FeltRng, RandomCoin, RandomCoinError};
use crate::{
	Felt, FieldElement, Word, ZERO,
	hash::rpo::{Rpo256, RpoDigest},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

const STATE_WIDTH: usize = Rpo256::STATE_WIDTH;
const RATE_START: usize = Rpo256::RATE_RANGE.start;
const RATE_END: usize = Rpo256::RATE_RANGE.end;
const HALF_RATE_WIDTH: usize = (RATE_END - RATE_START) / 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpoRandomCoin {
	state: [Felt; STATE_WIDTH],
	current: usize,
}

impl RpoRandomCoin {
	#[must_use]
	pub fn new(seed: Word) -> Self {
		let mut state = [ZERO; STATE_WIDTH];

		for i in 0..HALF_RATE_WIDTH {
			state[RATE_START + i] += seed[i];
		}

		Rpo256::apply_permutation(&mut state);

		Self {
			state,
			current: RATE_START,
		}
	}

	#[must_use]
	pub fn from_parts(state: [Felt; STATE_WIDTH], current: usize) -> Self {
		assert!(
			(RATE_START..RATE_END).contains(&current),
			"current value outside of valid range"
		);

		Self { state, current }
	}

	#[must_use]
	pub const fn into_parts(self) -> ([Felt; STATE_WIDTH], usize) {
		(self.state, self.current)
	}

	pub fn fill_bytes(&mut self, dest: &mut [u8]) {
		<Self as RngCore>::fill_bytes(self, dest);
	}

	fn draw_basefield(&mut self) -> Felt {
		if self.current == RATE_END {
			Rpo256::apply_permutation(&mut self.state);
			self.current = RATE_START;
		}

		self.current += 1;
		self.state[self.current - 1]
	}
}

impl Deserializable for RpoRandomCoin {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let state = [
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
			Felt::read_from(source)?,
		];
		let current = source.read_u8()? as usize;
		if !(RATE_START..RATE_END).contains(&current) {
			return Err(DeserializationError::InvalidValue(String::from(
				"current value outside of valid range",
			)));
		}

		Ok(Self { state, current })
	}
}

impl FeltRng for RpoRandomCoin {
	fn draw_element(&mut self) -> Felt {
		self.draw_basefield()
	}
}

impl RandomCoin for RpoRandomCoin {
	type BaseField = Felt;
	type Hasher = Rpo256;

	fn new(seed: &[Self::BaseField]) -> Self {
		let digest: Word = Rpo256::hash_elements(seed).into();
		Self::new(digest)
	}

	fn reseed(&mut self, data: RpoDigest) {
		self.current = RATE_START;

		let data: Word = data.into();

		self.state[RATE_START] += data[0];
		self.state[RATE_START + 1] += data[1];
		self.state[RATE_START + 2] += data[2];
		self.state[RATE_START + 3] += data[3];

		Rpo256::apply_permutation(&mut self.state);
	}

	fn check_leading_zeros(&self, value: u64) -> u32 {
		let value = Felt::new(value);
		let mut state_tmp = self.state;

		state_tmp[RATE_START] += value;

		Rpo256::apply_permutation(&mut state_tmp);

		let first_rate_element = state_tmp[RATE_START].as_int();
		first_rate_element.trailing_zeros()
	}

	fn draw<E>(&mut self) -> Result<E, RandomCoinError>
	where
		E: FieldElement<BaseField = Self::BaseField>,
	{
		let ext_degree = E::EXTENSION_DEGREE;
		let mut result = vec![ZERO; ext_degree];
		for r in result.iter_mut().take(ext_degree) {
			*r = self.draw_basefield();
		}

		let result = E::slice_from_base_elements(&result);
		Ok(result[0])
	}

	fn draw_integers(
		&mut self,
		num_values: usize,
		domain_size: usize,
		nonce: u64,
	) -> Result<Vec<usize>, RandomCoinError> {
		assert!(
			domain_size.is_power_of_two(),
			"domain size must be a power of two"
		);
		assert!(
			num_values < domain_size,
			"number of values must be smaller than the domain size"
		);

		let nonce = Felt::new(nonce);
		self.state[RATE_START] += nonce;
		Rpo256::apply_permutation(&mut self.state);

		self.current = RATE_START + 1;

		let v_mask = (domain_size - 1) as u64;

		let mut values = Vec::new();
		for _ in 0..1000 {
			let value = self.draw_basefield().as_int();
			let value = (value & v_mask) as usize;

			values.push(value);
			if values.len() == num_values {
				break;
			}
		}

		if values.len() < num_values {
			return Err(RandomCoinError::FailedToDrawIntegers(
				num_values,
				values.len(),
				1000,
			));
		}

		Ok(values)
	}
}

impl RngCore for RpoRandomCoin {
	fn next_u32(&mut self) -> u32 {
		self.draw_basefield().as_int() as u32
	}

	fn next_u64(&mut self) -> u64 {
		impls::next_u64_via_u32(self)
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		impls::fill_bytes_via_next(self, dest);
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
		self.fill_bytes(dest);
		Ok(())
	}
}

impl Serializable for RpoRandomCoin {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.state.iter().for_each(|v| v.write_into(target));
		target.write_u8(self.current as u8);
	}
}

#[cfg(test)]
mod tests {
	use super::{Deserializable, DeserializationError, FeltRng, RpoRandomCoin, Serializable, ZERO};
	use crate::ONE;

	#[test]
	fn feltrng_felt() {
		let mut rpocoin = RpoRandomCoin::new([ZERO; 4]);
		let output = rpocoin.draw_element();

		let mut rpocoin = RpoRandomCoin::new([ZERO; 4]);
		let expected = rpocoin.draw_basefield();

		assert_eq!(output, expected);
	}

	#[test]
	fn feltrng_word() {
		let mut rpocoin = RpoRandomCoin::new([ZERO; 4]);
		let output = rpocoin.draw_word();

		let mut rpocoin = RpoRandomCoin::new([ZERO; 4]);
		let mut expected = [ZERO; 4];
		for o in &mut expected {
			*o = rpocoin.draw_basefield();
		}

		assert_eq!(output, expected);
	}

	#[test]
	fn feltrng_serialization() -> Result<(), DeserializationError> {
		let coin1 = RpoRandomCoin::from_parts([ONE; 12], 5);

		let bytes = coin1.to_bytes();
		let coin2 = RpoRandomCoin::read_from_bytes(&bytes)?;

		assert_eq!(coin1, coin2);

		Ok(())
	}
}
