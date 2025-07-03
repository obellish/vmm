use serde::{
	de::{Error as DeError, Visitor}, forward_to_deserialize_any, Deserialize, Deserializer, Serialize, Serializer
};

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

pub(crate) struct TagAccess<D> {
	parent: Option<D>,
	state: usize,
	tag: Option<u64>,
}

impl<D> TagAccess<D> {
	pub const fn new(parent: D, tag: Option<u64>) -> Self {
		Self {
			parent: Some(parent),
			state: 0,
			tag,
		}
	}
}

impl<'de, D> Deserializer<'de> for &mut TagAccess<D>
where
	D: Deserializer<'de>,
{
	type Error = D::Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.state += 1;

		match self.state {
			1 => visitor.visit_str(match self.tag {
				Some(..) => "@@TAGGED@@",
				None => "@@UNTAGGED@@",
			}),
			_ => visitor.visit_u64(self.tag.unwrap()),
		}
	}

	forward_to_deserialize_any! {
		i8 i16 i32 i64 i128
		u8 u16 u32 u64 u128
		bool f32 f64
		char str string
		bytes byte_buf
		seq map
		struct tuple tuple_struct
		identifier ignored_any
		option unit unit_struct newtype_struct enum
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
