use std::{any::TypeId, collections::BTreeMap};

use erased_discriminant::Discriminant;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, de::DeserializeSeed};

use super::{Error, Result, Value, format::*};

#[derive(Debug)]
pub struct Tracer {
	pub(crate) config: TracerConfig,
	pub(crate) registry: Registry,
	pub(crate) incomplete_enums: BTreeMap<String, EnumProgress>,
	pub(crate) discriminants: BTreeMap<(TypeId, VariantId<'static>), Discriminant>,
}

impl Tracer {
	#[must_use]
	pub const fn new(config: TracerConfig) -> Self {
		Self {
			config,
			registry: Registry::new(),
			incomplete_enums: BTreeMap::new(),
			discriminants: BTreeMap::new(),
		}
	}

	pub fn registry(self) -> Result<Registry> {
		let mut registry = self.registry;
		for (name, format) in &mut registry {
			format
				.normalize()
				.map_err(|_| Error::UnknownFormatInContainer(name.clone()))?;
		}

		if self.incomplete_enums.is_empty() {
			Ok(registry)
		} else {
			Err(Error::MissingVariants(
				self.incomplete_enums.into_keys().collect(),
			))
		}
	}

	#[must_use]
	pub fn registry_unchecked(self) -> Registry {
		let mut registry = self.registry;
		for format in registry.values_mut() {
			format.normalize().unwrap_or(());
		}

		registry
	}

	pub(crate) fn record_variant(
		&mut self,
		samples: &mut Samples,
		name: &'static str,
		variant_index: u32,
		variant_name: &'static str,
		variant: VariantFormat,
		variant_value: Value,
	) -> Result<(Format, Value)> {
		let mut variants = BTreeMap::new();
		variants.insert(variant_index, Named::new(variant_name.to_owned(), variant));

		let format = ContainerFormat::Enum(variants);
		let value = Value::Variant(variant_index, Box::new(variant_value));
		self.record_container(samples, name, format, value, false)
	}

	pub(crate) fn record_container(
		&mut self,
		samples: &mut Samples,
		name: &'static str,
		format: ContainerFormat,
		value: Value,
		record_value: bool,
	) -> Result<(Format, Value)> {
		self.registry.entry(name.to_owned()).unify(format)?;
		if record_value {
			samples.values.insert(name, value.clone());
		}

		Ok((Format::TypeName(name.to_owned()), value))
	}

	pub(crate) fn get_sample<'a, 'de>(
		&'a self,
		samples: &'de Samples,
		name: &'static str,
	) -> Option<(&'a ContainerFormat, &'de Value)> {
		let value = samples.value(name)?;

		let format = self.registry.get(name)?;

		Some((format, value))
	}
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Samples {
	pub(crate) values: BTreeMap<&'static str, Value>,
}

impl Samples {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			values: BTreeMap::new(),
		}
	}

	#[must_use]
	pub fn value(&self, name: &'static str) -> Option<&Value> {
		self.values.get(name)
	}
}

#[derive(Debug)]
pub struct TracerConfig {
	pub(crate) is_human_readable: bool,
	pub(crate) record_samples_for_newtype_structs: bool,
	pub(crate) record_samples_for_tuple_structs: bool,
	pub(crate) record_samples_for_structs: bool,
	pub(crate) default_bool_value: bool,
	pub(crate) default_u8_value: u8,
	pub(crate) default_u16_value: u16,
	pub(crate) default_u32_value: u32,
	pub(crate) default_u64_value: u64,
	pub(crate) default_u128_value: u128,
	pub(crate) default_i8_value: i8,
	pub(crate) default_i16_value: i16,
	pub(crate) default_i32_value: i32,
	pub(crate) default_i64_value: i64,
	pub(crate) default_i128_value: i128,
	pub(crate) default_f32_value: f32,
	pub(crate) default_f64_value: f64,
	pub(crate) default_char_value: char,
	pub(crate) default_borrowed_str_value: &'static str,
	pub(crate) default_string_value: String,
	pub(crate) default_borrowed_bytes_value: &'static [u8],
	pub(crate) default_byte_buf_value: Vec<u8>,
}

macro_rules! impl_default_value_setter {
	(const $method:ident, $ty:ty) => {
		#[must_use]
		pub const fn $method(mut self, value: $ty) -> Self {
			self.$method = value;
			self
		}
	};
	($method:ident, $ty:ty) => {
		#[must_use]
		pub fn $method(mut self, value: $ty) -> Self {
			self.$method = value;
			self
		}
	};
}

impl TracerConfig {
	impl_default_value_setter!(const default_bool_value, bool);

	impl_default_value_setter!(const default_u8_value, u8);

	impl_default_value_setter!(const default_u16_value, u16);

	impl_default_value_setter!(const default_u32_value, u32);

	impl_default_value_setter!(const default_u64_value, u64);

	impl_default_value_setter!(const default_u128_value, u128);

	impl_default_value_setter!(const default_i8_value, i8);

	impl_default_value_setter!(const default_i16_value, i16);

	impl_default_value_setter!(const default_i32_value, i32);

	impl_default_value_setter!(const default_i64_value, i64);

	impl_default_value_setter!(const default_i128_value, i128);

	impl_default_value_setter!(const default_f32_value, f32);

	impl_default_value_setter!(const default_f64_value, f64);

	impl_default_value_setter!(const default_char_value, char);

	impl_default_value_setter!(const default_borrowed_str_value, &'static str);

	impl_default_value_setter!(default_string_value, String);

	impl_default_value_setter!(const default_borrowed_bytes_value, &'static [u8]);

	impl_default_value_setter!(default_byte_buf_value, Vec<u8>);

	#[must_use]
	pub const fn new() -> Self {
		Self {
			is_human_readable: false,
			record_samples_for_newtype_structs: true,
			record_samples_for_tuple_structs: false,
			record_samples_for_structs: false,
			default_bool_value: false,
			default_u8_value: 0,
			default_u16_value: 0,
			default_u32_value: 0,
			default_u64_value: 0,
			default_u128_value: 0,
			default_i8_value: 0,
			default_i16_value: 0,
			default_i32_value: 0,
			default_i64_value: 0,
			default_i128_value: 0,
			default_f32_value: 0.0,
			default_f64_value: 0.0,
			default_char_value: 'A',
			default_borrowed_str_value: "",
			default_string_value: String::new(),
			default_borrowed_bytes_value: b"",
			default_byte_buf_value: Vec::new(),
		}
	}

	#[must_use]
	pub const fn human_readable(mut self, value: bool) -> Self {
		self.is_human_readable = value;
		self
	}

	#[must_use]
	pub const fn record_samples_for_newtype_structs(mut self, value: bool) -> Self {
		self.record_samples_for_newtype_structs = value;
		self
	}

	#[must_use]
	pub const fn record_samples_for_tuple_structs(mut self, value: bool) -> Self {
		self.record_samples_for_tuple_structs = value;
		self
	}

	#[must_use]
	pub const fn record_samples_for_structs(mut self, value: bool) -> Self {
		self.record_samples_for_structs = value;
		self
	}
}

impl Default for TracerConfig {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum EnumProgress {
	NamedVariantsRemaining,
	IndexedVariantsRemaining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum VariantId<'a> {
	Index(u32),
	Name(&'a str),
}

pub type Registry = BTreeMap<String, ContainerFormat>;
