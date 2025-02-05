use alloc::{borrow::ToOwned, string::String, sync::Arc};
use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::Deref,
	str::FromStr,
};

use thiserror::Error;

use crate::{SourceSpan, Span, Spanned};

#[derive(Clone)]
pub struct Ident {
	span: SourceSpan,
	name: Arc<str>,
}

impl Ident {
	pub fn new(source: impl AsRef<str>) -> Result<Self, IdentError> {
		source.as_ref().parse()
	}

	pub fn new_with_span(source: impl AsRef<str>, span: SourceSpan) -> Result<Self, IdentError> {
		source.as_ref().parse::<Self>().map(|id| id.with_span(span))
	}

	#[must_use]
	pub const fn with_span(mut self, span: SourceSpan) -> Self {
		self.span = span;
		self
	}

	#[must_use]
	pub fn new_unchecked(name: Span<Arc<str>>) -> Self {
		let (span, name) = name.into_parts();
		Self { span, name }
	}

	#[must_use]
	pub fn into_inner(self) -> Arc<str> {
		self.name
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		self
	}

	pub fn validate(source: impl AsRef<str>) -> Result<(), IdentError> {
		let source = source.as_ref();
		if source.is_empty() {
			return Err(IdentError::Empty);
		}

		if !source.starts_with(|c: char| c.is_ascii_alphabetic()) {
			return Err(IdentError::InvalidStart);
		}

		if !source
			.chars()
			.all(|c| c.is_ascii_alphabetic() || matches!(c, '_' | '0'..='9'))
		{
			return Err(IdentError::InvalidChars {
				ident: source.into(),
			});
		}

		Ok(())
	}
}

impl AsRef<str> for Ident {
	fn as_ref(&self) -> &str {
		self
	}
}

impl Debug for Ident {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("Ident").field(&self.name).finish()
	}
}

impl Deref for Ident {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.name
	}
}

impl Display for Ident {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self)
	}
}

impl Eq for Ident {}

impl FromStr for Ident {
	type Err = IdentError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::validate(s)?;
		let name: Arc<str> = Arc::from(s.to_owned());
		Ok(Self {
			span: SourceSpan::default(),
			name,
		})
	}
}

impl Hash for Ident {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl Ord for Ident {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.name.cmp(&other.name)
	}
}

impl PartialEq for Ident {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl PartialEq<str> for Ident {
	fn eq(&self, other: &str) -> bool {
		(*self.name).eq(other)
	}
}

impl PartialEq<&str> for Ident {
	fn eq(&self, other: &&str) -> bool {
		self.eq(*other)
	}
}

impl PartialEq<String> for Ident {
	fn eq(&self, other: &String) -> bool {
		self.eq(other.as_str())
	}
}

impl PartialOrd for Ident {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Spanned for Ident {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

#[derive(Debug, Error)]
pub enum IdentError {
	#[error("invalid identifier: cannot be empty")]
	Empty,
	#[error(
		"invalid identifier '{ident}': must contain only lowercase, ascii alphanumeric characters, or underscores"
	)]
	InvalidChars { ident: Arc<str> },
	#[error("invalid identifier: must start with lowercase ascii alphabetic character")]
	InvalidStart,
	#[error("invalid identifier: length exceeds the maximum of {max} bytes")]
	InvalidLength { max: usize },
	#[error("invalid identifier: {0}")]
	Casing(#[from] CaseKindError),
}

#[derive(Debug, Error)]
pub enum CaseKindError {
	#[error(
		"only uppercase characters or underscores are allowed, and must start with an alphabetic character"
	)]
	Screaming,
	#[error(
		"only lowercase characters or underscores are allowed, and must start with an alphabetic character"
	)]
	Snake,
	#[error(
		"only alphanumeric characters are allowed, and must start with a lowercase alphabetic character"
	)]
	Camel,
}
