use std::io::{ErrorKind, Read};

use super::{END_ID, Error, Result, deserializer::NbtReadHelper, tag::NbtTag};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct NbtCompound {
	pub child_tags: Vec<(String, NbtTag)>,
}

impl NbtCompound {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			child_tags: Vec::new(),
		}
	}

	pub fn skip_content<R: Read>(reader: &mut NbtReadHelper<R>) -> Result<()> {
		loop {
			let tag_id = match reader.try_get_u8_be().map_err(Error::from) {
				Ok(id) => id,
				Err(Error::Incomplete(err)) => match err.kind() {
					ErrorKind::UnexpectedEof => break,
					_ => return Err(Error::Incomplete(err)),
				},
				Err(e) => return Err(e.into()),
			};

			if matches!(tag_id, END_ID) {
				break;
			}

			let len = reader.try_get_u16_be()?;
			reader.skip_bytes(u64::from(len))?;

			NbtTag::skip_data(reader, tag_id)?;
		}

		Ok(())
	}
}
