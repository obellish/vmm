#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, boxed::Box, vec::Vec};

use serde::{Serialize as SerdeSerialize, Serializer};

#[cfg(feature = "alloc")]
use super::ByteBuf;
use super::{ByteArray, Bytes};

pub trait Serialize {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl Serialize for [u8] {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

#[cfg(feature = "alloc")]
impl Serialize for Vec<u8> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

impl Serialize for Bytes {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

impl<const N: usize> Serialize for [u8; N] {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

impl<const N: usize> Serialize for ByteArray<N> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(&**self)
	}
}

#[cfg(feature = "alloc")]
impl Serialize for ByteBuf {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

#[cfg(feature = "alloc")]
impl Serialize for Cow<'_, [u8]> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

#[cfg(feature = "alloc")]
impl Serialize for Cow<'_, Bytes> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

impl<T> Serialize for &T
where
	T: ?Sized + Serialize,
{
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		(**self).serialize(serializer)
	}
}

#[cfg(feature = "alloc")]
impl<T> Serialize for Box<T>
where
	T: ?Sized + Serialize,
{
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		(**self).serialize(serializer)
	}
}

impl<T: Serialize> Serialize for Option<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[repr(transparent)]
		struct AsBytes<T>(T);

		impl<T: Serialize> SerdeSerialize for AsBytes<T> {
			fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
				self.0.serialize(serializer)
			}
		}

		match self {
			Some(b) => serializer.serialize_some(&AsBytes(b)),
			None => serializer.serialize_none(),
		}
	}
}
