#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::{
	borrow::{Borrow, Cow},
	cmp::Ordering,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;
#[doc(hidden)]
pub use vmm_ident_macros::parse_ident_str;

#[macro_export]
macro_rules! ident {
	($string:literal) => {
		$crate::Ident::<&'static str>::new_unchecked($crate::parse_ident_str!($string))
	};
}

#[derive(Clone, Copy, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct Ident<S> {
	string: S,
}

impl<S> Ident<S> {
	#[doc(hidden)]
	pub const fn new_unchecked(string: S) -> Self {
		Self { string }
	}

	pub fn as_str(&self) -> &str
	where
		S: AsRef<str>,
	{
		self.string.as_ref()
	}

	pub fn as_str_ident(&self) -> Ident<&str>
	where
		S: AsRef<str>,
	{
		Ident {
			string: self.as_str(),
		}
	}

	pub fn to_string_ident(&self) -> Ident<String>
	where
		S: AsRef<str>,
	{
		Ident {
			string: self.as_str().to_owned(),
		}
	}

	pub fn into_inner(self) -> S {
		self.string
	}

	pub fn namespace(&self) -> &str
	where
		S: AsRef<str>,
	{
		self.namespace_and_path().0
	}

	pub fn path(&self) -> &str
	where
		S: AsRef<str>,
	{
		self.namespace_and_path().1
	}

	pub fn namespace_and_path(&self) -> (&str, &str)
	where
		S: AsRef<str>,
	{
		self.as_str()
			.split_once(':')
			.expect("invalid resource identifier")
	}
}

impl<'a> Ident<Cow<'a, str>> {
	pub fn new(string: impl Into<Cow<'a, str>>) -> Result<Self, IdentError> {
		parse(string.into())
	}

	#[expect(clippy::use_self)]
	#[must_use]
	pub fn borrowed(&self) -> Ident<Cow<'_, str>> {
		Ident::new_unchecked(Cow::Borrowed(self.as_str()))
	}
}

impl<S> AsRef<str> for Ident<S>
where
	S: AsRef<str>,
{
	fn as_ref(&self) -> &str {
		self.string.as_ref()
	}
}

impl<S> Borrow<str> for Ident<S>
where
	S: Borrow<str>,
{
	fn borrow(&self) -> &str {
		self.string.borrow()
	}
}

impl<S: Debug> Debug for Ident<S> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.string, f)
	}
}

impl<'de, S> Deserialize<'de> for Ident<S>
where
	S: Deserialize<'de>,
	Self: TryFrom<S, Error = IdentError>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Self::try_from(S::deserialize(deserializer)?).map_err(D::Error::custom)
	}
}

impl<S: Display> Display for Ident<S> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.string, f)
	}
}

impl From<Ident<&str>> for String {
	fn from(value: Ident<&str>) -> Self {
		value.as_str().to_owned()
	}
}

impl From<Ident<Self>> for String {
	fn from(value: Ident<Self>) -> Self {
		value.into_inner()
	}
}

impl From<Ident<Self>> for Cow<'_, str> {
	fn from(value: Ident<Self>) -> Self {
		value.into_inner()
	}
}

impl<'a> From<Ident<Cow<'a, str>>> for Ident<String> {
	fn from(value: Ident<Cow<'a, str>>) -> Self {
		Self {
			string: value.string.into_owned(),
		}
	}
}

impl From<Ident<String>> for Ident<Cow<'_, str>> {
	fn from(value: Ident<String>) -> Self {
		Self {
			string: value.string.into(),
		}
	}
}

impl<'a> From<Ident<&'a str>> for Ident<Cow<'a, str>> {
	fn from(value: Ident<&'a str>) -> Self {
		Self {
			string: value.string.into(),
		}
	}
}

impl<'a> From<Ident<&'a str>> for Ident<String> {
	fn from(value: Ident<&'a str>) -> Self {
		Self {
			string: value.string.to_owned(),
		}
	}
}

impl FromStr for Ident<String> {
	type Err = IdentError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Ident::new(s)?.into())
	}
}

impl FromStr for Ident<Cow<'static, str>> {
	type Err = IdentError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ident::<String>::try_from(s).map(From::from)
	}
}

impl<S, T> PartialEq<Ident<T>> for Ident<S>
where
	S: PartialEq<T>,
{
	fn eq(&self, other: &Ident<T>) -> bool {
		self.string == other.string
	}
}

impl<S, T> PartialOrd<Ident<T>> for Ident<S>
where
	S: PartialOrd<T>,
{
	fn partial_cmp(&self, other: &Ident<T>) -> Option<Ordering> {
		self.string.partial_cmp(&other.string)
	}
}

impl<T: Serialize> Serialize for Ident<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.string.serialize(serializer)
	}
}

impl<'a> TryFrom<&'a str> for Ident<String> {
	type Error = IdentError;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Ok(Ident::new(value)?.into())
	}
}

impl TryFrom<String> for Ident<String> {
	type Error = IdentError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Ok(Ident::new(value)?.into())
	}
}

impl<'a> TryFrom<Cow<'a, str>> for Ident<String> {
	type Error = IdentError;

	fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
		Ok(Ident::new(value)?.into())
	}
}

impl<'a> TryFrom<&'a str> for Ident<Cow<'a, str>> {
	type Error = IdentError;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl TryFrom<String> for Ident<Cow<'_, str>> {
	type Error = IdentError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl<'a> TryFrom<Cow<'a, str>> for Ident<Cow<'a, str>> {
	type Error = IdentError;

	fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("invalid resource identifier \"{0}\"")]
#[repr(transparent)]
pub struct IdentError(pub String);

fn parse(string: Cow<'_, str>) -> Result<Ident<Cow<'_, str>>, IdentError> {
	let check_namespace = |s: &str| {
		!s.is_empty()
			&& s.chars()
				.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
	};

	let check_path = |s: &str| {
		!s.is_empty()
			&& s.chars()
				.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'))
	};

	match string.split_once(':') {
		Some((namespace, path)) if check_namespace(namespace) && check_path(path) => {
			Ok(Ident { string })
		}
		None if check_path(&string) => Ok(Ident {
			string: format!("minecraft:{string}").into(),
		}),
		_ => Err(IdentError(string.into())),
	}
}

#[cfg(test)]
#[expect(clippy::should_panic_without_expect)]
mod tests {
	use super::{Ident, ident};

	#[test]
	fn check_namespace_and_path() {
		let id = ident!("namespace:path");
		assert_eq!(id.namespace(), "namespace");
		assert_eq!(id.path(), "path");
	}

	#[test]
	fn parse_valid() {
		ident!("minecraft:whatever");
		ident!("_what-ever55_:.whatever/whatever123456789_");
		ident!("vmm:frobnicator");
	}

	#[test]
	#[should_panic]
	fn parse_invalid_0() {
		Ident::new("").unwrap();
	}

	#[test]
	#[should_panic]
	fn parse_invalid_1() {
		Ident::new(":").unwrap();
	}

	#[test]
	#[should_panic]
	fn parse_invalid_2() {
		Ident::new("foo:bar:baz").unwrap();
	}

	#[test]
	fn equality() {
		assert_eq!(ident!("minecraft:my.identifier"), ident!("my.identifier"));
	}
}
