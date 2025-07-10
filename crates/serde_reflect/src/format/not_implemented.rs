use serde::{Deserializer, Serializer, de::Error as _, ser::Error as _};

pub fn serialize<T, S: Serializer>(_: &T, _: S) -> Result<S::Ok, S::Error> {
	Err(S::Error::custom("cannot serialize variables"))
}

pub fn deserialize<'de, T, D>(_: D) -> Result<T, D::Error>
where
	D: Deserializer<'de>,
{
	Err(D::Error::custom("cannot deserialize variables"))
}
