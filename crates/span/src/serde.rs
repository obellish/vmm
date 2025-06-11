use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::{Span, SpanBound, SpanStartBound};

const FIELDS: &[&str] = &["start", "end"];

impl<'de, T, From, To> Deserialize<'de> for Span<T, From, To>
where
	T: Deserialize<'de>,
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_struct(
			"Span",
			FIELDS,
			SpanVisitor {
				value: PhantomData,
				from: PhantomData,
				to: PhantomData,
			},
		)
	}
}

impl<T: Serialize, From, To> Serialize for Span<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut state = serializer.serialize_struct("Span", 2)?;

		state.serialize_field("start", &self.start)?;
		state.serialize_field("end", &self.end)?;
		state.end()
	}
}

#[repr(transparent)]
struct SpanVisitor<T, From, To>
where
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	value: PhantomData<T>,
	from: PhantomData<From>,
	to: PhantomData<To>,
}

impl<'de, T, From, To> Visitor<'de> for SpanVisitor<T, From, To>
where
	T: Deserialize<'de>,
	From: ?Sized + SpanStartBound<T>,
	To: ?Sized + SpanBound<T>,
{
	type Value = Span<T, From, To>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct Span")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(start) = seq.next_element::<Option<T>>()? else {
			return Err(DeError::invalid_length(0, &"struct Span with 2 elements"));
		};

		let Some(end) = seq.next_element::<Option<T>>()? else {
			return Err(DeError::invalid_length(1, &"struct Span with 2 elements"));
		};

		Ok(Span {
			start,
			end,
			marker_from: PhantomData,
			marker_to: PhantomData,
		})
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
				SpanField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(start) = start else {
			return Err(DeError::missing_field("start"));
		};

		let Some(end) = end else {
			return Err(DeError::missing_field("end"));
		};

		Ok(Span {
			start,
			end,
			marker_from: PhantomData,
			marker_to: PhantomData,
		})
	}
}

struct SpanFieldVisitor;

impl Visitor<'_> for SpanFieldVisitor {
	type Value = SpanField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
		Ok(match v {
			0 => SpanField::Start,
			1 => SpanField::End,
			_ => SpanField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
		Ok(match v {
			"start" => SpanField::Start,
			"end" => SpanField::End,
			_ => SpanField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
		Ok(match v {
			b"start" => SpanField::Start,
			b"end" => SpanField::End,
			_ => SpanField::Ignore,
		})
	}
}

enum SpanField {
	Start,
	End,
	Ignore,
}

impl<'de> Deserialize<'de> for SpanField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_identifier(SpanFieldVisitor)
	}
}
