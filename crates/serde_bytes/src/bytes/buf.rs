use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use core::{
	borrow::{Borrow, BorrowMut},
	cmp::{self, Ordering},
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
	ptr,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, SeqAccess, Visitor},
};

use super::Bytes;

#[derive(Default, Clone, Eq, Ord)]
#[repr(transparent)]
pub struct ByteBuf {
	inner: Vec<u8>,
}

impl ByteBuf {
	#[must_use]
	pub const fn new() -> Self {
		Self { inner: Vec::new() }
	}

	#[must_use]
	pub fn with_capacity(cap: usize) -> Self {
		Self {
			inner: Vec::with_capacity(cap),
		}
	}

	#[must_use]
	pub fn into_vec(self) -> Vec<u8> {
		self.inner
	}

	#[must_use]
	pub fn into_boxed_slice(self) -> Box<[u8]> {
		self.inner.into_boxed_slice()
	}

	#[must_use]
	pub fn into_boxed_bytes(self) -> Box<Bytes> {
		self.into_boxed_slice().into()
	}
}

impl AsMut<[u8]> for ByteBuf {
	fn as_mut(&mut self) -> &mut [u8] {
		self
	}
}

impl AsRef<[u8]> for ByteBuf {
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Borrow<Bytes> for ByteBuf {
	fn borrow(&self) -> &Bytes {
		Bytes::new(&self.inner)
	}
}

impl BorrowMut<Bytes> for ByteBuf {
	fn borrow_mut(&mut self) -> &mut Bytes {
		unsafe { &mut *(ptr::from_mut::<[u8]>(self.inner.as_mut_slice()) as *mut Bytes) }
	}
}

impl Debug for ByteBuf {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.inner, f)
	}
}

impl Deref for ByteBuf {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for ByteBuf {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<'de> Deserialize<'de> for ByteBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_byte_buf(ByteBufVisitor)
	}
}

impl From<Vec<u8>> for ByteBuf {
	fn from(value: Vec<u8>) -> Self {
		Self { inner: value }
	}
}

impl Hash for ByteBuf {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl IntoIterator for ByteBuf {
	type IntoIter = alloc::vec::IntoIter<u8>;
	type Item = u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

impl<'a> IntoIterator for &'a ByteBuf {
	type IntoIter = core::slice::Iter<'a, u8>;
	type Item = &'a u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

impl<'a> IntoIterator for &'a mut ByteBuf {
	type IntoIter = core::slice::IterMut<'a, u8>;
	type Item = &'a mut u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter_mut()
	}
}

impl<Rhs> PartialEq<Rhs> for ByteBuf
where
	Rhs: ?Sized + AsRef<[u8]>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_ref().eq(other.as_ref())
	}
}

impl<Rhs> PartialOrd<Rhs> for ByteBuf
where
	Rhs: ?Sized + AsRef<[u8]>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_ref().partial_cmp(other.as_ref())
	}
}

impl Serialize for ByteBuf {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

struct ByteBufVisitor;

impl<'de> Visitor<'de> for ByteBufVisitor {
	type Value = ByteBuf;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("byte array")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let len = cmp::min(seq.size_hint().unwrap_or(0), 4096);
		let mut bytes = Vec::with_capacity(len);

		while let Some(b) = seq.next_element()? {
			bytes.push(b);
		}

		Ok(ByteBuf { inner: bytes })
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(ByteBuf::from(v.to_owned()))
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(ByteBuf::from(v))
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_bytes(v.as_bytes())
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(ByteBuf::from(v.into_bytes()))
	}
}
