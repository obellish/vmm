use alloc::{
	fmt::{Formatter, Result as FmtResult},
	string::String,
};
use core::marker::PhantomData;

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::SpanInclusive;

const FIELDS: &[&str] = &["start", "end"];

impl<'de, Idx> Deserialize<'de> for SpanInclusive<Idx>
where
	Idx: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let (start, end) =
			deserializer.deserialize_struct("SpanInclusive", FIELDS, SpanVisitor(PhantomData))?;

		Ok(Self::new(start, end))
	}
}

impl<Idx: Serialize> Serialize for SpanInclusive<Idx> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("SpanInclusive", 2)?;
		state.serialize_field("start", &self.start())?;
		state.serialize_field("end", &self.end())?;
		state.end()
	}
}

enum SpanField {
	Start,
	End,
}

impl<'de> Deserialize<'de> for SpanField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_identifier(SpanFieldVisitor)
	}
}

struct SpanFieldVisitor;

impl Visitor<'_> for SpanFieldVisitor {
	type Value = SpanField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("`start` or `end`")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match v {
			"start" => Ok(SpanField::Start),
			"end" => Ok(SpanField::End),
			_ => Err(DeError::unknown_field(v, FIELDS)),
		}
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match v {
			b"start" => Ok(SpanField::Start),
			b"end" => Ok(SpanField::End),
			_ => {
				let value = String::from_utf8_lossy(v);
				Err(DeError::unknown_field(&value, FIELDS))
			}
		}
	}
}

struct SpanVisitor<Idx>(PhantomData<Idx>);

impl<'de, Idx> Visitor<'de> for SpanVisitor<Idx>
where
	Idx: Deserialize<'de>,
{
	type Value = (Idx, Idx);

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a SpanInclusive")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(start) = seq.next_element()? else {
			return Err(DeError::invalid_length(0, &self));
		};

		let Some(end) = seq.next_element()? else {
			return Err(DeError::invalid_length(1, &self));
		};

		Ok((start, end))
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut start = None;
		let mut end = None;

		while let Some(key) = map.next_key()? {
			match key {
				SpanField::Start => {
					if start.is_some() {
						return Err(DeError::duplicate_field("start"));
					}
					start = Some(map.next_value()?);
				}
				SpanField::End => {
					if end.is_some() {
						return Err(DeError::duplicate_field("end"));
					}
					end = Some(map.next_value()?);
				}
			}
		}

		let Some(start) = start else {
			return Err(DeError::missing_field("start"));
		};

		let Some(end) = end else {
			return Err(DeError::missing_field("end"));
		};

		Ok((start, end))
	}
}
