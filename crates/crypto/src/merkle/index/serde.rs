use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::NodeIndex;

const FIELDS: &[&str] = &["depth", "value"];

impl<'de> Deserialize<'de> for NodeIndex {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("NodeIndex", FIELDS, NodeIndexVisitor {
			marker: PhantomData,
			lifetime: PhantomData,
		})
	}
}

impl Serialize for NodeIndex {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("NodeIndex", 2)?;

		state.serialize_field("depth", &self.depth)?;
		state.serialize_field("value", &self.value)?;
		state.end()
	}
}

struct NodeIndexVisitor<'de> {
	marker: PhantomData<NodeIndex>,
	lifetime: PhantomData<&'de ()>,
}

impl<'de> Visitor<'de> for NodeIndexVisitor<'de> {
	type Value = NodeIndex;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct NodeIndex")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(depth) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct NodeIndex with 2 elements",
			));
		};

		let Some(value) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct NodeIndex with 2 elements",
			));
		};

		Ok(NodeIndex { depth, value })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut depth = None;
		let mut value = None;

		while let Some(key) = map.next_key::<NodeIndexField>()? {
			match key {
				NodeIndexField::Depth => {
					if depth.is_some() {
						return Err(DeError::duplicate_field("depth"));
					}

					depth = Some(map.next_value()?);
				}
				NodeIndexField::Value => {
					if value.is_some() {
						return Err(DeError::duplicate_field("value"));
					}

					value = Some(map.next_value()?);
				}
				NodeIndexField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(depth) = depth else {
			return Err(DeError::missing_field("depth"));
		};

		let Some(value) = value else {
			return Err(DeError::missing_field("value"));
		};

		Ok(NodeIndex { depth, value })
	}
}

enum NodeIndexField {
	Depth,
	Value,
	Ignore,
}

impl<'de> Deserialize<'de> for NodeIndexField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(NodeIndexFieldVisitor)
	}
}

struct NodeIndexFieldVisitor;

impl Visitor<'_> for NodeIndexFieldVisitor {
	type Value = NodeIndexField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match v {
			0 => Ok(NodeIndexField::Depth),
			1 => Ok(NodeIndexField::Value),
			_ => Ok(NodeIndexField::Ignore),
		}
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match v {
			"depth" => Ok(NodeIndexField::Depth),
			"value" => Ok(NodeIndexField::Value),
			_ => Ok(NodeIndexField::Ignore),
		}
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match v {
			b"depth" => Ok(NodeIndexField::Depth),
			b"value" => Ok(NodeIndexField::Value),
			_ => Ok(NodeIndexField::Ignore),
		}
	}
}
