use std::io::Write;

use super::{Decode, Encode, ProtocolError, VarInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FixedArray<T, const N: usize>(pub [T; N]);

impl<'a, T, const N: usize> Decode<'a> for FixedArray<T, N>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert_eq!(
			len, N as i32,
			"unexpected length of {len} for fixed-sized array of length {N}"
		);

		<[T; N]>::decode(r).map(Self)
	}
}

impl<T: Encode, const N: usize> Encode for FixedArray<T, N> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		VarInt(N as i32).encode(&mut w)?;
		self.0.encode(w)
	}
}

impl<T, const N: usize> From<[T; N]> for FixedArray<T, N> {
	fn from(value: [T; N]) -> Self {
		Self(value)
	}
}

impl<T, const N: usize> From<FixedArray<T, N>> for [T; N] {
	fn from(value: FixedArray<T, N>) -> Self {
		value.0
	}
}
