use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::InnerNodeInfo;

impl Serialize for InnerNodeInfo {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("InnerNodeInfo", 3)?;

		state.serialize_field("value", &self.value)?;
		state.serialize_field("left", &self.left)?;
		state.serialize_field("right", &self.right)?;

		state.end()
	}
}

const FIELDS: &[&str] = &["value", "left", "right"];

impl<'de> Deserialize<'de> for InnerNodeInfo {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("InnerNodeInfo", FIELDS, InnerNodeInfoVisitor)
	}
}

struct InnerNodeInfoVisitor;

impl<'de> Visitor<'de> for InnerNodeInfoVisitor {
	type Value = InnerNodeInfo;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct InnerNodeInfo")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(value) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct InnerNodeInfo with 3 elements",
			));
		};

		let Some(left) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct InnerNodeInfo with 3 elements",
			));
		};

		let Some(right) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				2,
				&"struct InnerNodeInfo with 3 elements",
			));
		};

		Ok(InnerNodeInfo { value, left, right })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut value = None;
		let mut left = None;
		let mut right = None;

		while let Some(key) = map.next_key::<InnerNodeInfoField>()? {
			match key {
				InnerNodeInfoField::Value => {
					if value.is_some() {
						return Err(DeError::duplicate_field("value"));
					}

					value = Some(map.next_value()?);
				}
				InnerNodeInfoField::Left => {
					if left.is_some() {
						return Err(DeError::duplicate_field("left"));
					}

					left = Some(map.next_value()?);
				}
				InnerNodeInfoField::Right => {
					if right.is_some() {
						return Err(DeError::duplicate_field("right"));
					}

					right = Some(map.next_value()?);
				}
				InnerNodeInfoField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(value) = value else {
			return Err(DeError::missing_field("value"));
		};

		let Some(left) = left else {
			return Err(DeError::missing_field("left"));
		};

		let Some(right) = right else {
			return Err(DeError::missing_field("right"));
		};

		Ok(InnerNodeInfo { value, left, right })
	}
}

struct InnerNodeInfoFieldVisitor;

impl Visitor<'_> for InnerNodeInfoFieldVisitor {
	type Value = InnerNodeInfoField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => InnerNodeInfoField::Value,
			1 => InnerNodeInfoField::Left,
			2 => InnerNodeInfoField::Right,
			_ => InnerNodeInfoField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"value" => InnerNodeInfoField::Value,
			"left" => InnerNodeInfoField::Left,
			"right" => InnerNodeInfoField::Right,
			_ => InnerNodeInfoField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"value" => InnerNodeInfoField::Value,
			b"left" => InnerNodeInfoField::Left,
			b"right" => InnerNodeInfoField::Right,
			_ => InnerNodeInfoField::Ignore,
		})
	}
}

enum InnerNodeInfoField {
	Value,
	Left,
	Right,
	Ignore,
}

impl<'de> Deserialize<'de> for InnerNodeInfoField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(InnerNodeInfoFieldVisitor)
	}
}
