use core::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, SeqAccess, Visitor},
	ser::SerializeSeq,
};

use super::SmallVec;

impl<'de, T, const N: usize> Deserialize<'de> for SmallVec<T, N>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_seq(SmallVecVisitor(PhantomData))
	}
}

impl<T: Serialize, const N: usize> Serialize for SmallVec<T, N> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut state = serializer.serialize_seq(Some(self.len()))?;

		self.iter()
			.try_for_each(|item| state.serialize_element(item))?;

		state.end()
	}
}

#[repr(transparent)]
struct SmallVecVisitor<T, const N: usize>(PhantomData<T>);

impl<'de, T, const N: usize> Visitor<'de> for SmallVecVisitor<T, N>
where
	T: Deserialize<'de>,
{
	type Value = SmallVec<T, N>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a sequence")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let len = seq.size_hint().unwrap_or(0);
		let mut values = SmallVec::new();

		values.try_reserve(len).map_err(DeError::custom)?;

		while let Some(value) = seq.next_element()? {
			values.push(value);
		}

		Ok(values)
	}
}
