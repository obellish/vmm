use std::fmt::{Formatter, Result as FmtResult};

use serde::de::{Deserializer as SerdeDeserializer, Error as DeError, Visitor as SerdeVisitor};

use super::param::Param;

pub struct Deserializer<D> {
	pub inner: D,
	pub red_zone: usize,
	pub stack_size: usize,
}

impl<D> Deserializer<D> {
	pub fn new(inner: D) -> Self {
		let default_params = Param::default();
		Self {
			inner,
			stack_size: default_params.stack_size,
			red_zone: default_params.red_zone,
		}
	}
}

struct Visitor<V> {
	inner: V,
	param: Param,
}

impl<V> Visitor<V> {
	pub const fn new(inner: V, param: Param) -> Self {
		Self { inner, param }
	}
}

impl<'de, V> SerdeVisitor<'de> for Visitor<V>
where
	V: SerdeVisitor<'de>,
{
	type Value = V::Value;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		self.inner.expecting(formatter)
	}

	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_bool(v)
	}

	fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i8(v)
	}

	fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i16(v)
	}

	fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i32(v)
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i64(v)
	}

	fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_i128(v)
	}

	fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u8(v)
	}

	fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u16(v)
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u32(v)
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u64(v)
	}

	fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_u128(v)
	}

	fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_f32(v)
	}

	fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_f64(v)
	}

	fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_char(v)
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_str(v)
	}

	fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_borrowed_str(v)
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_string(v)
	}

	fn visit_unit<E>(self) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_unit()
	}

	fn visit_none<E>(self) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.inner.visit_none()
	}

	fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: SerdeDeserializer<'de>,
	{
		stacker::maybe_grow(self.param.red_zone, self.param.stack_size, || {
			self.inner.visit_some(Deserializer {
				inner: deserializer,
				red_zone: self.param.red_zone,
				stack_size: self.param.stack_size,
			})
		})
	}
}
