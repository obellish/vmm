use std::fmt::{Formatter, Result as FmtResult};

use serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	de::{Error as DeError, SeqAccess, Unexpected, Visitor, value::SeqAccessDeserializer},
	ser::SerializeSeq,
};

use super::{JavaCodePoint, JavaStr, JavaString};

impl<'de> Deserialize<'de> for JavaCodePoint {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(JavaCodePointVisitor)
	}
}

impl Serialize for JavaCodePoint {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self.as_char() {
			Some(ch) => ch.serialize(serializer),
			None => self.as_u32().serialize(serializer),
		}
	}
}

impl<'de: 'a, 'a> Deserialize<'de> for &'a JavaStr {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(JavaStrVisitor)
	}
}

impl Serialize for JavaStr {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		if let Ok(str) = self.as_str() {
			str.serialize(serializer)
		} else {
			let mut seq = serializer.serialize_seq(None)?;
			for ch in self.chars() {
				seq.serialize_element(&ch.as_u32())?;
			}
			seq.end()
		}
	}
}

impl<'de> Deserialize<'de> for JavaString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(JavaStringVisitor)
	}
}

impl Serialize for JavaString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		if let Ok(str) = self.as_str() {
			str.serialize(serializer)
		} else {
			let mut seq = serializer.serialize_seq(None)?;
			for ch in self.chars() {
				seq.serialize_element(&ch.as_u32())?;
			}
			seq.end()
		}
	}
}

struct JavaStringVisitor;

impl<'de> Visitor<'de> for JavaStringVisitor {
	type Value = JavaString;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a JavaString")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(JavaString::from(v))
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(JavaString::from(v))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match JavaStr::from_semi_utf8(v) {
			Ok(str) => Ok(str.to_owned()),
			Err(_) => Err(DeError::invalid_value(Unexpected::Bytes(v), &self)),
		}
	}

	fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		JavaString::from_semi_utf8(v)
			.map_err(|err| DeError::invalid_value(Unexpected::Bytes(&err.into_bytes()), &self))
	}

	fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let vec = Vec::<u8>::deserialize(SeqAccessDeserializer::new(seq))?;
		JavaString::from_semi_utf8(vec).map_err(|_| DeError::invalid_value(Unexpected::Seq, &self))
	}
}

struct JavaStrVisitor;

impl<'de> Visitor<'de> for JavaStrVisitor {
	type Value = &'de JavaStr;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a borrowed JavaStr")
	}

	fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(JavaStr::from_str(v))
	}

	fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		JavaStr::from_semi_utf8(v).map_err(|_| DeError::invalid_value(Unexpected::Bytes(v), &self))
	}
}

struct JavaCodePointVisitor;

impl Visitor<'_> for JavaCodePointVisitor {
	type Value = JavaCodePoint;

	fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
		formatter.write_str("a character")
	}

	fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_i32(v.into())
	}

	fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_i32(v.into())
	}

	fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		if v < 0 {
			Err(DeError::invalid_value(Unexpected::Signed(v.into()), &self))
		} else {
			self.visit_u32(v as u32)
		}
	}

	fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		if v < 0 {
			Err(DeError::invalid_value(Unexpected::Signed(v), &self))
		} else {
			self.visit_u64(v as u64)
		}
	}

	fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_u32(v.into())
	}

	fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		self.visit_u32(v.into())
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		JavaCodePoint::from_u32(v)
			.ok_or_else(|| DeError::invalid_value(Unexpected::Unsigned(v.into()), &self))
	}

	fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		match u32::try_from(v) {
			Ok(v) => self.visit_u32(v),
			Err(_) => Err(DeError::invalid_value(Unexpected::Unsigned(v), &self)),
		}
	}

	fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		Ok(JavaCodePoint::from_char(v))
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: DeError,
	{
		let mut iter = v.chars();
		match (iter.next(), iter.next()) {
			(Some(c), None) => Ok(JavaCodePoint::from_char(c)),
			_ => Err(DeError::invalid_value(Unexpected::Str(v), &self)),
		}
	}
}
