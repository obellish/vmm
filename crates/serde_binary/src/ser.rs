use serde::{
	Serialize,
	ser::{SerializeStruct, SerializeStructVariant, Serializer as SerdeSerializer},
};

use super::{Config, Error, Type, VarInt, io::Output};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serializer<O> {
	output: O,
	use_indices: bool,
}

impl<O> Serializer<O> {
	pub fn into_output(self) -> O {
		self.output
	}
}

impl<O: Output> Serializer<O> {
	pub fn new(output: O) -> Self {
		Self {
			output,
			use_indices: Config::default().use_indices,
		}
	}

	#[must_use]
	pub const fn use_indices(mut self, use_indices: bool) -> Self {
		self.use_indices = use_indices;
		self
	}
}

impl<'a, O: Output> SerdeSerializer for &'a mut Serializer<O> {
	type Error = Error;
	type Ok = ();
	type SerializeMap = Self;
	type SerializeSeq = Self;
	type SerializeStruct = StructSerializer<'a, O>;
	type SerializeStructVariant = StructSerializer<'a, O>;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
}

#[derive(Debug)]
pub struct StructSerializer<'a, O> {
	serializer: &'a mut Serializer<O>,
	field_index: u32,
}

impl<'a, O> StructSerializer<'a, O> {
	const fn new(serializer: &'a mut Serializer<O>) -> Self {
		Self {
			serializer,
			field_index: 0,
		}
	}
}

impl<O: Output> SerializeStruct for StructSerializer<'_, O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}

		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer.output.write_byte(Type::MapEnd.into())?;
		Ok(())
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}

impl<O: Output> SerializeStructVariant for StructSerializer<'_, O> {
	type Error = Error;
	type Ok = ();

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		if self.serializer.use_indices {
			self.field_index.serialize(&mut *self.serializer)?;
		} else {
			key.serialize(&mut *self.serializer)?;
		}

		self.field_index += 1;
		value.serialize(&mut *self.serializer)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		self.serializer
			.output
			.write_all(&[Type::MapEnd.into(); 2])?;
		Ok(())
	}

	fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
		self.field_index += 1;
		Ok(())
	}
}
