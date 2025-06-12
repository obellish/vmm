use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::{Excluded, Included, Span, SpanBound, SpanStartBound, Unbounded};

const FIELDS: &[&str] = &["start", "end"];

impl<'de, T, From, To> Deserialize<'de> for Span<T, From, To>
where
	From: Deserialize<'de> + SpanStartBound<T>,
	To: Deserialize<'de> + SpanBound<T>,
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

impl<T, From, To> Serialize for Span<T, From, To>
where
	From: Serialize + SpanStartBound<T>,
	To: Serialize + SpanBound<T>,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut state = serializer.serialize_struct("Span", 2)?;
		state.serialize_field("start", &self.start)?;
		state.serialize_field("end", &self.end)?;
		state.end()
	}
}

impl<'de, T: ?Sized> Deserialize<'de> for Unbounded<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_newtype_struct("Unbounded", UnboundedVisitor(PhantomData))
	}
}

impl<T: ?Sized> Serialize for Unbounded<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_newtype_struct("Unbounded", &self.0)
	}
}

impl<'de, T> Deserialize<'de> for Included<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_newtype_struct("Included", IncludedVisitor(PhantomData))
	}
}

impl<T: Serialize> Serialize for Included<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_newtype_struct("Included", &self.0)
	}
}

impl<'de, T> Deserialize<'de> for Excluded<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_newtype_struct("Excluded", ExcludedVisitor(PhantomData))
	}
}

impl<T: Serialize> Serialize for Excluded<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_newtype_struct("Excluded", &self.0)
	}
}

struct UnboundedVisitor<T: ?Sized>(PhantomData<T>);

impl<'de, T: ?Sized> Visitor<'de> for UnboundedVisitor<T> {
	type Value = Unbounded<T>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("tuple struct Unbounded")
	}

	fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		let field = Deserialize::deserialize(deserializer)?;

		Ok(Unbounded(field))
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(field) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"tuple struct Unbounded with 1 element",
			));
		};

		Ok(Unbounded(field))
	}
}

struct IncludedVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for IncludedVisitor<T>
where
	T: Deserialize<'de>,
{
	type Value = Included<T>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("tuple struct Included")
	}

	fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		let field = Deserialize::deserialize(deserializer)?;
		Ok(Included(field))
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(field) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"tuple struct Included with 1 element",
			));
		};

		Ok(Included(field))
	}
}

struct ExcludedVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for ExcludedVisitor<T>
where
	T: Deserialize<'de>,
{
	type Value = Excluded<T>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("tuple struct Excluded")
	}

	fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		let field = Deserialize::deserialize(deserializer)?;
		Ok(Excluded(field))
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(field) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"tuple struct Excluded with 1 field",
			));
		};

		Ok(Excluded(field))
	}
}

struct SpanVisitor<T, From, To> {
	value: PhantomData<T>,
	from: PhantomData<From>,
	to: PhantomData<To>,
}

impl<'de, T, From, To> Visitor<'de> for SpanVisitor<T, From, To>
where
	From: Deserialize<'de> + SpanStartBound<T>,
	To: Deserialize<'de> + SpanBound<T>,
{
	type Value = Span<T, From, To>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct Span")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(start) = seq.next_element()? else {
			return Err(DeError::invalid_length(0, &"struct Span with 2 elements"));
		};

		let Some(end) = seq.next_element()? else {
			return Err(DeError::invalid_length(1, &"struct Span with 2 elements"));
		};

		Ok(Span {
			start,
			end,
			marker: PhantomData,
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
			marker: PhantomData,
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
