use core::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Serialize,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::SerializeStruct,
};

use super::MmrPeaks;

const FIELDS: &[&str] = &["num_leaves", "peaks"];

impl<'de> Deserialize<'de> for MmrPeaks {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_struct("MmrPeaks", FIELDS, MmrPeaksVisitor)
	}
}

impl Serialize for MmrPeaks {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("MmrPeaks", 2)?;

		state.serialize_field("num_leaves", &self.num_leaves)?;
		state.serialize_field("peaks", &self.peaks)?;

		state.end()
	}
}

struct MmrPeaksVisitor;

impl<'de> Visitor<'de> for MmrPeaksVisitor {
	type Value = MmrPeaks;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct MmrPeaks")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(num_leaves) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				0,
				&"struct MmrPeaks with 2 elements",
			));
		};

		let Some(peaks) = seq.next_element()? else {
			return Err(DeError::invalid_length(
				1,
				&"struct MmrPeaks with 2 elements",
			));
		};

		Ok(MmrPeaks { num_leaves, peaks })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut num_leaves = None;
		let mut peaks = None;

		while let Some(key) = map.next_key::<MmrPeaksField>()? {
			match key {
				MmrPeaksField::NumLeaves => {
					if num_leaves.is_some() {
						return Err(DeError::duplicate_field("num_leaves"));
					}

					num_leaves = Some(map.next_value()?);
				}
				MmrPeaksField::Peaks => {
					if peaks.is_some() {
						return Err(DeError::duplicate_field("peaks"));
					}

					peaks = Some(map.next_value()?);
				}
				MmrPeaksField::Ignore => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(num_leaves) = num_leaves else {
			return Err(DeError::missing_field("num_leaves"));
		};

		let Some(peaks) = peaks else {
			return Err(DeError::missing_field("peaks"));
		};

		Ok(MmrPeaks { num_leaves, peaks })
	}
}

struct MmrPeaksFieldVisitor;

impl Visitor<'_> for MmrPeaksFieldVisitor {
	type Value = MmrPeaksField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("field identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			0 => MmrPeaksField::NumLeaves,
			1 => MmrPeaksField::Peaks,
			_ => MmrPeaksField::Ignore,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			"num_leaves" => MmrPeaksField::NumLeaves,
			"peaks" => MmrPeaksField::Peaks,
			_ => MmrPeaksField::Ignore,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(match v {
			b"num_leaves" => MmrPeaksField::NumLeaves,
			b"peaks" => MmrPeaksField::Peaks,
			_ => MmrPeaksField::Ignore,
		})
	}
}

enum MmrPeaksField {
	NumLeaves,
	Peaks,
	Ignore,
}

impl<'de> Deserialize<'de> for MmrPeaksField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_identifier(MmrPeaksFieldVisitor)
	}
}
