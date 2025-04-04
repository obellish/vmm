use std::{borrow::Cow, io::Write, rc::Rc, sync::Arc};

use crate::{Decode, Encode, ProtocolError};

impl<'a, T> Decode<'a> for Box<T>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		T::decode(r).map(Self::new)
	}
}

impl<'a, T> Decode<'a> for Rc<T>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		T::decode(r).map(Self::new)
	}
}

impl<'a, T> Decode<'a> for Arc<T>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		T::decode(r).map(Self::new)
	}
}

impl<'a, B> Decode<'a> for Cow<'_, B>
where
	B: ?Sized + ToOwned,
	B::Owned: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		B::Owned::decode(r).map(Self::Owned)
	}
}

impl<T> Encode for &T
where
	T: ?Sized + Encode,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		(**self).encode(w)
	}
}

impl<T> Encode for &mut T
where
	T: ?Sized + Encode,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		(**self).encode(w)
	}
}

impl<T> Encode for Box<T>
where
	T: ?Sized + Encode,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_ref().encode(w)
	}
}

impl<T> Encode for Rc<T>
where
	T: ?Sized + Encode,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_ref().encode(w)
	}
}

impl<T> Encode for Arc<T>
where
	T: ?Sized + Encode,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_ref().encode(w)
	}
}

impl<B> Encode for Cow<'_, B>
where
	B: ?Sized + Encode + ToOwned,
{
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_ref().encode(w)
	}
}
