use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::{MerkleStore, StoreNode};
use crate::{hash::rpo::RpoDigest, utils::collections::KvMap};

impl<'de, T> Deserialize<'de> for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode> + Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<T as Deserialize<'de>>::deserialize(deserializer).map(|nodes| Self { nodes })
	}
}

impl<T> Serialize for MerkleStore<T>
where
	T: KvMap<RpoDigest, StoreNode> + Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		<T as Serialize>::serialize(&self.nodes, serializer)
	}
}

const STORE_NODE_FIELDS: &[&str] = &["left", "right"];

impl<'de> Deserialize<'de> for StoreNode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("StoreNode", STORE_NODE_FIELDS, StoreNodeVisitor)
	}
}

impl Serialize for StoreNode {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("StoreNode", 2)?;

		state.serialize_field("left", &self.left)?;
		state.serialize_field("right", &self.right)?;

		state.end()
	}
}

struct StoreNodeVisitor;

impl<'de> Visitor<'de> for StoreNodeVisitor {
	type Value = StoreNode;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct StoreNode")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(left) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct StoreNode with 2 elements",
			));
		};

		let Some(right) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct StoreNode with 2 elements",
			));
		};

		Ok(StoreNode { left, right })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut left = None;
		let mut right = None;

		while let Some(key) = map.next_key::<StoreNodeField>()? {
			match key {
				StoreNodeField::Left => {
					if left.is_some() {
						return Err(DeError::duplicate_field("left"));
					}

					left = Some(map.next_value()?);
				}
				StoreNodeField::Right => {
					if right.is_some() {
						return Err(DeError::duplicate_field("right"));
					}

					right = Some(map.next_value()?);
				}
				StoreNodeField::Ignore => {
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

		Ok(StoreNode { left, right })
	}
}

struct StoreNodeFieldVisitor;

impl Visitor<'_> for StoreNodeFieldVisitor {
	type Value = StoreNodeField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => StoreNodeField::Left,
			1 => StoreNodeField::Right,
			_ => StoreNodeField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"left" => StoreNodeField::Left,
			"right" => StoreNodeField::Right,
			_ => StoreNodeField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"left" => StoreNodeField::Left,
			b"right" => StoreNodeField::Right,
			_ => StoreNodeField::Ignore,
		})
	}
}

enum StoreNodeField {
	Left,
	Right,
	Ignore,
}

impl<'de> Deserialize<'de> for StoreNodeField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(StoreNodeFieldVisitor)
	}
}
