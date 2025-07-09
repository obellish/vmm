#[cfg(feature = "alloc")]
use alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	vec::Vec,
};
use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::de::{Deserialize as SerdeDeserialize, Deserializer, Error as DeError, Visitor};

#[cfg(feature = "alloc")]
use super::ByteBuf;
use super::{ByteArray, Bytes};

pub trait Deserialize<'de>: Sized {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>;
}

impl<'a, 'de: 'a> Deserialize<'de> for &'a [u8] {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer)
	}
}

#[cfg(feature = "alloc")]
impl<'de> Deserialize<'de> for Vec<u8> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer).map(ByteBuf::into_vec)
	}
}

impl<'a, 'de: 'a> Deserialize<'de> for &'a Bytes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer).map(Bytes::new)
	}
}

impl<'de, const N: usize> Deserialize<'de> for [u8; N] {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let arr: ByteArray<N> = SerdeDeserialize::deserialize(deserializer)?;
		Ok(*arr)
	}
}

impl<'de, const N: usize> Deserialize<'de> for &'de [u8; N] {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let arr: &ByteArray<N> = SerdeDeserialize::deserialize(deserializer)?;
		Ok(arr)
	}
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer)
	}
}

impl<'a, 'de: 'a, const N: usize> Deserialize<'de> for &'a ByteArray<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer)
	}
}

#[cfg(feature = "alloc")]
impl<'de> Deserialize<'de> for ByteBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		SerdeDeserialize::deserialize(deserializer)
	}
}

#[cfg(feature = "alloc")]
impl<'a, 'de: 'a> Deserialize<'de> for Cow<'a, [u8]> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct CowVisitor;

		impl<'de> Visitor<'de> for CowVisitor {
			type Value = Cow<'de, [u8]>;

			fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
				formatter.write_str("a byte array")
			}

			fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(Cow::Borrowed(v))
			}

			fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(Cow::Borrowed(v.as_bytes()))
			}

			fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				use alloc::borrow::ToOwned;

				Ok(Cow::Owned(v.to_owned()))
			}

			fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(Cow::Owned(v))
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(Cow::Owned(v.as_bytes().to_owned()))
			}

			fn visit_string<E>(self, v: alloc::string::String) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(Cow::Owned(v.into_bytes()))
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let len = core::cmp::min(seq.size_hint().unwrap_or(0), 4096);
				let mut bytes = Vec::with_capacity(len);

				while let Some(b) = seq.next_element()? {
					bytes.push(b);
				}

				Ok(Cow::Owned(bytes))
			}
		}

		deserializer.deserialize_bytes(CowVisitor)
	}
}

#[cfg(feature = "alloc")]
impl<'a, 'de: 'a> Deserialize<'de> for Cow<'a, Bytes> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let cow: Cow<'a, [u8]> = Deserialize::deserialize(deserializer)?;

		match cow {
			Cow::Borrowed(bytes) => Ok(Self::Borrowed(Bytes::new(bytes))),
			Cow::Owned(bytes) => Ok(Self::Owned(ByteBuf::from(bytes))),
		}
	}
}

#[cfg(feature = "alloc")]
impl<'de> Deserialize<'de> for Box<[u8]> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(Vec::into_boxed_slice)
	}
}

#[cfg(feature = "alloc")]
impl<'de> Deserialize<'de> for Box<Bytes> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let bytes: Box<[u8]> = Deserialize::deserialize(deserializer)?;
		Ok(bytes.into())
	}
}

impl<'de, T> Deserialize<'de> for Option<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct BytesVisitor<T>(PhantomData<T>);

		impl<'de, T> Visitor<'de> for BytesVisitor<T>
		where
			T: Deserialize<'de>,
		{
			type Value = Option<T>;

			fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
				formatter.write_str("optional byte array")
			}

			fn visit_unit<E>(self) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(None)
			}

			fn visit_none<E>(self) -> Result<Self::Value, E>
			where
				E: DeError,
			{
				Ok(None)
			}

			fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
			where
				D: Deserializer<'de>,
			{
				T::deserialize(deserializer).map(Some)
			}
		}

		deserializer.deserialize_option(BytesVisitor(PhantomData))
	}
}
