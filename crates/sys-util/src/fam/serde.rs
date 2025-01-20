use std::{
	fmt::{Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Serialize, Serializer,
	de::{Deserialize, Deserializer, Error as DeError, SeqAccess, Visitor},
	ser::SerializeTuple,
};

use super::{FamStruct, FamStructWrapper};

struct FamStructWrapperVisitor<X> {
	unused: PhantomData<X>,
}

impl<'de, X> Visitor<'de> for FamStructWrapperVisitor<X>
where
	X: Default + Deserialize<'de> + FamStruct,
	X::Entry: Deserialize<'de>,
{
	type Value = FamStructWrapper<X>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("struct FamStructWrapper")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let header: X = seq.next_element()?.ok_or_else(|| {
			DeError::invalid_length(0, &"struct FamStructWrapper with 2 elements")
		})?;

		let entries: Vec<X::Entry> = seq.next_element()?.ok_or_else(|| {
			DeError::invalid_length(1, &"struct FamStructWrapper with 2 elements")
		})?;

		if header.len() != entries.len() {
			let message = format!(
				"mismatch between length of FAM specified in FamStruct header ({}) and actual size of FAM ({})",
				header.len(),
				entries.len()
			);

			return Err(A::Error::custom(message));
		}

		let mut result = FamStructWrapper::from_entries(entries.as_slice())
			.map_err(|e| A::Error::custom(format!("{e:?}")))?;
		result.mem_allocator[0] = header;
		Ok(result)
	}
}

impl<'de, T> Deserialize<'de> for FamStructWrapper<T>
where
	T: Default + Deserialize<'de> + FamStruct,
	T::Entry: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_tuple(2, FamStructWrapperVisitor {
			unused: PhantomData,
		})
	}
}

impl<T> Serialize for FamStructWrapper<T>
where
	T: Default + FamStruct + Serialize,
	T::Entry: Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut s = serializer.serialize_tuple(2)?;
		s.serialize_element(self.as_fam_struct())?;
		s.serialize_element(self.as_slice())?;
		s.end()
	}
}
