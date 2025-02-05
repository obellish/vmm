use alloc::{
	borrow::{Cow, ToOwned},
	string::{String, ToString},
	sync::Arc,
	vec::Vec,
};
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	str::{self, FromStr},
};

use thiserror::Error;

use crate::{
	LibraryNamespace, LibraryNamespaceError, Span,
	ast::{Ident, IdentError},
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct LibraryPath {
	inner: Arc<LibraryPathInner>,
}

impl LibraryPath {
	pub fn new(source: impl AsRef<str>) -> Result<Self, PathError> {
		let source = source.as_ref();
		if source.is_empty() {
			return Err(PathError::Empty);
		}

		let mut parts = source.split("::");
		let ns = LibraryNamespace::new(parts.next().ok_or(PathError::Empty)?)?;

		let mut components = Components::default();
		parts.map(Ident::new).try_for_each(|part| {
			part.map_err(PathError::InvalidComponent)
				.map(|c| components.push(c))
		})?;

		Ok(Self::make(ns, components))
	}

	pub fn from_components(
		ns: LibraryNamespace,
		components: impl IntoIterator<Item = Ident>,
	) -> Self {
		Self::make(ns, components.into_iter().collect())
	}

	fn make(ns: LibraryNamespace, components: Components) -> Self {
		Self {
			inner: Arc::new(LibraryPathInner { ns, components }),
		}
	}

	#[allow(clippy::len_without_is_empty)]
	#[must_use]
	pub fn len(&self) -> usize {
		self.inner.components.iter().map(|c| c.len()).sum::<usize>()
			+ self.inner.ns.len()
			+ (self.inner.components.len() * 2)
	}

	#[must_use]
	pub fn byte_len(&self) -> usize {
		self.len()
	}

	#[must_use]
	pub fn path(&self) -> Cow<'_, str> {
		if self.inner.components.is_empty() {
			Cow::Borrowed(self.inner.ns.as_str())
		} else {
			Cow::Owned(self.to_string())
		}
	}

	#[must_use]
	pub fn namespace(&self) -> &LibraryNamespace {
		&self.inner.ns
	}

	#[must_use]
	pub fn last(&self) -> &str {
		self.last_component().as_str()
	}

	pub fn last_component(&self) -> LibraryPathComponent<'_> {
		self.inner.components.last().map_or_else(
			|| LibraryPathComponent::Namespace(self.namespace()),
			LibraryPathComponent::Normal,
		)
	}

	#[must_use]
	pub fn num_components(&self) -> usize {
		self.inner.components.len() + 1
	}

	pub fn components(&self) -> impl Iterator<Item = LibraryPathComponent<'_>> + '_ {
		core::iter::once(LibraryPathComponent::Namespace(self.namespace())).chain(
			self.inner
				.components
				.iter()
				.map(LibraryPathComponent::Normal),
		)
	}

	#[must_use]
	pub fn is_kernel_path(&self) -> bool {
		matches!(self.namespace(), LibraryNamespace::Kernel)
	}

	#[must_use]
	pub fn is_exec_path(&self) -> bool {
		matches!(self.namespace(), LibraryNamespace::Exec)
	}

	#[must_use]
	pub fn is_anon_path(&self) -> bool {
		matches!(self.namespace(), LibraryNamespace::Anon)
	}

	#[must_use]
	pub fn starts_with(&self, other: &Self) -> bool {
		let mut a = self.components();
		let mut b = other.components();
		loop {
			match (a.next(), b.next()) {
				(_, None) => break true,
				(None, _) => break false,
				(Some(a), Some(b)) => {
					if a != b {
						break false;
					}
				}
			}
		}
	}

	pub fn set_namespace(&mut self, ns: LibraryNamespace) {
		let inner = Arc::make_mut(&mut self.inner);
		inner.ns = ns;
	}

	pub fn join(&self, other: &Self) -> Result<Self, PathError> {
		if other.inner.ns.is_reserved() {
			return Err(PathError::UnsupportedJoin);
		}

		let mut path = self.clone();
		{
			let inner = Arc::make_mut(&mut path.inner);
			inner.components.push(other.inner.ns.to_ident());
			inner
				.components
				.extend(other.inner.components.iter().cloned());
		}

		Ok(path)
	}

	pub fn try_push(&mut self, component: impl AsRef<str>) -> Result<(), PathError> {
		let component = component.as_ref().parse::<Ident>()?;
		self.push_ident(component);
		Ok(())
	}

	pub fn push_ident(&mut self, component: Ident) {
		let inner = Arc::make_mut(&mut self.inner);
		inner.components.push(component);
	}

	pub fn try_append(&self, component: impl AsRef<str>) -> Result<Self, PathError> {
		let mut path = self.clone();
		path.try_push(component)?;
		Ok(path)
	}

	pub fn try_append_ident(&self, component: Ident) -> Result<Self, PathError> {
		let mut path = self.clone();
		path.push_ident(component);
		Ok(path)
	}

	pub fn try_prepend(&self, component: impl AsRef<str>) -> Result<Self, PathError> {
		let ns = component.as_ref().parse::<LibraryNamespace>()?;
		let component = self.inner.ns.to_ident();
		let mut components = vec![component];
		components.extend(self.inner.components.iter().cloned());
		Ok(Self::make(ns, components))
	}

	pub fn pop(&mut self) -> Option<Ident> {
		let inner = Arc::make_mut(&mut self.inner);
		inner.components.pop()
	}

	#[must_use]
	pub fn strip_last(&self) -> Option<Self> {
		match self.inner.components.len() {
			0 => None,
			1 => Some(Self::make(self.inner.ns.clone(), Vec::new())),
			_ => {
				let ns = self.inner.ns.clone();
				let mut components = self.inner.components.clone();
				components.pop();
				Some(Self::make(ns, components))
			}
		}
	}

	pub fn validate(source: impl AsRef<str>) -> Result<usize, PathError> {
		let source = source.as_ref();

		let mut count = 0;
		let mut components = source.split("::");

		let ns = components.next().ok_or(PathError::Empty)?;
		LibraryNamespace::validate(ns)?;
		count += 1;

		for component in components {
			validate_component(component)?;
			count += 1;
		}

		Ok(count)
	}

	#[must_use]
	pub fn append_unchecked(&self, component: impl AsRef<str>) -> Self {
		let component = component.as_ref().to_owned().into_boxed_str();
		let component = Ident::new_unchecked(Span::unknown(component.into()));
		let mut path = self.clone();
		path.push_ident(component);
		path
	}
}

impl Deserializable for LibraryPath {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let len = source.read_u16()? as usize;
		let path = source.read_slice(len)?;
		let path =
			str::from_utf8(path).map_err(|e| DeserializationError::InvalidValue(e.to_string()))?;
		Self::new(path).map_err(|e| DeserializationError::InvalidValue(e.to_string()))
	}
}

impl Display for LibraryPath {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.inner.ns)?;
		for component in &self.inner.components {
			f.write_str("::")?;
			Display::fmt(&component, f)?;
		}

		Ok(())
	}
}

impl From<LibraryNamespace> for LibraryPath {
	fn from(value: LibraryNamespace) -> Self {
		Self::make(value, Vec::new())
	}
}

impl FromStr for LibraryPath {
	type Err = PathError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s)
	}
}

impl Serializable for LibraryPath {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let len = self.byte_len();

		target.write_u16(len as u16);
		target.write_bytes(self.inner.ns.as_bytes());
		for component in &self.inner.components {
			target.write_bytes(b"::");
			target.write_bytes(component.as_bytes());
		}
	}
}

impl TryFrom<String> for LibraryPath {
	type Error = PathError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl<'a> TryFrom<&'a str> for LibraryPath {
	type Error = PathError;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LibraryPathInner {
	ns: LibraryNamespace,
	components: Components,
}

#[derive(Debug, Error)]
pub enum PathError {
	#[error("invalid library path: cannot be empty")]
	Empty,
	#[error("invalid library path component: cannot be empty")]
	EmptyComponent,
	#[error("invalid library path component: {0}")]
	InvalidComponent(#[from] IdentError),
	#[error("invalid library path: contains invalid utf8 byte sequences")]
	InvalidUtf8,
	#[error(transparent)]
	InvalidNamespace(#[from] LibraryNamespaceError),
	#[error("cannot join a path with reserved name to other paths")]
	UnsupportedJoin,
}

pub enum LibraryPathComponent<'a> {
	Namespace(&'a LibraryNamespace),
	Normal(&'a Ident),
}

impl<'a> LibraryPathComponent<'a> {
	#[must_use]
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Namespace(ns) => ns,
			Self::Normal(id) => id,
		}
	}

	#[must_use]
	pub fn to_ident(&self) -> Ident {
		match self {
			Self::Namespace(ns) => ns.to_ident(),
			Self::Normal(id) => (*id).clone(),
		}
	}
}

impl AsRef<str> for LibraryPathComponent<'_> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Display for LibraryPathComponent<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self.as_str())
	}
}

impl Eq for LibraryPathComponent<'_> {}

impl From<LibraryPathComponent<'_>> for Ident {
	fn from(value: LibraryPathComponent<'_>) -> Self {
		value.to_ident()
	}
}

impl PartialEq for LibraryPathComponent<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Namespace(a), Self::Namespace(b)) => a == b,
			(Self::Normal(a), Self::Normal(b)) => a == b,
			_ => false,
		}
	}
}

impl PartialEq<str> for LibraryPathComponent<'_> {
	fn eq(&self, other: &str) -> bool {
		self.as_str().eq(other)
	}
}

type Components = Vec<Ident>;

fn validate_component(component: &str) -> Result<(), PathError> {
	if component.is_empty() {
		Err(PathError::EmptyComponent)
	} else if component.len() > LibraryNamespace::MAX_LENGTH {
		Err(PathError::InvalidComponent(IdentError::InvalidLength {
			max: LibraryNamespace::MAX_LENGTH,
		}))
	} else {
		Ident::validate(component)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;

	use super::{IdentError, LibraryNamespaceError, LibraryPath, PathError};

	#[test]
	fn new_path() -> Result<(), PathError> {
		let path = LibraryPath::new("foo")?;
		assert_eq!(path.num_components(), 1);

		Ok(())
	}

	#[test]
	fn new_path_fail() {
		let path = LibraryPath::new("");
		assert_matches!(path, Err(PathError::Empty));

		let path = LibraryPath::new("::");
		assert_matches!(
			path,
			Err(PathError::InvalidNamespace(LibraryNamespaceError::Empty))
		);

		let path = LibraryPath::new("foo::");
		assert_matches!(path, Err(PathError::InvalidComponent(IdentError::Empty)));

		let path = LibraryPath::new("::foo");
		assert_matches!(
			path,
			Err(PathError::InvalidNamespace(LibraryNamespaceError::Empty))
		);

		let path = LibraryPath::new("foo::1bar");
		assert_matches!(
			path,
			Err(PathError::InvalidComponent(IdentError::InvalidStart))
		);

		let path = LibraryPath::new("foo::b@r");
		assert_matches!(
			path,
			Err(PathError::InvalidComponent(IdentError::InvalidChars {
				ident: _
			}))
		);

		let path = LibraryPath::new("#foo::bar");
		assert_matches!(
			path,
			Err(PathError::InvalidNamespace(
				LibraryNamespaceError::InvalidStart
			))
		);
	}
}
