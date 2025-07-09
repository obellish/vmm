mod array;
#[cfg(feature = "alloc")]
mod buf;

#[cfg(feature = "alloc")]
use alloc::{borrow::ToOwned, boxed::Box};
use core::{
	cmp::Ordering,
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
	ptr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use self::array::*;
#[cfg(feature = "alloc")]
pub use self::buf::*;

#[derive(Eq, Ord)]
#[repr(transparent)]
pub struct Bytes {
	inner: [u8],
}

impl Bytes {
	#[must_use]
	pub const fn new(bytes: &[u8]) -> &Self {
		unsafe { &*(ptr::from_ref::<[u8]>(bytes) as *const Self) }
	}
}

impl AsMut<[u8]> for Bytes {
	fn as_mut(&mut self) -> &mut [u8] {
		self
	}
}

impl AsRef<[u8]> for Bytes {
	fn as_ref(&self) -> &[u8] {
		self
	}
}

impl Debug for Bytes {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.inner, f)
	}
}

impl Default for &Bytes {
	fn default() -> Self {
		Bytes::new(&[])
	}
}

#[cfg(feature = "alloc")]
impl Default for Box<Bytes> {
	fn default() -> Self {
		ByteBuf::new().into_boxed_bytes()
	}
}

impl Deref for Bytes {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Bytes {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<'a, 'de: 'a> Deserialize<'de> for &'a Bytes {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(Bytes::new)
	}
}

impl<'a> From<&'a [u8]> for &'a Bytes {
	fn from(value: &'a [u8]) -> Self {
		Bytes::new(value)
	}
}

#[cfg(feature = "alloc")]
impl From<Box<[u8]>> for Box<Bytes> {
	fn from(value: Box<[u8]>) -> Self {
		unsafe { Self::from_raw(Box::into_raw(value) as *mut Bytes) }
	}
}

impl Hash for Bytes {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.inner.hash(state);
	}
}

impl<'a> IntoIterator for &'a Bytes {
	type IntoIter = core::slice::Iter<'a, u8>;
	type Item = &'a u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

impl<'a> IntoIterator for &'a mut Bytes {
	type IntoIter = core::slice::IterMut<'a, u8>;
	type Item = &'a mut u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter_mut()
	}
}

impl<Rhs> PartialEq<Rhs> for Bytes
where
	Rhs: ?Sized + AsRef<[u8]>,
{
	fn eq(&self, other: &Rhs) -> bool {
		self.as_ref().eq(other.as_ref())
	}
}

impl<Rhs> PartialOrd<Rhs> for Bytes
where
	Rhs: ?Sized + AsRef<[u8]>,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
		self.as_ref().partial_cmp(other.as_ref())
	}
}

impl Serialize for Bytes {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_bytes(self)
	}
}

#[cfg(feature = "alloc")]
impl ToOwned for Bytes {
	type Owned = ByteBuf;

	fn to_owned(&self) -> Self::Owned {
		ByteBuf::from(self.inner.to_owned())
	}
}
