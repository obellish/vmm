use alloc::string::String;
use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use nom_locate::LocatedSpan;
use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, MapAccess, Visitor},
	ser::SerializeStruct,
};

use super::Ident;

const FIELDS: &[&str] = &["offset", "line", "fragment", "extra"];

impl<'a, 'de: 'a> Deserialize<'de> for Ident<'a> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_struct("Ident", FIELDS, IdentVisitor(PhantomData))
	}
}

impl Serialize for Ident<'_> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let fragment = self.0.into_fragment_and_extra();
		let mut state = serializer.serialize_struct("Ident", 4)?;
		state.serialize_field("offset", &self.offset())?;
		state.serialize_field("line", &self.line())?;
		state.serialize_field("fragment", &fragment.0)?;
		state.serialize_field("extra", &fragment.1)?;
		state.end()
	}
}

struct IdentVisitor<'a>(PhantomData<fn() -> Ident<'a>>);

impl<'a, 'de: 'a> Visitor<'de> for IdentVisitor<'a> {
	type Value = Ident<'de>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct Ident")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut offset = None;
		let mut line = None;
		let mut fragment = None;
		let mut extra = None;

		while let Some(key) = map.next_key::<String>()? {
			match &*key {
				"offset" => {
					if offset.is_some() {
						return Err(DeError::duplicate_field("offset"));
					}

					offset = Some(map.next_value()?);
				}
				"line" => {
					if line.is_some() {
						return Err(DeError::duplicate_field("line"));
					}

					line = Some(map.next_value()?);
				}
				"fragment" => {
					if fragment.is_some() {
						return Err(DeError::duplicate_field("fragment"));
					}

					fragment = Some(map.next_value()?);
				}
				"extra" => {
					if extra.is_some() {
						return Err(DeError::duplicate_field("extra"));
					}

					extra = Some(map.next_value()?);
				}
				_ => return Err(DeError::unknown_field(&key, FIELDS)),
			}
		}

		let Some(offset) = offset else {
			return Err(DeError::missing_field("offset"));
		};

		let Some(line) = line else {
			return Err(DeError::missing_field("line"));
		};

		let Some(fragment) = fragment else {
			return Err(DeError::missing_field("fragment"));
		};

		let Some(extra) = extra else {
			return Err(DeError::missing_field("extra"));
		};

		let located = unsafe { LocatedSpan::new_from_raw_offset(offset, line, fragment, extra) };

		Ok(Ident(located))
	}
}
