use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Saturating<T>(pub T);

impl<'de, T> Deserialize<'de> for Saturating<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		T::deserialize(deserializer).map(Self)
	}
}

impl<T: Serialize> Serialize for Saturating<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		T::serialize(&self.0, serializer)
	}
}
