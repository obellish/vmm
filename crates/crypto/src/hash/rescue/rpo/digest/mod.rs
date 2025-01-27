#[cfg(feature = "serde")]
mod serde;

use alloc::string::String;
use core::{
	cmp::Ordering,
	fmt::{Display, Formatter, Result as FmtResult},
	ops::Deref,
	slice,
};

use thiserror::Error;
use winter_crypto::Digest;

use crate::{
	Felt, StarkField, ZERO,
	hash::rescue::{DIGEST_BYTES, DIGEST_SIZE},
	rand::Randomizable,
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, HexParseError, Serializable,
		bytes_to_hex_string, hex_to_bytes,
	},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RpoDigest([Felt; DIGEST_SIZE]);

#[allow(clippy::trivially_copy_pass_by_ref)]
impl RpoDigest {
	pub const SERIALIZED_SIZE: usize = DIGEST_BYTES;

	#[must_use]
	pub const fn new(value: [Felt; DIGEST_SIZE]) -> Self {
		Self(value)
	}

	#[must_use]
	pub fn as_elements(&self) -> &[Felt] {
		self.as_ref()
	}

	#[must_use]
	pub fn as_bytes(&self) -> [u8; DIGEST_BYTES] {
		<Self as Digest>::as_bytes(self)
	}

	pub fn digests_as_elements_iter<'a, I>(digests: I) -> impl Iterator<Item = &'a Felt>
	where
		I: Iterator<Item = &'a Self>,
	{
		digests.flat_map(|d| d.0.iter())
	}

	#[must_use]
	pub const fn digests_as_elements(digests: &[Self]) -> &[Felt] {
		let p = digests.as_ptr();
		let len = digests.len() * DIGEST_SIZE;
		unsafe { slice::from_raw_parts(p.cast::<Felt>(), len) }
	}

	#[must_use]
	pub fn to_hex(self) -> String {
		bytes_to_hex_string(self.as_bytes())
	}
}

impl Deref for RpoDigest {
	type Target = [Felt; DIGEST_SIZE];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Deserializable for RpoDigest {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let mut inner: [Felt; DIGEST_SIZE] = [ZERO; DIGEST_SIZE];
		for inner in &mut inner {
			let e = source.read_u64()?;
			if e >= Felt::MODULUS {
				return Err(DeserializationError::InvalidValue(String::from(
					"value not in the appropriate range",
				)));
			}
			*inner = Felt::new(e);
		}

		Ok(Self(inner))
	}
}

impl Digest for RpoDigest {
	fn as_bytes(&self) -> [u8; DIGEST_BYTES] {
		let mut result = [0; DIGEST_BYTES];

		result[..=7].copy_from_slice(&self.0[0].as_int().to_le_bytes());
		result[8..=15].copy_from_slice(&self.0[1].as_int().to_le_bytes());
		result[16..=23].copy_from_slice(&self.0[2].as_int().to_le_bytes());
		result[24..].copy_from_slice(&self.0[3].as_int().to_le_bytes());

		result
	}
}

impl Display for RpoDigest {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let encoded: String = self.into();
		f.write_str(&encoded)
	}
}

impl From<[bool; DIGEST_SIZE]> for RpoDigest {
	fn from(value: [bool; DIGEST_SIZE]) -> Self {
		value.map(u32::from).into()
	}
}

impl From<&[bool; DIGEST_SIZE]> for RpoDigest {
	fn from(value: &[bool; DIGEST_SIZE]) -> Self {
		(*value).into()
	}
}

impl From<[u8; DIGEST_SIZE]> for RpoDigest {
	fn from(value: [u8; DIGEST_SIZE]) -> Self {
		Self(value.map(Into::into))
	}
}

impl From<&[u8; DIGEST_SIZE]> for RpoDigest {
	fn from(value: &[u8; DIGEST_SIZE]) -> Self {
		(*value).into()
	}
}

impl From<[u16; DIGEST_SIZE]> for RpoDigest {
	fn from(value: [u16; DIGEST_SIZE]) -> Self {
		value.map(u32::from).into()
	}
}

impl From<&[u16; DIGEST_SIZE]> for RpoDigest {
	fn from(value: &[u16; DIGEST_SIZE]) -> Self {
		(*value).into()
	}
}

impl From<[u32; DIGEST_SIZE]> for RpoDigest {
	fn from(value: [u32; DIGEST_SIZE]) -> Self {
		Self(value.map(Into::into))
	}
}

impl From<&[u32; DIGEST_SIZE]> for RpoDigest {
	fn from(value: &[u32; DIGEST_SIZE]) -> Self {
		(*value).into()
	}
}

impl From<[Felt; DIGEST_SIZE]> for RpoDigest {
	fn from(value: [Felt; DIGEST_SIZE]) -> Self {
		Self::new(value)
	}
}

impl From<&[Felt; DIGEST_SIZE]> for RpoDigest {
	fn from(value: &[Felt; DIGEST_SIZE]) -> Self {
		(*value).into()
	}
}

impl From<RpoDigest> for [u64; DIGEST_SIZE] {
	fn from(value: RpoDigest) -> Self {
		value.0.map(|v| v.as_int())
	}
}

impl From<&RpoDigest> for [u64; DIGEST_SIZE] {
	fn from(value: &RpoDigest) -> Self {
		(*value).into()
	}
}

impl From<RpoDigest> for [Felt; DIGEST_SIZE] {
	fn from(value: RpoDigest) -> Self {
		value.0
	}
}

impl From<&RpoDigest> for [Felt; DIGEST_SIZE] {
	fn from(value: &RpoDigest) -> Self {
		(*value).into()
	}
}

impl From<RpoDigest> for [u8; DIGEST_BYTES] {
	fn from(value: RpoDigest) -> Self {
		value.as_bytes()
	}
}

impl From<&RpoDigest> for [u8; DIGEST_BYTES] {
	fn from(value: &RpoDigest) -> Self {
		(*value).into()
	}
}

impl From<RpoDigest> for String {
	fn from(value: RpoDigest) -> Self {
		value.to_hex()
	}
}

impl From<&RpoDigest> for String {
	fn from(value: &RpoDigest) -> Self {
		(*value).into()
	}
}

impl IntoIterator for RpoDigest {
	type IntoIter = <[Felt; DIGEST_SIZE] as IntoIterator>::IntoIter;
	type Item = Felt;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl Ord for RpoDigest {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0
			.iter()
			.map(Felt::inner)
			.zip(other.0.iter().map(Felt::inner))
			.fold(Ordering::Equal, |ord, (a, b)| match ord {
				Ordering::Equal => a.cmp(&b),
				_ => ord,
			})
	}
}

impl PartialOrd for RpoDigest {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Randomizable for RpoDigest {
	const VALUE_SIZE: usize = DIGEST_BYTES;

	fn from_random_bytes(source: &[u8]) -> Option<Self> {
		let bytes_array: [u8; 32] = source.try_into().ok()?;
		Self::try_from(bytes_array).ok()
	}
}

impl Serializable for RpoDigest {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_bytes(&self.as_bytes());
	}

	fn get_size_hint(&self) -> usize {
		Self::SERIALIZED_SIZE
	}
}

impl TryFrom<[u8; DIGEST_BYTES]> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: [u8; DIGEST_BYTES]) -> Result<Self, Self::Error> {
		let a = u64::from_le_bytes(value[0..=7].try_into().unwrap());
		let b = u64::from_le_bytes(value[8..=15].try_into().unwrap());
		let c = u64::from_le_bytes(value[16..=23].try_into().unwrap());
		let d = u64::from_le_bytes(value[24..=31].try_into().unwrap());

		if [a, b, c, d].iter().any(|v| *v >= Felt::MODULUS) {
			return Err(HexParseError::OutOfRange);
		}

		Ok(Self([
			Felt::new(a),
			Felt::new(b),
			Felt::new(c),
			Felt::new(d),
		]))
	}
}

impl TryFrom<&[u8; DIGEST_BYTES]> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: &[u8; DIGEST_BYTES]) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<[u64; DIGEST_SIZE]> for RpoDigest {
	type Error = RpoDigestError;

	fn try_from(value: [u64; DIGEST_SIZE]) -> Result<Self, Self::Error> {
		Ok(Self([
			value[0]
				.try_into()
				.map_err(RpoDigestError::InvalidFieldElement)?,
			value[1]
				.try_into()
				.map_err(RpoDigestError::InvalidFieldElement)?,
			value[2]
				.try_into()
				.map_err(RpoDigestError::InvalidFieldElement)?,
			value[3]
				.try_into()
				.map_err(RpoDigestError::InvalidFieldElement)?,
		]))
	}
}

impl TryFrom<&[u64; DIGEST_SIZE]> for RpoDigest {
	type Error = RpoDigestError;

	fn try_from(value: &[u64; DIGEST_SIZE]) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<&[u8]> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<&str> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		hex_to_bytes::<DIGEST_BYTES>(value).and_then(Self::try_from)
	}
}

impl TryFrom<String> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		value.as_str().try_into()
	}
}

impl TryFrom<&String> for RpoDigest {
	type Error = HexParseError;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		value.as_str().try_into()
	}
}

impl TryFrom<RpoDigest> for [bool; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: RpoDigest) -> Result<Self, Self::Error> {
		fn to_bool(v: u64) -> Option<bool> {
			(v <= 1).then_some(matches!(v, 1))
		}

		Ok([
			to_bool(value.0[0].as_int()).ok_or(RpoDigestError::TypeConversion("bool"))?,
			to_bool(value.0[1].as_int()).ok_or(RpoDigestError::TypeConversion("bool"))?,
			to_bool(value.0[2].as_int()).ok_or(RpoDigestError::TypeConversion("bool"))?,
			to_bool(value.0[3].as_int()).ok_or(RpoDigestError::TypeConversion("bool"))?,
		])
	}
}

impl TryFrom<&RpoDigest> for [bool; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: &RpoDigest) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<RpoDigest> for [u8; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: RpoDigest) -> Result<Self, Self::Error> {
		Ok([
			value.0[0]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u8"))?,
			value.0[1]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u8"))?,
			value.0[2]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u8"))?,
			value.0[3]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u8"))?,
		])
	}
}

impl TryFrom<&RpoDigest> for [u8; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: &RpoDigest) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<RpoDigest> for [u16; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: RpoDigest) -> Result<Self, Self::Error> {
		Ok([
			value.0[0]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u16"))?,
			value.0[1]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u16"))?,
			value.0[2]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u16"))?,
			value.0[3]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u16"))?,
		])
	}
}

impl TryFrom<&RpoDigest> for [u16; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: &RpoDigest) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

impl TryFrom<RpoDigest> for [u32; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: RpoDigest) -> Result<Self, Self::Error> {
		Ok([
			value.0[0]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u32"))?,
			value.0[1]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u32"))?,
			value.0[2]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u32"))?,
			value.0[3]
				.as_int()
				.try_into()
				.map_err(|_| RpoDigestError::TypeConversion("u32"))?,
		])
	}
}

impl TryFrom<&RpoDigest> for [u32; DIGEST_SIZE] {
	type Error = RpoDigestError;

	fn try_from(value: &RpoDigest) -> Result<Self, Self::Error> {
		(*value).try_into()
	}
}

#[derive(Debug, Error)]
pub enum RpoDigestError {
	#[error("failed to convert digest field element to {0}")]
	TypeConversion(&'static str),
	#[error("failed to convert to field element: {0}")]
	InvalidFieldElement(String),
}

#[cfg(test)]
mod tests {
	use alloc::{string::String, vec::Vec};

	use rand_utils::rand_value;

	use super::{
		DIGEST_BYTES, DIGEST_SIZE, Deserializable, DeserializationError, Felt, HexParseError,
		RpoDigest, Serializable,
	};
	use crate::utils::SliceReader;

	fn get_random_digest() -> RpoDigest {
		RpoDigest::new(core::array::from_fn(|_| Felt::new(rand_value())))
	}

	#[test]
	fn digest_serialization() -> Result<(), DeserializationError> {
		let e1 = Felt::new(rand_value());
		let e2 = Felt::new(rand_value());
		let e3 = Felt::new(rand_value());
		let e4 = Felt::new(rand_value());

		let d1 = RpoDigest::new([e1, e2, e3, e4]);

		let mut bytes = Vec::new();
		d1.write_into(&mut bytes);
		assert_eq!(bytes.len(), DIGEST_BYTES);
		assert_eq!(bytes.len(), d1.get_size_hint());

		let mut reader = SliceReader::new(&bytes);
		let d2 = RpoDigest::read_from(&mut reader)?;

		assert_eq!(d1, d2);

		Ok(())
	}

	#[test]
	fn digest_encoding() -> Result<(), HexParseError> {
		let digest = get_random_digest();

		let string: String = digest.into();
		let round_trip = RpoDigest::try_from(string)?;

		assert_eq!(digest, round_trip);

		Ok(())
	}

	#[test]
	fn conversions() -> eyre::Result<()> {
		let digest = get_random_digest();

		let v: [bool; DIGEST_SIZE] = [true, false, true, true];
		let v2: RpoDigest = v.into();
		assert_eq!(v, <[bool; DIGEST_SIZE]>::try_from(v2)?);

		let v: [u8; DIGEST_SIZE] = [0, 1, 2, 3];
		let v2: RpoDigest = v.into();
		assert_eq!(v, <[u8; DIGEST_SIZE]>::try_from(v2)?);

		let v: [u16; DIGEST_SIZE] = [0, 1, 2, 3];
		let v2: RpoDigest = v.into();
		assert_eq!(v, <[u16; DIGEST_SIZE]>::try_from(v2)?);

		let v: [u32; DIGEST_SIZE] = [0, 1, 2, 3];
		let v2: RpoDigest = v.into();
		assert_eq!(v, <[u32; DIGEST_SIZE]>::try_from(v2)?);

		let v: [u64; DIGEST_SIZE] = digest.into();
		let v2: RpoDigest = v.try_into()?;
		assert_eq!(digest, v2);

		let v: [Felt; DIGEST_SIZE] = digest.into();
		let v2: RpoDigest = v.into();
		assert_eq!(digest, v2);

		let v: [u8; DIGEST_BYTES] = digest.into();
		let v2: RpoDigest = v.try_into()?;
		assert_eq!(digest, v2);

		let v: String = digest.into();
		let v2: RpoDigest = v.try_into()?;
		assert_eq!(digest, v2);

		let v: [bool; DIGEST_SIZE] = [true, false, true, true];
		let v2: RpoDigest = (&v).into();
		assert_eq!(v, <[bool; DIGEST_SIZE]>::try_from(&v2)?);

		let v: [u8; DIGEST_SIZE] = [0_u8, 1_u8, 2_u8, 3_u8];
		let v2: RpoDigest = (&v).into();
		assert_eq!(v, <[u8; DIGEST_SIZE]>::try_from(&v2)?);

		let v: [u16; DIGEST_SIZE] = [0_u16, 1_u16, 2_u16, 3_u16];
		let v2: RpoDigest = (&v).into();
		assert_eq!(v, <[u16; DIGEST_SIZE]>::try_from(&v2)?);

		let v: [u32; DIGEST_SIZE] = [0_u32, 1_u32, 2_u32, 3_u32];
		let v2: RpoDigest = (&v).into();
		assert_eq!(v, <[u32; DIGEST_SIZE]>::try_from(&v2)?);

		let v: [u64; DIGEST_SIZE] = (&digest).into();
		let v2: RpoDigest = (&v).try_into()?;
		assert_eq!(digest, v2);

		let v: [Felt; DIGEST_SIZE] = (&digest).into();
		let v2: RpoDigest = (&v).into();
		assert_eq!(digest, v2);

		let v: [u8; DIGEST_BYTES] = (&digest).into();
		let v2: RpoDigest = (&v).try_into()?;
		assert_eq!(digest, v2);

		let v: String = (&digest).into();
		let v2: RpoDigest = (&v).try_into()?;
		assert_eq!(digest, v2);

		Ok(())
	}
}
