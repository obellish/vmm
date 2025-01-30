use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::PartialMerkleTree;

const FIELDS: &[&str] = &["max_depth", "nodes", "leaves"];

impl<'de> Deserialize<'de> for PartialMerkleTree {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("PartialMerkleTree", FIELDS, PartialMerkleTreeVisitor)
	}
}

impl Serialize for PartialMerkleTree {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("PartialMerkleTree", 3)?;

		state.serialize_field("max_depth", &self.max_depth)?;
		state.serialize_field("nodes", &self.nodes)?;
		state.serialize_field("leaves", &self.leaves)?;

		state.end()
	}
}

struct PartialMerkleTreeVisitor;

impl<'de> Visitor<'de> for PartialMerkleTreeVisitor {
	type Value = PartialMerkleTree;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct PartialMerkleTree")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(max_depth) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct PartialMerkleTree with 3 elements",
			));
		};

		let Some(nodes) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct PartialMerkleTree with 3 elements",
			));
		};

		let Some(leaves) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				2,
				&"struct PartialMerkleTree with 3 elements",
			));
		};

		Ok(PartialMerkleTree {
			max_depth,
			nodes,
			leaves,
		})
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut max_depth = None;
		let mut nodes = None;
		let mut leaves = None;

		while let Some(key) = map.next_key::<PartialMerkleTreeField>()? {
			match key {
				PartialMerkleTreeField::MaxDepth => {
					if max_depth.is_some() {
						return Err(DeError::duplicate_field("max_depth"));
					}

					max_depth = Some(map.next_value()?);
				}
				PartialMerkleTreeField::Nodes => {
					if nodes.is_some() {
						return Err(DeError::duplicate_field("nodes"));
					}

					nodes = Some(map.next_value()?);
				}
				PartialMerkleTreeField::Leaves => {
					if leaves.is_some() {
						return Err(DeError::duplicate_field("leaves"));
					}

					leaves = Some(map.next_value()?);
				}
				PartialMerkleTreeField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(max_depth) = max_depth else {
			return Err(DeError::missing_field("max_depth"));
		};

		let Some(nodes) = nodes else {
			return Err(DeError::missing_field("nodes"));
		};

		let Some(leaves) = leaves else {
			return Err(DeError::missing_field("leaves"));
		};

		Ok(PartialMerkleTree {
			max_depth,
			nodes,
			leaves,
		})
	}
}

struct PartialMerkleTreeFieldVisitor;

impl Visitor<'_> for PartialMerkleTreeFieldVisitor {
	type Value = PartialMerkleTreeField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => PartialMerkleTreeField::MaxDepth,
			1 => PartialMerkleTreeField::Nodes,
			2 => PartialMerkleTreeField::Leaves,
			_ => PartialMerkleTreeField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"max_depth" => PartialMerkleTreeField::MaxDepth,
			"nodes" => PartialMerkleTreeField::Nodes,
			"leaves" => PartialMerkleTreeField::Leaves,
			_ => PartialMerkleTreeField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"max_depth" => PartialMerkleTreeField::MaxDepth,
			b"nodes" => PartialMerkleTreeField::Nodes,
			b"leaves" => PartialMerkleTreeField::Leaves,
			_ => PartialMerkleTreeField::Ignore,
		})
	}
}

enum PartialMerkleTreeField {
	MaxDepth,
	Nodes,
	Leaves,
	Ignore,
}

impl<'de> Deserialize<'de> for PartialMerkleTreeField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(PartialMerkleTreeFieldVisitor)
	}
}
