use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

use super::{Length, SmallLength};

impl<'de> Deserialize<'de> for Length {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(|index| Self { index })
	}
}

impl Serialize for Length {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		Serialize::serialize(&self.index, serializer)
	}
}

impl<'de> Deserialize<'de> for SmallLength {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let content = serde::__private::de::Content::deserialize(deserializer)?;
		let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);
		if let Ok(value) = <u8 as Deserialize>::deserialize(deserializer) {
			return Ok(Self::Byte(value));
		}

		if let Ok(value) = <u16 as Deserialize>::deserialize(deserializer) {
			return Ok(Self::Word(value));
		}

		if let Ok(value) = <u32 as Deserialize>::deserialize(deserializer) {
			return Ok(Self::Double(value));
		}

		if let Ok(value) = <u64 as Deserialize>::deserialize(deserializer) {
			return Ok(Self::Quad(value));
		}

		Err(DeError::custom(
			"data did not match any variant of untagged enum SmallLength",
		))
	}
}

impl Serialize for SmallLength {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match *self {
			Self::Byte(byte) => serializer.serialize_u8(byte),
			Self::Word(word) => serializer.serialize_u16(word),
			Self::Double(double) => serializer.serialize_u32(double),
			Self::Quad(quad) => serializer.serialize_u64(quad),
		}
	}
}
