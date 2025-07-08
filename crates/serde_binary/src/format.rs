use core::mem;

use super::{Error, Result};
use crate::{Input, Output};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
	Null = 0,
	False,
	True,
	UnsignedInt,
	SignedInt,
	Float16,
	Float32,
	Float64,
	Float128,
	Bytes = 10,
	String,
	SeqStart = 15,
	SeqEnd,
	MapStart,
	MapEnd,
}

impl Type {
	#[must_use]
	pub const fn as_u8(self) -> u8 {
		self as u8
	}
}

impl From<Type> for u8 {
	fn from(value: Type) -> Self {
		value.as_u8()
	}
}

impl TryFrom<u8> for Type {
	type Error = Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Self::Null),
			1 => Ok(Self::False),
			2 => Ok(Self::True),
			3 => Ok(Self::UnsignedInt),
			4 => Ok(Self::SignedInt),
			5 => Ok(Self::Float16),
			6 => Ok(Self::Float32),
			7 => Ok(Self::Float64),
			8 => Ok(Self::Float128),
			10 => Ok(Self::Bytes),
			11 => Ok(Self::String),
			15 => Ok(Self::SeqStart),
			16 => Ok(Self::SeqEnd),
			17 => Ok(Self::MapStart),
			18 => Ok(Self::MapEnd),
			_ => Err(Error::InvalidType(value)),
		}
	}
}

pub trait VarInt: Sized {
	const MAX_BYTES: usize = varint_max::<Self>();

	fn encode<O: Output>(&self, output: &mut O) -> Result<()>;

	fn decode<'de, I>(input: &mut I) -> Result<Self>
	where
		I: Input<'de>;
}

macro_rules! impl_varint_unsigned {
	($($ty:ty)*) => {
		$(
			impl $crate::VarInt for $ty {
				fn encode<O: $crate::Output>(&self, output: &mut O) -> $crate::Result<()> {
					let mut value = *self;

					for _ in 0..$crate::format::varint_max::<$ty>() {
						let byte = value.to_le_bytes()[0];

						if value < 0x80 {
							output.write_byte(byte)?;
							return $crate::Result::Ok(())
						}

						output.write_byte(byte | 0x80)?;
						value >>= 7;
					}

					panic!("varint needs more than maximum bytes");
				}

				fn decode<'de, I>(input: &mut I) -> $crate::Result<Self>
				where
					I: $crate::Input<'de>,
				{
					let mut value = 0;
					let mut bits = <$ty>::BITS;
					for i in 0..varint_max::<$ty>() {
						let byte = input.read_byte()?;

						if bits < 8 && !matches!((byte & 0x7F) >> bits, 0) {
							return $crate::Result::Err($crate::Error::VarIntTooLarge);
						}
						bits = bits.saturating_sub(7);

						value |= (<$ty>::from(byte & 0x7F)) << (i * 7);
						if matches!(byte & 0x80, 0) {
							return $crate::Result::Ok(value);
						}
					}

					$crate::Result::Err($crate::Error::VarIntTooLarge)
				}
			}
		)*
	};
}

macro_rules! impl_varint_signed {
	($($u:ty => $t:ty),*) => {
		$(
			impl $crate::VarInt for $t {
				fn encode<O: $crate::Output>(&self, output: &mut O) -> $crate::Result<()> {
					let value = if self.is_negative() {
						self.rotate_left(1).wrapping_neg()
					} else {
						self.rotate_left(1)
					} as $u;

					<$u>::encode(&value, output)
				}

				fn decode<'de, I>(input: &mut I) -> $crate::Result<Self>
				where
					I: $crate::Input<'de>,
				{
					let value = <$u>::decode(input)? as $t;
					if !matches!(value & 1, 0) {
						$crate::Result::Ok(value.wrapping_neg().rotate_right(1))
					} else {
						$crate::Result::Ok(value.rotate_right(1))
					}
				}
			}
		)*
	};
}

impl_varint_unsigned!(u8 u16 u32 u64 u128 usize);
impl_varint_signed!(u8 => i8, u16 => i16, u32 => i32, u64 => i64, u128 => i128, usize => isize);

#[must_use]
pub const fn varint_max<T>() -> usize {
	let bits = mem::size_of::<T>() * 8;
	bits.div_ceil(7)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn type_conversion_works() {
		let valid_types = [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 15, 16, 17, 18];
		for byte in 0..=u8::MAX {
			match Type::try_from(byte) {
				Ok(t) => {
					assert!(
						valid_types.contains(&byte),
						"type {t:?} should have been recognized from {byte} here"
					);
					assert_eq!(u8::from(t), byte);
				}
				Err(..) => assert!(
					!valid_types.contains(&byte),
					"type should have been recognized from {byte}"
				),
			}
		}
	}

	#[test]
	fn unsigned_varint_encode() -> Result<()> {
		let mut bytes = [0; 1];
		let mut output = bytes.as_mut_slice();
		0u8.encode(&mut output)?;
		assert_eq!(bytes, [0]);
		let mut output = bytes.as_mut_slice();
		0x7Fu8.encode(&mut output)?;
		assert_eq!(bytes, [0x7F]);
		let mut output = bytes.as_mut_slice();
		let result = 0xFFu8.encode(&mut output);
		assert!(matches!(result, Err(Error::BufferTooSmall)));

		let mut bytes = [0; 10];
		let mut output = bytes.as_mut_slice();
		0xFFu8.encode(&mut output)?;
		assert_eq!(&bytes[0..2], [0xFF, 0x01]);
		let mut output = bytes.as_mut_slice();
		0xFFusize.encode(&mut output)?;
		assert_eq!(&bytes[0..2], [0xFF, 0x01]);

		let mut bytes = [0; u32::MAX_BYTES];
		let mut output = bytes.as_mut_slice();
		64u32.encode(&mut output)?;
		assert_eq!(&bytes[0..1], [0x40]);
		let mut output = bytes.as_mut_slice();
		0xFFFF_FFFFu32.encode(&mut output)?;
		assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
		let mut output = bytes.as_mut_slice();
		0x0196_0713u32.encode(&mut output)?;
		assert_eq!(&bytes[0..4], [0x93, 0x8E, 0xD8, 0x0C]);

		Ok(())
	}

	#[test]
	fn unsigned_varint_decode() -> Result<()> {
		let bytes = &[0x00; 2];
		let mut input = bytes.as_slice();
		let value = u16::decode(&mut input)?;
		assert_eq!(input.len(), 1);
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x00];
		let value = u16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, 0);

		let bytes = [0x80, 0x80, 0x80, 0x00];
		let result = u16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		let bytes = &[0xFF, 0xFF, 0x03];
		let value = u16::decode(&mut bytes.as_slice()).unwrap();
		assert_eq!(value, 0xFFFF);

		let bytes = &[0xFF, 0xFF, 0x07];
		let result = u16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		Ok(())
	}

	#[test]
	fn signed_varint_encode() -> Result<()> {
		let mut bytes = [0; 1];
		let mut output = bytes.as_mut_slice();
		0i8.encode(&mut output)?;
		assert_eq!(bytes, [0]);
		let mut output = bytes.as_mut_slice();
		(-1i8).encode(&mut output)?;
		assert_eq!(bytes, [0x01]);
		let mut output = bytes.as_mut_slice();
		(1i8).encode(&mut output)?;
		assert_eq!(bytes, [0x02]);
		let mut output = bytes.as_mut_slice();
		let result = 64i8.encode(&mut output);
		assert!(matches!(result, Err(Error::BufferTooSmall)));

		let mut bytes = [0; 10];
		let mut output = bytes.as_mut_slice();
		64i8.encode(&mut output)?;
		assert_eq!(&bytes[0..2], [0x80, 0x01]);
		let mut output = bytes.as_mut_slice();
		(-65i8).encode(&mut output)?;
		assert_eq!(&bytes[0..2], [0x81, 0x01]);
		let mut output = bytes.as_mut_slice();
		(-65isize).encode(&mut output)?;
		assert_eq!(&bytes[0..2], [0x81, 0x01]);

		let mut bytes = [0; i32::MAX_BYTES];
		let mut output = bytes.as_mut_slice();
		0x7FFFi32.encode(&mut output)?;
		assert_eq!(&bytes[0..3], [0xFE, 0xFF, 0x03]);
		let mut output = bytes.as_mut_slice();
		(-0x8000i32).encode(&mut output)?;
		assert_eq!(&bytes[0..3], [0xFF, 0xFF, 0x03]);

		Ok(())
	}

	#[test]
	fn signed_varint_decode() -> Result<()> {
		let bytes = &[0x00, 0x00];
		let mut input = bytes.as_slice();
		let value = i16::decode(&mut input)?;
		assert_eq!(input.len(), 1);
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x00];
		let value = i16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, 0);

		let bytes = &[0x80, 0x80, 0x80, 0x00];
		let result = i16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		let bytes = &[0x80, 0x01];
		let value = i16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, 64);

		let bytes = &[0x81, 0x01];
		let value = i16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, -65);

		let bytes = &[0xFE, 0xFF, 0x03];
		let value = i16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, 0x7FFF);

		let bytes = &[0xFF, 0xFF, 0x03];
		let value = i16::decode(&mut bytes.as_slice())?;
		assert_eq!(value, -0x8000);

		let bytes = &[0xFF, 0xFF, 0x07];
		let result = i16::decode(&mut bytes.as_slice());
		assert!(matches!(result, Err(Error::VarIntTooLarge)));

		Ok(())
	}
}
