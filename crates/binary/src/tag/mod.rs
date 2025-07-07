mod de;
mod ser;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

pub use self::{de::*, ser::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Captured<V>(pub Option<u64>, pub V);

impl<'de, V> Deserialize<'de> for Captured<V>
where
	V: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		match Internal::deserialize(deserializer)? {
			Internal::Tagged(t, v) => Ok(Self(Some(t), v)),
			Internal::Untagged(v) => Ok(Self(None, v)),
		}
	}
}

impl<V: Serialize> Serialize for Captured<V> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self.0 {
			Some(tag) => Internal::Tagged(tag, &self.1).serialize(serializer),
			None => Internal::Untagged(&self.1).serialize(serializer),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Required<V, const TAG: u64>(pub V);

impl<'de, V, const TAG: u64> Deserialize<'de> for Required<V, TAG>
where
	V: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		match Internal::deserialize(deserializer)? {
			Internal::Tagged(t, v) if t == TAG => Ok(Self(v)),
			_ => Err(DeError::custom("required tag not found")),
		}
	}
}

impl<V: Serialize, const TAG: u64> Serialize for Required<V, TAG> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		Internal::Tagged(TAG, &self.0).serialize(serializer)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "@@TAG@@")]
enum Internal<T> {
	#[serde(rename = "@@UNTAGGED@@")]
	Untagged(T),
	#[serde(rename = "@@TAGGED@@")]
	Tagged(u64, T),
}
