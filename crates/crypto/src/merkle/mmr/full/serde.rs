use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::Mmr;

const FIELDS: &[&str] = &["forest", "nodes"];

impl<'de> Deserialize<'de> for Mmr {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("Mmr", FIELDS, MmrVisitor)
	}
}

impl Serialize for Mmr {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Mmr", 2)?;

		state.serialize_field("forest", &self.forest)?;
		state.serialize_field("nodes", &self.nodes)?;

		state.end()
	}
}

struct MmrVisitor;

impl<'de> Visitor<'de> for MmrVisitor {
	type Value = Mmr;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct Mmr")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(forest) = seq.next_element()? else {
			return Err(DeError::invalid_length(0, &"struct Mmr with 2 elements"));
		};

		let Some(nodes) = seq.next_element()? else {
			return Err(DeError::invalid_length(1, &"struct Mmr with 2 elements"));
		};

		Ok(Mmr { forest, nodes })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut forest = None;
		let mut nodes = None;

		while let Some(key) = map.next_key::<MmrField>()? {
			match key {
				MmrField::Forest => {
					if forest.is_some() {
						return Err(DeError::duplicate_field("forest"));
					}

					forest = Some(map.next_value()?);
				}
				MmrField::Nodes => {
					if nodes.is_some() {
						return Err(DeError::duplicate_field("nodes"));
					}

					nodes = Some(map.next_value()?);
				}
				MmrField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(forest) = forest else {
			return Err(DeError::missing_field("forest"));
		};

		let Some(nodes) = nodes else {
			return Err(DeError::missing_field("nodes"));
		};

		Ok(Mmr { forest, nodes })
	}
}

struct MmrFieldVisitor;

impl Visitor<'_> for MmrFieldVisitor {
	type Value = MmrField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => MmrField::Forest,
			1 => MmrField::Nodes,
			_ => MmrField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"forest" => MmrField::Forest,
			"nodes" => MmrField::Nodes,
			_ => MmrField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"forest" => MmrField::Forest,
			b"nodes" => MmrField::Nodes,
			_ => MmrField::Ignore,
		})
	}
}

enum MmrField {
	Forest,
	Nodes,
	Ignore,
}

impl<'de> Deserialize<'de> for MmrField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(MmrFieldVisitor)
	}
}
