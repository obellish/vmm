use serde::{Deserialize, Serialize};

use super::Blake3Digest;

impl<const N: usize> Serialize for Blake3Digest<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		alloc::string::String::serialize(&(*self).into(), serializer)
	}
}

impl<'de, const N: usize> Deserialize<'de> for Blake3Digest<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<&str as Deserialize<'de>>::deserialize(deserializer)
			.and_then(|v| Self::try_from(v).map_err(serde::de::Error::custom))
	}
}
