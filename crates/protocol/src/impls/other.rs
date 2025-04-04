use std::io::Write;

use uuid::Uuid;
use vmm_generated::{
	block::{BlockEntityKind, BlockKind, BlockState},
	item::ItemKind,
};
use vmm_ident::{Ident, IdentError};
use vmm_nbt::Compound;

use crate::{Decode, Encode, ProtocolError, VarInt};

impl<'a, T> Decode<'a> for Option<T>
where
	T: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(if bool::decode(r)? {
			Some(T::decode(r)?)
		} else {
			None
		})
	}
}

impl<T: Encode> Encode for Option<T> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		match self {
			Some(t) => {
				true.encode(&mut w)?;
				t.encode(w)
			}
			None => false.encode(w),
		}
	}
}

impl Decode<'_> for Uuid {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		u128::decode(r).map(Self::from_u128)
	}
}

impl Encode for Uuid {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_u128().encode(w)
	}
}

impl Decode<'_> for Compound {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		if matches!(r.first(), Some(&0)) {
			*r = &r[1..];
			return Ok(Self::new());
		}

		Ok(vmm_nbt::from_binary(r)?.0)
	}
}

impl Encode for Compound {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		Ok(vmm_nbt::to_binary(self, w, "")?)
	}
}

impl<'a, S> Decode<'a> for Ident<S>
where
	S: Decode<'a>,
	Ident<S>: TryFrom<S, Error = IdentError>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		Ok(Self::try_from(S::decode(r)?)?)
	}
}

impl<S: Encode> Encode for Ident<S> {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		self.as_ref().encode(w)
	}
}

impl Decode<'_> for BlockState {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		let id = VarInt::decode(r)?.0;

		Self::from_raw(u16::try_from(id).map_err(|e| ProtocolError::Other(e.to_string()))?)
			.ok_or_else(|| ProtocolError::Other("invalid block state ID".to_owned()))
	}
}

impl Encode for BlockState {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		VarInt(i32::from(self.to_raw())).encode(w)
	}
}

impl Decode<'_> for BlockKind {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		let id = VarInt::decode(r)?.0;

		Self::from_raw(u16::try_from(id).map_err(|e| ProtocolError::Other(e.to_string()))?)
			.ok_or_else(|| ProtocolError::Other("invalid block kind ID".to_owned()))
	}
}

impl Encode for BlockKind {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		VarInt(i32::from(self.to_raw())).encode(w)
	}
}

impl<'a> Decode<'a> for BlockEntityKind {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let id = VarInt::decode(r)?;
		Self::from_id(id.0 as u32).ok_or_else(|| ProtocolError::Other(format!("id {}", id.0)))
	}
}

impl Encode for BlockEntityKind {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		VarInt(self.id() as i32).encode(w)
	}
}

impl Decode<'_> for ItemKind {
	fn decode(r: &mut &'_ [u8]) -> Result<Self, ProtocolError> {
		let id = VarInt::decode(r)?.0;

		Self::from_raw(u16::try_from(id).map_err(|e| ProtocolError::Other(e.to_string()))?)
			.ok_or_else(|| ProtocolError::Other("invalid item ID".to_owned()))
	}
}

impl Encode for ItemKind {
	fn encode(&self, w: impl Write) -> Result<(), ProtocolError> {
		VarInt(i32::from(self.to_raw())).encode(w)
	}
}
