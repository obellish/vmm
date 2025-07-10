use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	marker::PhantomData,
};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor},
	ser::{SerializeMap, SerializeStruct},
};

use super::{FormatHolder, unification_error};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Named<T> {
	name: String,
	value: T,
}

impl<T> Named<T> {
	pub fn new(name: impl Into<String>, value: T) -> Self {
		Self {
			name: name.into(),
			value,
		}
	}

	pub const fn name(&self) -> &String {
		&self.name
	}

	pub const fn name_mut(&mut self) -> &mut String {
		&mut self.name
	}

	pub const fn value(&self) -> &T {
		&self.value
	}

	pub const fn value_mut(&mut self) -> &mut T {
		&mut self.value
	}

	pub fn into_inner(self) -> T {
		self.value
	}

	pub fn into_parts(self) -> (String, T) {
		(self.name, self.value)
	}
}

impl<'de, T> Deserialize<'de> for Named<T>
where
	T: Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		if deserializer.is_human_readable() {
			deserializer.deserialize_map(NamedVisitor(PhantomData))
		} else {
			deserializer.deserialize_struct(
				"Named",
				&["name", "value"],
				InternalNamedVisitor(PhantomData),
			)
		}
	}
}

impl<T> FormatHolder for Named<T>
where
	T: Debug + FormatHolder,
{
	fn visit<'a>(
		&'a self,
		f: &mut dyn FnMut(&'a super::Format) -> crate::Result<()>,
	) -> crate::Result<()> {
		self.value.visit(f)
	}

	fn visit_mut(
		&mut self,
		f: &mut dyn FnMut(&mut super::Format) -> crate::Result<()>,
	) -> crate::Result<()> {
		self.value.visit_mut(f)
	}

	fn unify(&mut self, other: Self) -> crate::Result<()> {
		if self.name != other.name {
			return Err(unification_error(&*self, &other));
		}

		self.value.unify(other.value)
	}

	fn is_unknown(&self) -> bool {
		false
	}
}

impl<T: Serialize> Serialize for Named<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
		if serializer.is_human_readable() {
			let mut map = serializer.serialize_map(Some(1))?;
			map.serialize_entry(&self.name, &self.value)?;
			map.end()
		} else {
			let mut inner = serializer.serialize_struct("Named", 2)?;
			inner.serialize_field("name", &self.name)?;
			inner.serialize_field("value", &self.value)?;
			inner.end()
		}
	}
}

struct NamedVisitor<T: ?Sized>(PhantomData<T>);

impl<'de, T> Visitor<'de> for NamedVisitor<T>
where
	T: Deserialize<'de>,
{
	type Value = Named<T>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a single entry map")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let named_value = match map.next_entry::<String, T>()? {
			Some((name, value)) => Named { name, value },
			_ => return Err(DeError::custom("missing entry")),
		};

		if map.next_entry::<String, T>()?.is_some() {
			return Err(DeError::custom("too many entries"));
		}

		Ok(named_value)
	}
}

#[repr(transparent)]
struct InternalNamedVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for InternalNamedVisitor<T>
where
	T: Deserialize<'de>,
{
	type Value = Named<T>;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a named value")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let Some(name) = seq.next_element()? else {
			return Err(DeError::invalid_length(0, &self));
		};

		let Some(value) = seq.next_element()? else {
			return Err(DeError::invalid_length(1, &self));
		};

		Ok(Named { name, value })
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut name = None;
		let mut value = None;

		while let Some(key) = map.next_key()? {
			match key {
				NamedField::Name => {
					if name.is_some() {
						return Err(DeError::duplicate_field("name"));
					}

					name = Some(map.next_value()?);
				}
				NamedField::Value => {
					if value.is_some() {
						return Err(DeError::duplicate_field("value"));
					}

					value = Some(map.next_value()?);
				}
				NamedField::Ignored => {
					_ = map.next_value::<IgnoredAny>()?;
				}
			}
		}

		let Some(name) = name else {
			return Err(DeError::missing_field("name"));
		};

		let Some(value) = value else {
			return Err(DeError::missing_field("value"));
		};

		Ok(Named { name, value })
	}
}

struct NamedFieldVisitor;

impl Visitor<'_> for NamedFieldVisitor {
	type Value = NamedField;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("an identifier")
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
		Ok(match v {
			0 => NamedField::Name,
			1 => NamedField::Value,
			_ => NamedField::Ignored,
		})
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
		Ok(match v {
			"name" => NamedField::Name,
			"value" => NamedField::Value,
			_ => NamedField::Ignored,
		})
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
		Ok(match v {
			b"name" => NamedField::Name,
			b"value" => NamedField::Value,
			_ => NamedField::Ignored,
		})
	}
}

enum NamedField {
	Name,
	Value,
	Ignored,
}

impl<'de> Deserialize<'de> for NamedField {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_identifier(NamedFieldVisitor)
	}
}
