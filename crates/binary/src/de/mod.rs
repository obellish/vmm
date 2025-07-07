mod error;

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, marker::PhantomData};

use serde::{
	Deserializer as SerdeDeserializer,
	de::{
		DeserializeSeed, EnumAccess, Error as DeError, IntoDeserializer, MapAccess, SeqAccess,
		Unexpected, VariantAccess, value::BytesDeserializer,
	},
};
use vmm_binary_io::Read;
use vmm_binary_ll::{Decoder, Header, simple, tag};

pub use self::error::*;
use super::TagAccess;

pub struct Deserializer<'b, R> {
	decoder: Decoder<R>,
	scratch: &'b mut [u8],
	recurse: usize,
}

impl<R> Deserializer<'_, R>
where
	R: Read,
	R::Error: Debug,
{
	fn recurse<V>(
		&mut self,
		func: impl FnOnce(&mut Self) -> Result<V, Error<R::Error>>,
	) -> Result<V, Error<R::Error>> {
		if matches!(self.recurse, 0) {
			return Err(Error::RecursionLimitExceeded);
		}

		self.recurse -= 1;
		let result = func(self);
		self.recurse += 1;
		result
	}

	fn integer(
		&mut self,
		mut header: Option<Header>,
		should_append: bool,
		mut append: impl FnMut(u8),
	) -> Result<(bool, u128), Error<R::Error>> {
		loop {
			let header = match header.take() {
				Some(h) => h,
				None => self.decoder.pull()?,
			};

			let neg = match header {
				Header::Positive(x) => return Ok((false, x.into())),
				Header::Negative(x) => return Ok((true, x.into())),
				Header::Tag(tag::BIGPOS) => false,
				Header::Tag(tag::BIGNEG) => true,
				Header::Tag(..) => continue,
				header => return Err(header.expected("integer")),
			};

			let mut buffer = [0u8; 16];
			let mut value = [0u8; 16];
			let mut index = 0usize;

			return match self.decoder.pull()? {
				Header::Bytes(len) => {
					let mut segments = self.decoder.bytes(len);
					while let Some(mut segment) = segments.pull()? {
						while let Some(chunk) = segment.pull(&mut buffer)? {
							for b in chunk {
								match index {
									16 => {
										if should_append {
											for v in value {
												append(v);
											}

											append(*b);
											index = 17;
											continue;
										}

										return Err(DeError::custom("bigint too large"));
									}
									17 => {
										debug_assert!(should_append);
										append(*b);
										continue;
									}
									0 if matches!(*b, 0) => continue,
									_ => value[index] = *b,
								}

								index += 1;
							}
						}
					}

					if matches!(index, 17) {
						Ok((false, 0))
					} else {
						value[..index].reverse();
						Ok((neg, u128::from_le_bytes(value)))
					}
				}
				h => Err(h.expected("bytes")),
			};
		}
	}
}

trait Expected<E: DeError> {
	fn expected(self, kind: &'static str) -> E;
}

impl<E: DeError> Expected<E> for Header {
	fn expected(self, kind: &'static str) -> E {
		DeError::invalid_type(
			match self {
				Self::Positive(x) => Unexpected::Unsigned(x),
				Self::Negative(x) => Unexpected::Signed(x as i64 ^ !0),
				Self::Bytes(..) => Unexpected::Other("bytes"),
				Self::Text(..) => Unexpected::Other("text"),
				Self::Array(..) => Unexpected::Seq,
				Self::Map(..) => Unexpected::Map,
				Self::Tag(..) => Unexpected::Other("tag"),
				Self::Simple(simple::FALSE) => Unexpected::Bool(false),
				Self::Simple(simple::TRUE) => Unexpected::Bool(true),
				Self::Simple(simple::NULL) => Unexpected::Other("null"),
				Self::Simple(simple::UNDEFINED) => Unexpected::Other("undefined"),
				Self::Simple(..) => Unexpected::Other("simple"),
				Self::Float(x) => Unexpected::Float(x),
				Self::Break => Unexpected::Other("break"),
			},
			&kind,
		)
	}
}

impl<'de, 'a, 'b, R> SerdeDeserializer<'de> for &'a mut Deserializer<'b, R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;
}

struct Access<'a, 'b, R>(&'a mut Deserializer<'b, R>, Option<usize>);

impl<'de, 'a, 'b, R> EnumAccess<'de> for Access<'a, 'b, R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let variant = seed.deserialize(&mut *self.0)?;
		Ok((variant, self))
	}
}

impl<'de, 'a, 'b, R> MapAccess<'de> for Access<'a, 'b, R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		match self.1 {
			Some(0) => return Ok(None),
			Some(x) => self.1 = Some(x - 1),
			None => match self.0.decoder.pull()? {
				Header::Break => return Ok(None),
				header => self.0.decoder.push(header),
			},
		}

		seed.deserialize(&mut *self.0).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.0)
	}

	fn size_hint(&self) -> Option<usize> {
		self.1
	}
}

impl<'de, 'a, 'b, R> SeqAccess<'de> for Access<'a, 'b, R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.1 {
			Some(0) => return Ok(None),
			Some(x) => self.1 = Some(x - 1),
			None => match self.0.decoder.pull()? {
				Header::Break => return Ok(None),
				header => self.0.decoder.push(header),
			},
		}

		seed.deserialize(&mut *self.0).map(Some)
	}

	fn size_hint(&self) -> Option<usize> {
		self.1
	}
}

impl<'de, 'a, 'b, R> VariantAccess<'de> for Access<'a, 'b, R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;

	fn unit_variant(self) -> Result<(), Self::Error> {
		Ok(())
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(&mut *self.0)
	}

	fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.0.deserialize_any(visitor)
	}

	fn struct_variant<V>(
		self,
		_: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.0.deserialize_any(visitor)
	}
}

struct BytesAccess<R>(usize, Vec<u8>, PhantomData<R>);

impl<'de, R> SeqAccess<'de> for BytesAccess<R>
where
	R: Read,
	R::Error: Debug,
{
	type Error = Error<R::Error>;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		if self.0 < self.1.len() {
			let byte = self.1[self.0];
			self.0 += 1;
			seed.deserialize(byte.into_deserializer()).map(Some)
		} else {
			Ok(None)
		}
	}

	fn size_hint(&self) -> Option<usize> {
		Some(self.1.len() - self.0)
	}
}

const fn noop(_: u8) {}
