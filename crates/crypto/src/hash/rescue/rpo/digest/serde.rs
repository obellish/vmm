use serde::{Deserialize, Serialize};

use super::RpoDigest;

impl<'de> Deserialize<'de> for RpoDigest {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<&str as Deserialize<'de>>::deserialize(deserializer)
			.and_then(|v| Self::try_from(v).map_err(serde::de::Error::custom))
	}
}

impl Serialize for RpoDigest {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		alloc::string::String::serialize(&(*self).into(), serializer)
	}
}
