use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::{Read, Write},
	ops::{Deref, DerefMut},
};

use byteorder::ReadBytesExt as _;
use serde::{Deserialize, Serialize};

use super::{Decode, Encode, ProtocolError};

#[derive(
	Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(transparent)]
#[serde(transparent)]
pub struct VarInt(pub i32);

impl VarInt {
	pub const MAX_SIZE: usize = 5;

	pub const fn written_size(self) -> usize {
		match self.0 {
			0 => 1,
			n => (31 - n.leading_zeros() as usize) / 7 + 1,
		}
	}

	pub fn decode_partial(mut r: impl Read) -> Result<i32, VarIntDecodeError> {
		let mut val = 0;
		for i in 0..Self::MAX_SIZE {
			let byte = r.read_u8().map_err(|_| VarIntDecodeError::Incomplete)?;
			val |= (i32::from(byte) & 0b0111_1111) << (i * 7);
			if matches!(byte & 0b1000_0000, 0) {
				return Ok(val);
			}
		}

		Err(VarIntDecodeError::TooLarge)
	}
}

impl Decode<'_> for VarInt {
	fn decode(r: &mut &[u8]) -> Result<Self, ProtocolError> {
		Ok(Self(Self::decode_partial(r)?))
	}
}

impl Deref for VarInt {
	type Target = i32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for VarInt {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Encode for VarInt {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let x = self.0 as u64;
		let stage1 = (x & 0x000000000000007f)
			| ((x & 0x0000000000003f80) << 1)
			| ((x & 0x00000000001fc000) << 2)
			| ((x & 0x000000000fe00000) << 3)
			| ((x & 0x00000000f0000000) << 4);

		let leading = stage1.leading_zeros();

		let unused_bytes = (leading - 1) >> 3;
		let bytes_needed = 8 - unused_bytes;

		let msbs = 0x8080808080808080;
		let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

		let merged = stage1 | (msbs & msbmask);
		let bytes = merged.to_le_bytes();

		w.write_all(unsafe { bytes.get_unchecked(..bytes_needed as usize) })?;

		Ok(())
	}
}

impl From<i32> for VarInt {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

impl From<VarInt> for i32 {
	fn from(value: VarInt) -> Self {
		value.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarIntDecodeError {
	Incomplete,
	TooLarge,
}

impl Display for VarIntDecodeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Incomplete => "incomplete VarInt decode",
			Self::TooLarge => "VarInt is too large",
		})
	}
}

impl StdError for VarIntDecodeError {}

#[cfg(test)]
mod tests {
	use rand::prelude::*;

	use super::{Decode, Encode, ProtocolError, VarInt};

	#[test]
	fn var_int_written_size() -> Result<(), ProtocolError> {
		let mut rng = rand::rng();
		let mut buf = Vec::new();

		for n in (0..100_000)
			.map(|_| rng.random())
			.chain([0, i32::MIN, i32::MAX])
			.map(VarInt)
		{
			buf.clear();
			n.encode(&mut buf)?;
			assert_eq!(buf.len(), n.written_size());
		}

		Ok(())
	}

	#[test]
	fn var_int_round_trip() -> Result<(), ProtocolError> {
		let mut rng = rand::rng();
		let mut buf = Vec::new();

		for n in (0..1_000_000)
			.map(|_| rng.random())
			.chain([0, i32::MIN, i32::MAX])
		{
			VarInt(n).encode(&mut buf)?;

			let mut slice = buf.as_slice();
			assert!(slice.len() <= VarInt::MAX_SIZE);

			assert_eq!(n, VarInt::decode(&mut slice)?.0);

			assert!(slice.is_empty());
			buf.clear();
		}

		Ok(())
	}
}
