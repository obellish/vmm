use serde::{
	Deserializer,
	de::{DeserializeSeed, EnumAccess, Error as DeError, SeqAccess, VariantAccess, Visitor},
	forward_to_deserialize_any,
};

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
}

impl<'de, D> EnumAccess<'de> for TagAccess<D>
where
	D: Deserializer<'de>,
{
	type Error = D::Error;
	type Variant = Self;

	fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let variant = seed.deserialize(&mut self)?;
		Ok((variant, self))
	}
}

impl<'de, D> SeqAccess<'de> for TagAccess<D>
where
	D: Deserializer<'de>,
{
	type Error = D::Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		if self.state < 2 {
			return Ok(Some(seed.deserialize(self)?));
		}

		Ok(match self.parent.take() {
			Some(x) => Some(seed.deserialize(x)?),
			None => None,
		})
	}
}

impl<'de, D> VariantAccess<'de> for TagAccess<D>
where
	D: Deserializer<'de>,
{
	type Error = D::Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		Err(DeError::custom("expected tag"))
	}

	fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		seed.deserialize(self.parent.take().unwrap())
	}

	fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		visitor.visit_seq(self)
	}

	fn struct_variant<V>(self, _: &'static [&'static str], _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(DeError::custom("expected tag"))
	}
}
