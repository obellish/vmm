use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::SimpleSmt;

const FIELDS: &[&str] = &["root", "leaves", "inner_nodes"];

impl<'de, const DEPTH: u8> Deserialize<'de> for SimpleSmt<DEPTH> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("SimpleSmt", FIELDS, SimpleSmtVisitor)
	}
}

impl<const DEPTH: u8> Serialize for SimpleSmt<DEPTH> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("SimpleSmt", 3)?;

		state.serialize_field("root", &self.root)?;
		state.serialize_field("leaves", &self.leaves)?;
		state.serialize_field("inner_nodes", &self.inner_nodes)?;

		state.end()
	}
}

struct SimpleSmtVisitor<const DEPTH: u8>;

impl<'de, const DEPTH: u8> Visitor<'de> for SimpleSmtVisitor<DEPTH> {
	type Value = SimpleSmt<DEPTH>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct SimpleSmt")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(root) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct SimpleSmt with 3 elements",
			));
		};

		let Some(leaves) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct SimpleSmt with 3 elements",
			));
		};

		let Some(inner_nodes) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				2,
				&"struct SimpleSmt with 3 elements",
			));
		};

		Ok(SimpleSmt {
			root,
			leaves,
			inner_nodes,
		})
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut root = None;
		let mut leaves = None;
		let mut inner_nodes = None;

		while let Some(key) = map.next_key::<SimpleSmtField>()? {
			match key {
				SimpleSmtField::Root => {
					if root.is_some() {
						return Err(DeError::duplicate_field("root"));
					}

					root = Some(map.next_value()?);
				}
				SimpleSmtField::Leaves => {
					if leaves.is_some() {
						return Err(DeError::duplicate_field("leaves"));
					}

					leaves = Some(map.next_value()?);
				}
				SimpleSmtField::InnerNodes => {
					if inner_nodes.is_some() {
						return Err(DeError::duplicate_field("inner_nodes"));
					}

					inner_nodes = Some(map.next_value()?);
				}
				SimpleSmtField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(root) = root else {
			return Err(DeError::missing_field("root"));
		};

		let Some(leaves) = leaves else {
			return Err(DeError::missing_field("leaves"));
		};

		let Some(inner_nodes) = inner_nodes else {
			return Err(DeError::missing_field("inner_nodes"));
		};

		Ok(SimpleSmt {
			root,
			leaves,
			inner_nodes,
		})
	}
}

struct SimpleSmtFieldVisitor;

impl Visitor<'_> for SimpleSmtFieldVisitor {
	type Value = SimpleSmtField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => SimpleSmtField::Root,
			1 => SimpleSmtField::Leaves,
			2 => SimpleSmtField::InnerNodes,
			_ => SimpleSmtField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"root" => SimpleSmtField::Root,
			"leaves" => SimpleSmtField::Leaves,
			"inner_nodes" => SimpleSmtField::InnerNodes,
			_ => SimpleSmtField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"root" => SimpleSmtField::Root,
			b"leaves" => SimpleSmtField::Leaves,
			b"inner_nodes" => SimpleSmtField::InnerNodes,
			_ => SimpleSmtField::Ignore,
		})
	}
}

enum SimpleSmtField {
	Root,
	Leaves,
	InnerNodes,
	Ignore,
}

impl<'de> Deserialize<'de> for SimpleSmtField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(SimpleSmtFieldVisitor)
	}
}
