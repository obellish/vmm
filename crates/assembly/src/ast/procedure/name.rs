use alloc::{borrow::ToOwned, sync::Arc};
use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::{Hash, Hasher},
	ops::Deref,
	str::FromStr,
};

use vmm_core::prettier::{Document, PrettyPrint, display};

use crate::{
	LibraryNamespace, LibraryPath, SourceSpan, Span, Spanned,
	ast::{CaseKindError, Ident, IdentError},
	diagnostics::{IntoDiagnostic, Report},
};

#[derive(Clone)]
pub struct QualifiedProcedureName {
	pub span: SourceSpan,
	pub module: LibraryPath,
	pub name: ProcedureName,
}

impl QualifiedProcedureName {
	#[must_use]
	pub fn new(module: LibraryPath, name: ProcedureName) -> Self {
		Self {
			span: SourceSpan::default(),
			module,
			name,
		}
	}

	#[must_use]
	pub fn namespace(&self) -> &LibraryNamespace {
		self.module.namespace()
	}
}

impl Debug for QualifiedProcedureName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("FullyQualifiedProcedureName")
			.field("module", &self.module)
			.field("name", &self.name)
			.finish_non_exhaustive()
	}
}

impl Display for QualifiedProcedureName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.module, f)?;
		f.write_str("::")?;
		Display::fmt(&self.name, f)
	}
}

impl Eq for QualifiedProcedureName {}

impl From<QualifiedProcedureName> for miette::SourceSpan {
	fn from(value: QualifiedProcedureName) -> Self {
		value.span.into()
	}
}

impl FromStr for QualifiedProcedureName {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.rsplit_once("::") {
			None => Err(Report::msg(
				"invalid fully-qualified procedure name, expected namespace",
			)),
			Some((path, name)) => {
				let name = name.parse::<ProcedureName>().into_diagnostic()?;
				let path = path.parse::<LibraryPath>().into_diagnostic()?;
				Ok(Self::new(path, name))
			}
		}
	}
}

impl Ord for QualifiedProcedureName {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.module
			.cmp(&other.module)
			.then_with(|| self.name.cmp(&other.name))
	}
}

impl PartialEq for QualifiedProcedureName {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.module == other.module
	}
}

impl PartialOrd for QualifiedProcedureName {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PrettyPrint for QualifiedProcedureName {
	fn render(&self) -> Document {
		display(self)
	}
}

impl Spanned for QualifiedProcedureName {
	fn span(&self) -> SourceSpan {
		self.span
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ProcedureName(Ident);

impl ProcedureName {
	pub const MAIN_PROC_NAME: &str = "#main";

	pub fn new(name: impl AsRef<str>) -> Result<Self, IdentError> {
		name.as_ref().parse()
	}

	#[must_use]
	pub const fn new_unchecked(name: Ident) -> Self {
		Self(name)
	}

	pub fn new_with_span(name: impl AsRef<str>, span: SourceSpan) -> Result<Self, IdentError> {
		Self::new(name).map(|name| name.with_span(span))
	}

	#[must_use]
	pub fn with_span(self, span: SourceSpan) -> Self {
		Self(self.0.with_span(span))
	}

	#[must_use]
	pub fn main() -> Self {
		let name = Arc::from(Self::MAIN_PROC_NAME.to_owned());
		Self::new_unchecked(Ident::new_unchecked(Span::unknown(name)))
	}

	#[must_use]
	pub fn is_main(&self) -> bool {
		self.0 == Self::MAIN_PROC_NAME
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		self
	}
}

impl AsRef<Ident> for ProcedureName {
	fn as_ref(&self) -> &Ident {
		self
	}
}

impl AsRef<str> for ProcedureName {
	fn as_ref(&self) -> &str {
		self
	}
}

impl Deref for ProcedureName {
	type Target = Ident;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for ProcedureName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(self)
	}
}

impl Eq for ProcedureName {}

impl From<ProcedureName> for miette::SourceSpan {
	fn from(value: ProcedureName) -> Self {
		value.span().into()
	}
}

impl FromStr for ProcedureName {
	type Err = IdentError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut chars = s.char_indices();
		let raw = match chars.next() {
			None => Err(IdentError::Empty),
			Some((_, '"')) => loop {
				if let Some((pos, c)) = chars.next() {
					match c {
						'"' => {
							if chars.next().is_some() {
								break Err(IdentError::InvalidChars { ident: s.into() });
							}
							let tok = &s[1..pos];
							break Ok(Arc::from(tok.to_owned()));
						}
						c if c.is_alphanumeric() => continue,
						'_' | '$' | '-' | '!' | '?' | '<' | '>' | ':' | '.' => continue,
						_ => break Err(IdentError::InvalidChars { ident: s.into() }),
					}
				}
				break Err(IdentError::InvalidChars { ident: s.into() });
			},
			Some((_, c)) if c.is_ascii_lowercase() || matches!(c, '_' | '$') => {
				if chars.as_str().contains(|c| match c {
					c if c.is_ascii_alphanumeric() => false,
					'_' | '$' => false,
					_ => true,
				}) {
					Err(IdentError::InvalidChars { ident: s.into() })
				} else {
					Ok(Arc::from(s.to_owned()))
				}
			}
			Some((_, c)) if c.is_ascii_uppercase() => Err(IdentError::Casing(CaseKindError::Snake)),
			Some(_) => Err(IdentError::InvalidChars { ident: s.into() }),
		}?;

		Ok(Self(Ident::new_unchecked(Span::unknown(raw))))
	}
}

impl Hash for ProcedureName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl Ord for ProcedureName {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl PartialEq for ProcedureName {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl PartialEq<str> for ProcedureName {
	fn eq(&self, other: &str) -> bool {
		self.0 == other
	}
}

impl PartialEq<Ident> for ProcedureName {
	fn eq(&self, other: &Ident) -> bool {
		self.0.eq(other)
	}
}

impl PartialOrd for ProcedureName {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Spanned for ProcedureName {
	fn span(&self) -> SourceSpan {
		self.0.span()
	}
}
