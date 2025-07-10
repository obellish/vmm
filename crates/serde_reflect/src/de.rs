use std::{
	collections::{BTreeMap, btree_map::Entry},
	iter,
};

use erased_discriminant::Discriminant;
use serde::de::{
	DeserializeSeed, Deserializer as SerdeDeserializer, EnumAccess, IntoDeserializer, MapAccess,
	SeqAccess, VariantAccess, Visitor,
};

use super::{
	Error, Result, Samples, Tracer, VariantId,
	format::{ContainerFormat, ContainerFormatEntry, Format, FormatHolder, Named, VariantFormat},
};
use crate::{EnumProgress, value::IntoSeqDeserializer as _};

pub struct Deserializer<'a, 'de> {
	tracer: &'a mut Tracer,
	samples: &'de Samples,
	format: &'a mut Format,
}

impl<'a, 'de> Deserializer<'a, 'de> {
	pub(crate) const fn new(
		tracer: &'a mut Tracer,
		samples: &'de Samples,
		format: &'a mut Format,
	) -> Self {
		Self {
			tracer,
			samples,
			format,
		}
	}
}

impl<'de> SerdeDeserializer<'de> for Deserializer<'_, 'de> {
	type Error = Error;

	fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_any"))
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Bool)?;
		visitor.visit_bool(self.tracer.config.default_bool_value)
	}

	fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::I8)?;
		visitor.visit_i8(self.tracer.config.default_i8_value)
	}

	fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::I16)?;
		visitor.visit_i16(self.tracer.config.default_i16_value)
	}

	fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::I32)?;
		visitor.visit_i32(self.tracer.config.default_i32_value)
	}

	fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::I64)?;
		visitor.visit_i64(self.tracer.config.default_i64_value)
	}

	fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::I128)?;
		visitor.visit_i128(self.tracer.config.default_i128_value)
	}

	fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::U8)?;
		visitor.visit_u8(self.tracer.config.default_u8_value)
	}

	fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::U16)?;
		visitor.visit_u16(self.tracer.config.default_u16_value)
	}

	fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::U32)?;
		visitor.visit_u32(self.tracer.config.default_u32_value)
	}

	fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::U64)?;
		visitor.visit_u64(self.tracer.config.default_u64_value)
	}

	fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::U128)?;
		visitor.visit_u128(self.tracer.config.default_u128_value)
	}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::F32)?;
		visitor.visit_f32(self.tracer.config.default_f32_value)
	}

	fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::F64)?;
		visitor.visit_f64(self.tracer.config.default_f64_value)
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Char)?;
		visitor.visit_char(self.tracer.config.default_char_value)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Str)?;
		visitor.visit_borrowed_str(self.tracer.config.default_borrowed_str_value)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Str)?;
		visitor.visit_string(self.tracer.config.default_string_value.clone())
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Bytes)?;
		visitor.visit_borrowed_bytes(self.tracer.config.default_borrowed_bytes_value)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Bytes)?;
		visitor.visit_byte_buf(self.tracer.config.default_byte_buf_value.clone())
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut format = Format::unknown();
		self.format
			.unify(Format::Option(Box::new(format.clone())))?;

		if format.is_unknown() {
			let inner = Deserializer::new(self.tracer, self.samples, &mut format);
			visitor.visit_some(inner)
		} else {
			visitor.visit_none()
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::Unit)?;
		visitor.visit_unit()
	}

	fn deserialize_unit_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::TypeName(name.to_owned()))?;
		self.tracer
			.registry
			.entry(name.to_owned())
			.unify(ContainerFormat::Unit)?;

		visitor.visit_unit()
	}

	fn deserialize_newtype_struct<V>(
		self,
		name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::TypeName(name.to_owned()))?;
		if self.tracer.config.record_samples_for_newtype_structs
			&& let Some((format, sample)) = self.tracer.get_sample(self.samples, name)
		{
			return visitor
				.visit_newtype_struct(sample.into_deserializer())
				.map_err(|err| match err {
					Error::Deserialization(msg) => {
						let mut format = format.clone();
						format.reduce();
						Error::UnexpectedDeserializationFormat(name, format, msg)
					}
					_ => err,
				});
		}

		let mut format = Format::unknown();
		self.tracer
			.registry
			.entry(name.to_owned())
			.unify(ContainerFormat::Newtype(Box::new(format.clone())))?;

		let inner = Deserializer::new(self.tracer, self.samples, &mut format);
		visitor.visit_newtype_struct(inner)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut format = Format::unknown();
		self.format.unify(Format::Seq(Box::new(format.clone())))?;

		if format.is_unknown() {
			let inner = SeqDeserializer::new(self.tracer, self.samples, iter::once(&mut format));
			visitor.visit_seq(inner)
		} else {
			let inner = SeqDeserializer::new(self.tracer, self.samples, iter::empty());
			visitor.visit_seq(inner)
		}
	}

	fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut formats = iter::repeat_with(Format::unknown)
			.take(len)
			.collect::<Vec<_>>();
		self.format.unify(Format::Tuple(formats.clone()))?;
		let inner = SeqDeserializer::new(self.tracer, self.samples, formats.iter_mut());
		visitor.visit_seq(inner)
	}

	fn deserialize_tuple_struct<V>(
		self,
		name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::TypeName(name.to_owned()))?;
		if self.tracer.config.record_samples_for_tuple_structs
			&& let Some((format, sample)) = self.tracer.get_sample(self.samples, name)
		{
			let result = || visitor.visit_seq(sample.seq_values()?.into_seq_deserializer());
			return result().map_err(|err| match err {
				Error::Deserialization(msg) => {
					let mut format = format.clone();
					format.reduce();
					Error::UnexpectedDeserializationFormat(name, format, msg)
				}
				_ => err,
			});
		}

		let mut formats = iter::repeat_with(Format::unknown)
			.take(len)
			.collect::<Vec<_>>();
		self.tracer
			.registry
			.entry(name.to_owned())
			.unify(ContainerFormat::Tuple(formats.clone()))?;

		let inner = SeqDeserializer::new(self.tracer, self.samples, formats.iter_mut());
		visitor.visit_seq(inner)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut key_format = Format::unknown();
		let mut value_format = Format::unknown();
		self.format.unify(Format::Map {
			key: Box::new(key_format.clone()),
			value: Box::new(value_format.clone()),
		})?;

		if key_format.is_unknown() || value_format.is_unknown() {
			let inner = SeqDeserializer::new(
				self.tracer,
				self.samples,
				[&mut key_format, &mut value_format].into_iter(),
			);
			visitor.visit_map(inner)
		} else {
			let inner = SeqDeserializer::new(self.tracer, self.samples, iter::empty());
			visitor.visit_map(inner)
		}
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		self.format.unify(Format::TypeName(name.to_owned()))?;
		if self.tracer.config.record_samples_for_structs
			&& let Some((format, sample)) = self.tracer.get_sample(self.samples, name)
		{
			let result = || visitor.visit_seq(sample.seq_values()?.into_seq_deserializer());
			return result().map_err(|err| match err {
				Error::Deserialization(msg) => {
					let mut format = format.clone();
					format.reduce();
					Error::UnexpectedDeserializationFormat(name, format, msg)
				}
				_ => err,
			});
		}

		let mut formats = fields
			.iter()
			.map(|name| Named::new(name.to_owned(), Format::unknown()))
			.collect::<Vec<_>>();

		self.tracer
			.registry
			.entry(name.to_owned())
			.unify(ContainerFormat::Struct(formats.clone()))?;

		let inner = SeqDeserializer::new(
			self.tracer,
			self.samples,
			formats.iter_mut().map(Named::value_mut),
		);

		visitor.visit_seq(inner)
	}

	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		if variants.is_empty() {
			return Err(Error::NotSupported("deserialize_enum with 0 variants"));
		}

		let enum_type_id = typeid::of::<V::Value>();
		self.format.unify(Format::TypeName(name.to_owned()))?;

		self.tracer
			.registry
			.entry(name.to_owned())
			.unify(ContainerFormat::Enum(BTreeMap::new()))?;

		let Some(ContainerFormat::Enum(known_variants)) = self.tracer.registry.get_mut(name) else {
			unreachable!()
		};

		if self.tracer.incomplete_enums.contains_key(name) {
			return visitor.visit_enum(EnumDeserializer::new(
				self.tracer,
				self.samples,
				VariantId::Index(0),
				&mut VariantFormat::unknown(),
			));
		}

		let provisional_min = u32::MAX - (variants.len() - 1) as u32;
		for (i, &variant_name) in variants.iter().enumerate() {
			if self
				.tracer
				.discriminants
				.contains_key(&(enum_type_id, VariantId::Name(variant_name)))
			{
				continue;
			}

			let provisional_index = provisional_min + i as u32;
			let variant = known_variants
				.entry(provisional_index)
				.or_insert_with(|| Named::new(variant_name.to_owned(), VariantFormat::unknown()));

			self.tracer
				.incomplete_enums
				.insert(name.to_owned(), EnumProgress::NamedVariantsRemaining);

			let mut value = variant.value().clone();

			let enum_value = visitor.visit_enum(EnumDeserializer::new(
				self.tracer,
				self.samples,
				VariantId::Name(variant_name),
				&mut value,
			))?;

			let discriminant = Discriminant::of(&enum_value);
			self.tracer
				.discriminants
				.insert((enum_type_id, VariantId::Name(variant_name)), discriminant);
			return Ok(enum_value);
		}

		let mut index = 0;
		if known_variants.range(provisional_min..).next().is_some() {
			self.tracer
				.incomplete_enums
				.insert(name.to_owned(), EnumProgress::IndexedVariantsRemaining);

			while known_variants.contains_key(&index)
				&& self
					.tracer
					.discriminants
					.contains_key(&(enum_type_id, VariantId::Index(index)))
			{
				index += 1;
			}
		}

		let mut value = VariantFormat::unknown();
		let enum_value = visitor.visit_enum(EnumDeserializer::new(
			self.tracer,
			self.samples,
			VariantId::Index(index),
			&mut value,
		))?;

		let discriminant = Discriminant::of(&enum_value);
		self.tracer.discriminants.insert(
			(enum_type_id, VariantId::Index(index)),
			discriminant.clone(),
		);

		let Some(ContainerFormat::Enum(known_variants)) = self.tracer.registry.get_mut(name) else {
			unreachable!()
		};

		let mut has_indexed_variants_remaining = false;
		for provisional_index in provisional_min..=u32::MAX {
			if let Entry::Occupied(provisional_entry) = known_variants.entry(provisional_index) {
				if self.tracer.discriminants[&(
					enum_type_id,
					VariantId::Name(provisional_entry.get().name()),
				)] == discriminant
				{
					let provisional_entry = provisional_entry.remove();
					match known_variants.entry(index) {
						Entry::Vacant(vacant) => {
							vacant.insert(provisional_entry);
						}
						Entry::Occupied(mut existing_entry) => {
							existing_entry
								.get_mut()
								.value_mut()
								.unify(provisional_entry.value().clone())?;
						}
					}
				} else {
					has_indexed_variants_remaining = true;
				}
			}
		}

		if let Some(existing_entry) = known_variants.get_mut(&index) {
			existing_entry.value_mut().unify(value)?;
		}

		if has_indexed_variants_remaining {
			self.tracer
				.incomplete_enums
				.insert(name.to_owned(), EnumProgress::IndexedVariantsRemaining);
		} else {
			self.tracer.incomplete_enums.remove(name);
		}

		Ok(enum_value)
	}

	fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_identifier"))
	}

	fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		Err(Error::NotSupported("deserialize_ignored_any"))
	}

	fn is_human_readable(&self) -> bool {
		self.tracer.config.is_human_readable
	}
}

pub struct SeqDeserializer<'a, 'de, I> {
	tracer: &'a mut Tracer,
	samples: &'de Samples,
	formats: I,
}

impl<'a, 'de, I> SeqDeserializer<'a, 'de, I> {
	const fn new(tracer: &'a mut Tracer, samples: &'de Samples, formats: I) -> Self {
		Self {
			tracer,
			samples,
			formats,
		}
	}
}

impl<'a, 'de, I> MapAccess<'de> for SeqDeserializer<'a, 'de, I>
where
	I: Iterator<Item = &'a mut Format>,
{
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
	where
		K: DeserializeSeed<'de>,
	{
		let Some(format) = self.formats.next() else {
			return Ok(None);
		};

		let inner = Deserializer::new(self.tracer, self.samples, format);
		seed.deserialize(inner).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let Some(format) = self.formats.next() else {
			unreachable!()
		};

		let inner = Deserializer::new(self.tracer, self.samples, format);
		seed.deserialize(inner)
	}
}

impl<'a, 'de, I> SeqAccess<'de> for SeqDeserializer<'a, 'de, I>
where
	I: Iterator<Item = &'a mut Format>,
{
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		let Some(format) = self.formats.next() else {
			return Ok(None);
		};

		let inner = Deserializer::new(self.tracer, self.samples, format);
		seed.deserialize(inner).map(Some)
	}

	fn size_hint(&self) -> Option<usize> {
		self.formats.size_hint().1
	}
}

pub struct EnumDeserializer<'a, 'de> {
	tracer: &'a mut Tracer,
	samples: &'de Samples,
	variant_id: VariantId<'static>,
	format: &'a mut VariantFormat,
}

impl<'a, 'de> EnumDeserializer<'a, 'de> {
	const fn new(
		tracer: &'a mut Tracer,
		samples: &'de Samples,
		variant_id: VariantId<'static>,
		format: &'a mut VariantFormat,
	) -> Self {
		Self {
			tracer,
			samples,
			variant_id,
			format,
		}
	}
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'_, 'de> {
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let value = match self.variant_id {
			VariantId::Index(index) => seed.deserialize(index.into_deserializer()),
			VariantId::Name(name) => seed.deserialize(name.into_deserializer()),
		}?;

		Ok((value, self))
	}
}

impl<'de> VariantAccess<'de> for EnumDeserializer<'_, 'de> {
	type Error = Error;

	fn unit_variant(self) -> Result<(), Self::Error> {
		self.format.unify(VariantFormat::Unit)
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		let mut format = Format::unknown();
		self.format
			.unify(VariantFormat::Newtype(Box::new(format.clone())))?;
		let inner = Deserializer::new(self.tracer, self.samples, &mut format);
		seed.deserialize(inner)
	}

	fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut formats = std::iter::repeat_with(Format::unknown)
			.take(len)
			.collect::<Vec<_>>();
		self.format.unify(VariantFormat::Tuple(formats.clone()))?;
		let inner = SeqDeserializer::new(self.tracer, self.samples, formats.iter_mut());
		visitor.visit_seq(inner)
	}

	fn struct_variant<V>(
		self,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: Visitor<'de>,
	{
		let mut formats = fields
			.iter()
			.map(|name| Named::new(name.to_owned(), Format::unknown()))
			.collect::<Vec<_>>();
		self.format.unify(VariantFormat::Struct(formats.clone()))?;

		let inner = SeqDeserializer::new(
			self.tracer,
			self.samples,
			formats.iter_mut().map(Named::value_mut),
		);

		visitor.visit_seq(inner)
	}
}
