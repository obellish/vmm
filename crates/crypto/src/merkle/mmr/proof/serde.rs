use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::MmrProof;

const FIELDS: &[&str] = &["forest", "position", "merkle_path"];

impl<'de> Deserialize<'de> for MmrProof {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("MmrProof", FIELDS, MmrProofVisitor)
	}
}

impl Serialize for MmrProof {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("MmrProof", 3)?;

		state.serialize_field("forest", &self.forest)?;
		state.serialize_field("position", &self.position)?;
		state.serialize_field("merkle_path", &self.merkle_path)?;

		state.end()
	}
}

struct MmrProofVisitor;

impl<'de> Visitor<'de> for MmrProofVisitor {
	type Value = MmrProof;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct MmrProof")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(forest) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct MmrProof with 3 elements",
			));
		};

		let Some(position) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct MmrProof with 3 elements",
			));
		};

		let Some(merkle_path) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				2,
				&"struct MmrProof with 3 elements",
			));
		};

		Ok(MmrProof {
			forest,
			position,
			merkle_path,
		})
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut forest = None;
		let mut position = None;
		let mut merkle_path = None;

		while let Some(key) = map.next_key::<MmrProofField>()? {
			match key {
				MmrProofField::Forest => {
					if forest.is_some() {
						return Err(DeError::duplicate_field("forest"));
					}

					forest = Some(map.next_value()?);
				}
				MmrProofField::Position => {
					if position.is_some() {
						return Err(DeError::duplicate_field("position"));
					}

					position = Some(map.next_value()?);
				}
				MmrProofField::MerklePath => {
					if merkle_path.is_some() {
						return Err(DeError::duplicate_field("merkle_path"));
					}

					merkle_path = Some(map.next_value()?);
				}
				MmrProofField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(forest) = forest else {
			return Err(DeError::missing_field("forest"));
		};

		let Some(position) = position else {
			return Err(DeError::missing_field("position"));
		};

		let Some(merkle_path) = merkle_path else {
			return Err(DeError::missing_field("merkle_path"));
		};

		Ok(MmrProof {
			forest,
			position,
			merkle_path,
		})
	}
}

struct MmrProofFieldVisitor;

impl Visitor<'_> for MmrProofFieldVisitor {
	type Value = MmrProofField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => MmrProofField::Forest,
			1 => MmrProofField::Position,
			2 => MmrProofField::MerklePath,
			_ => MmrProofField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"forest" => MmrProofField::Forest,
			"position" => MmrProofField::Position,
			"merkle_path" => MmrProofField::MerklePath,
			_ => MmrProofField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"forest" => MmrProofField::Forest,
			b"position" => MmrProofField::Position,
			b"merkle_path" => MmrProofField::MerklePath,
			_ => MmrProofField::Ignore,
		})
	}
}

enum MmrProofField {
	Forest,
	Position,
	MerklePath,
	Ignore,
}

impl<'de> Deserialize<'de> for MmrProofField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(MmrProofFieldVisitor)
	}
}
