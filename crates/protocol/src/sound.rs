use std::{borrow::Cow, io::Write};

pub use vmm_generated::sound::Sound;
use vmm_ident::Ident;

use super::{Decode, Encode, ProtocolError, VarInt};

#[derive(Debug, Clone, PartialEq)]
pub enum SoundId<'a> {
	Direct {
		id: Ident<Cow<'a, str>>,
		range: Option<f32>,
	},
	Reference {
		id: VarInt,
	},
}

impl<'a> Decode<'a> for SoundId<'a> {
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let i = VarInt::decode(r)?.0;

		if matches!(i, 0) {
			Ok(Self::Direct {
				id: Ident::decode(r)?,
				range: <Option<f32>>::decode(r)?,
			})
		} else {
			Ok(Self::Reference { id: VarInt(i - 1) })
		}
	}
}

impl Encode for SoundId<'_> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		match self {
			Self::Direct { id, range } => {
				VarInt(0).encode(&mut w)?;
				id.encode(&mut w)?;
				range.encode(&mut w)?;
			}
			Self::Reference { id } => VarInt(id.0 + 1).encode(&mut w)?,
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCategory {
	Master,
	Music,
	Record,
	Weather,
	Block,
	Hostile,
	Neutral,
	Player,
	Ambient,
	Voice,
}
