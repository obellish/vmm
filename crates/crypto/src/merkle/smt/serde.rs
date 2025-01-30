use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::{InnerNode, LeafIndex};

impl<'de, const DEPTH: u8> Deserialize<'de> for LeafIndex<DEPTH> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(|index| Self { index })
	}
}

impl<const DEPTH: u8> Serialize for LeafIndex<DEPTH> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		Serialize::serialize(&self.index, serializer)
	}
}

const INNER_NODE_FIELDS: &[&str] = &["left", "right"];

impl<'de> Deserialize<'de> for InnerNode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("InnerNode", INNER_NODE_FIELDS, InnerNodeVisitor)
	}
}

impl Serialize for InnerNode {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("InnerNode", 2)?;

		state.serialize_field("left", &self.left)?;
		state.serialize_field("right", &self.right)?;

		state.end()
	}
}

struct InnerNodeVisitor;

impl<'de> Visitor<'de> for InnerNodeVisitor {
	type Value = InnerNode;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct InnerNode")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(left) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct InnerNode with 2 elements",
			));
		};

		let Some(right) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct InnerNode with 2 elements",
			));
		};

		Ok(InnerNode { left, right })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut left = None;
		let mut right = None;

		while let Some(key) = map.next_key::<InnerNodeField>()? {
			match key {
				InnerNodeField::Left => {
					if left.is_some() {
						return Err(DeError::duplicate_field("left"));
					}

					left = Some(map.next_value()?);
				}
				InnerNodeField::Right => {
					if right.is_some() {
						return Err(DeError::duplicate_field("right"));
					}

					right = Some(map.next_value()?);
				}
				InnerNodeField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(left) = left else {
			return Err(DeError::missing_field("left"));
		};

		let Some(right) = right else {
			return Err(DeError::missing_field("right"));
		};

		Ok(InnerNode { left, right })
	}
}

struct InnerNodeFieldVisitor;

impl Visitor<'_> for InnerNodeFieldVisitor {
	type Value = InnerNodeField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => InnerNodeField::Left,
			1 => InnerNodeField::Right,
			_ => InnerNodeField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"left" => InnerNodeField::Left,
			"right" => InnerNodeField::Right,
			_ => InnerNodeField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"left" => InnerNodeField::Left,
			b"right" => InnerNodeField::Right,
			_ => InnerNodeField::Ignore,
		})
	}
}

enum InnerNodeField {
	Left,
	Right,
	Ignore,
}

impl<'de> Deserialize<'de> for InnerNodeField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(InnerNodeFieldVisitor)
	}
}
