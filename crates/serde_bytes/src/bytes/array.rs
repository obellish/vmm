use core::{
	borrow::{Borrow, BorrowMut},
	cmp::Ordering,
	convert::TryInto as _,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
	ptr,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, SeqAccess, Visitor},
};

use super::Bytes;

#[derive(Clone, Copy, Eq, Ord)]
#[repr(transparent)]
pub struct ByteArray<const N: usize> {
	inner: [u8; N],
}

impl<const N: usize> ByteArray<N> {
	#[must_use]
	pub const fn new(bytes: [u8; N]) -> Self {
		Self { inner: bytes }
	}

	#[must_use]
	pub const fn into_array(self) -> [u8; N] {
		self.inner
	}

	const fn from_ref(bytes: &[u8; N]) -> &Self {
		unsafe { &*ptr::from_ref::<[u8; N]>(bytes).cast::<Self>() }
	}

	pub fn iter(&self) -> core::slice::Iter<'_, u8> {
		self.inner.iter()
	}

	pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, u8> {
		self.inner.iter_mut()
	}
}

impl<const N: usize> AsMut<[u8; N]> for ByteArray<N> {
	fn as_mut(&mut self) -> &mut [u8; N] {
		self
	}
}

impl<const N: usize> AsRef<[u8; N]> for ByteArray<N> {
	fn as_ref(&self) -> &[u8; N] {
		self
	}
}

impl<const N: usize> Borrow<[u8; N]> for ByteArray<N> {
	fn borrow(&self) -> &[u8; N] {
		self
	}
}

impl<const N: usize> BorrowMut<[u8; N]> for ByteArray<N> {
	fn borrow_mut(&mut self) -> &mut [u8; N] {
		self
	}
}

impl<const N: usize> Borrow<Bytes> for ByteArray<N> {
	fn borrow(&self) -> &Bytes {
		Bytes::new(&self.inner)
	}
}

impl<const N: usize> BorrowMut<Bytes> for ByteArray<N> {
	fn borrow_mut(&mut self) -> &mut Bytes {
		unsafe { &mut *(ptr::from_mut::<[u8]>(self.inner.as_mut_slice()) as *mut Bytes) }
	}
}

impl<const N: usize> Debug for ByteArray<N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.inner, f)
	}
}

impl<const N: usize> Default for ByteArray<N> {
	fn default() -> Self {
		Self::new([0; N])
	}
}

impl<const N: usize> Deref for ByteArray<N> {
	type Target = [u8; N];

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<const N: usize> DerefMut for ByteArray<N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(ByteArrayVisitor::<N>)
	}
}

impl<'a, 'de: 'a, const N: usize> Deserialize<'de> for &'a ByteArray<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_bytes(BorrowedByteArrayVisitor::<N>)
	}
}

impl<const N: usize> From<[u8; N]> for ByteArray<N> {
	fn from(value: [u8; N]) -> Self {
		Self::new(value)
	}
}

impl<const N: usize> Hash for ByteArray<N> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl<const N: usize> IntoIterator for ByteArray<N> {
	type IntoIter = core::array::IntoIter<u8, N>;
	type Item = u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

impl<'a, const N: usize> IntoIterator for &'a ByteArray<N> {
	type IntoIter = core::slice::Iter<'a, u8>;
	type Item = &'a u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

impl<'a, const N: usize> IntoIterator for &'a mut ByteArray<N> {
	type IntoIter = core::slice::IterMut<'a, u8>;
	type Item = &'a mut u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter_mut()
	}
}

impl<Rhs, const N: usize> PartialEq<Rhs> for ByteArray<N>
where
	Rhs: ?Sized + Borrow<[u8; N]>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_ref().eq(other.borrow())
	}
}

impl<Rhs, const N: usize> PartialOrd<Rhs> for ByteArray<N>
where
	Rhs: ?Sized + Borrow<[u8; N]>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_ref().partial_cmp(other.borrow())
	}
}

impl<const N: usize> Serialize for ByteArray<N> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(&**self)
	}
}

struct ByteArrayVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for ByteArrayVisitor<N> {
	type Value = ByteArray<N>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a byte array of length ")?;
		Display::fmt(&N, formatter)
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let mut bytes = [0; N];

		for (idx, byte) in bytes.iter_mut().enumerate() {
			*byte = seq
				.next_element()?
				.ok_or_else(|| DeError::invalid_length(idx, &self))?;
		}

		Ok(ByteArray::new(bytes))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(ByteArray {
			inner: v
				.try_into()
				.map_err(|_| DeError::invalid_length(v.len(), &self))?,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_bytes(v.as_bytes())
	}
}

struct BorrowedByteArrayVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for BorrowedByteArrayVisitor<N> {
	type Value = &'de ByteArray<N>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a borrowed byte array of length ")?;
		Display::fmt(&N, formatter)
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		let borrowed_byte_array: &'de [u8; N] = v
			.try_into()
			.map_err(|_| DeError::invalid_length(v.len(), &self))?;

		Ok(ByteArray::from_ref(borrowed_byte_array))
	}

	fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_borrowed_bytes(v.as_bytes())
	}
}
