use std::{
	io::Write,
	mem::{self, MaybeUninit},
	slice,
};

use super::cautious_capacity;
use crate::{Bounded, Decode, Encode, ProtocolError, VarInt};

impl<'a, const N: usize> Decode<'a> for &'a [u8; N] {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		assert!(
			r.len() >= N,
			"not enough data to decode u8 array of length {N}"
		);

		let (res, remaining) = r.split_at(N);
		let arr = <&[u8; N]>::try_from(res).map_err(|e| ProtocolError::Other(e.to_string()))?;
		*r = remaining;
		Ok(arr)
	}
}

impl<'a> Decode<'a> for &'a [u8] {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode slice with negative length");
		let len = len as usize;
		assert!(
			len <= r.len(),
			"not enough data remaining to decode byte slice (slice len is {len}, but input len is {})",
			r.len()
		);

		let (res, remaining) = r.split_at(len);
		*r = remaining;
		Ok(res)
	}
}

impl<'a, T, const N: usize> Decode<'a> for [T; N]
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

		for (i, elem) in data.iter_mut().enumerate() {
			match T::decode(r) {
				Ok(val) => {
					elem.write(val);
				}
				Err(e) => {
					for elem in &mut data[..i] {
						unsafe {
							elem.assume_init_drop();
						}
					}
					return Err(e);
				}
			}
		}

		unsafe { Ok(mem::transmute_copy(&data)) }
	}
}

impl<T: Encode, const N: usize> Encode for [T; N] {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		T::encode_slice(self, w)
	}
}

impl<T: Encode> Encode for [T] {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();
		assert!(
			i32::try_from(len).is_ok(),
			"length of {} slice exceeds i32::MAX (got {len})",
			std::any::type_name::<T>()
		);

		VarInt(len as i32).encode(&mut w)?;

		T::encode_slice(self, w)
	}
}

impl<T: Encode, const MAX_LEN: usize> Encode for Bounded<&'_ [T], MAX_LEN> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();
		assert!(
			len <= MAX_LEN,
			"length of {} slice exceeds max of {MAX_LEN} (got {len})",
			std::any::type_name::<T>()
		);

		VarInt(len as i32).encode(&mut w)?;

		T::encode_slice(self, w)
	}
}

impl<'a, const MAX_LEN: usize> Decode<'a> for Bounded<&'a [u8], MAX_LEN> {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let res = <&[u8]>::decode(r)?;

		assert!(
			res.len() <= MAX_LEN,
			"length of decoded byte slice exceeds max of {MAX_LEN} (got {})",
			res.len()
		);

		Ok(Self(res))
	}
}

impl<'a> Decode<'a> for &'a [i8] {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let bytes = <&[u8]>::decode(r)?;

		let bytes = unsafe { slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len()) };

		Ok(bytes)
	}
}

impl<T: Encode> Encode for Vec<T> {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_slice().encode(w)
	}
}

impl<'a, T> Decode<'a> for Vec<T>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode Vec with negative length");
		let len = len as usize;

		let mut vec = Self::with_capacity(cautious_capacity::<T>(len));

		for _ in 0..len {
			vec.push(T::decode(r)?);
		}

		Ok(vec)
	}
}

impl<'a, T, const MAX_LEN: usize> Decode<'a> for Bounded<Vec<T>, MAX_LEN>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode Vec with negative length");
		let len = len as usize;

		assert!(
			len <= MAX_LEN,
			"length of Vec exceeds max of {MAX_LEN} (got {len})"
		);

		let mut vec = Vec::with_capacity(len);

		for _ in 0..len {
			vec.push(T::decode(r)?);
		}

		Ok(Self(vec))
	}
}

impl<'a, T> Decode<'a> for Box<[T]>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(Vec::decode(r)?.into_boxed_slice())
	}
}

impl<'a, T, const MAX_LEN: usize> Decode<'a> for Bounded<Box<[T]>, MAX_LEN>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(Bounded::<Vec<_>, MAX_LEN>::decode(r)?.map_into())
	}
}
