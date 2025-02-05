use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

use vmm_core::{
	crypto::hash::RpoDigest,
	prettier::{Document, PrettyPrint, const_text, display, nl, text},
};

use super::{ProcedureName, QualifiedProcedureName};
use crate::{
	LibraryNamespace, LibraryPath,
	ast::InvocationTarget,
	diagnostics::{SourceSpan, Span, Spanned},
	utils::DisplayHex,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureAlias {
	docs: Option<Span<String>>,
	name: ProcedureName,
	target: AliasTarget,
}

impl ProcedureAlias {
	#[must_use]
	pub const fn new(name: ProcedureName, target: AliasTarget) -> Self {
		Self {
			docs: None,
			name,
			target,
		}
	}

	#[must_use]
	pub fn with_docs(mut self, docs: Option<Span<String>>) -> Self {
		self.docs = docs;
		self
	}

	#[must_use]
	pub const fn docs(&self) -> Option<&Span<String>> {
		self.docs.as_ref()
	}

	#[must_use]
	pub const fn name(&self) -> &ProcedureName {
		&self.name
	}

	#[must_use]
	pub const fn target(&self) -> &AliasTarget {
		&self.target
	}

	pub fn target_mut(&mut self) -> &mut AliasTarget {
		&mut self.target
	}

	#[must_use]
	pub const fn is_absolute(&self) -> bool {
		matches!(
			self.target(),
			AliasTarget::MastRoot(_) | AliasTarget::AbsoluteProcedurePath(_)
		)
	}

	#[must_use]
	pub fn is_renamed(&self) -> bool {
		match self.target() {
			AliasTarget::MastRoot(_) => true,
			AliasTarget::ProcedurePath(fqn) | AliasTarget::AbsoluteProcedurePath(fqn) => {
				fqn.name != self.name
			}
		}
	}
}

impl Display for ProcedureAlias {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for ProcedureAlias {
	fn render(&self) -> Document {
		let mut doc = Document::Empty;
		if let Some(docs) = self.docs.as_deref() {
			doc = docs
				.lines()
				.map(text)
				.reduce(|acc, line| acc + nl() + const_text("#! ") + line)
				.unwrap_or_default();
		}

		doc += const_text("export.");
		doc += match self.target() {
			target @ AliasTarget::MastRoot(_) => display(format_args!("{target}->{}", self.name)),
			target => {
				let prefix = if self.is_absolute() { "::" } else { "" };
				if self.is_renamed() {
					display(format_args!("{prefix}{target}->{}", self.name))
				} else {
					display(format_args!("{prefix}{target}"))
				}
			}
		};

		doc
	}
}

impl Spanned for ProcedureAlias {
	fn span(&self) -> SourceSpan {
		self.target().span()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasTarget {
	MastRoot(Span<RpoDigest>),
	ProcedurePath(QualifiedProcedureName),
	AbsoluteProcedurePath(QualifiedProcedureName),
}

impl Display for AliasTarget {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl From<Span<RpoDigest>> for AliasTarget {
	fn from(value: Span<RpoDigest>) -> Self {
		Self::MastRoot(value)
	}
}

impl From<&AliasTarget> for InvocationTarget {
	fn from(value: &AliasTarget) -> Self {
		match value {
			AliasTarget::MastRoot(digest) => Self::MastRoot(*digest),
			AliasTarget::ProcedurePath(fqn) => {
				let name = fqn.name.clone();
				let module = fqn.module.last_component().to_ident();
				Self::ProcedurePath { name, module }
			}
			AliasTarget::AbsoluteProcedurePath(fqn) => Self::AbsoluteProcedurePath {
				name: fqn.name.clone(),
				path: fqn.module.clone(),
			},
		}
	}
}

impl From<AliasTarget> for InvocationTarget {
	fn from(value: AliasTarget) -> Self {
		(&value).into()
	}
}

impl PrettyPrint for AliasTarget {
	fn render(&self) -> Document {
		match self {
			Self::MastRoot(digest) => display(DisplayHex(digest.as_bytes().as_slice())),
			Self::ProcedurePath(fqn) => display(fqn),
			Self::AbsoluteProcedurePath(fqn) => display(format_args!("::{fqn}")),
		}
	}
}

impl Spanned for AliasTarget {
	fn span(&self) -> SourceSpan {
		match self {
			Self::MastRoot(spanned) => spanned.span(),
			Self::ProcedurePath(spanned) | Self::AbsoluteProcedurePath(spanned) => spanned.span(),
		}
	}
}

impl TryFrom<InvocationTarget> for AliasTarget {
	type Error = InvocationTarget;

	fn try_from(value: InvocationTarget) -> Result<Self, Self::Error> {
		let span = value.span();
		match value {
			InvocationTarget::MastRoot(digest) => Ok(Self::MastRoot(digest)),
			InvocationTarget::ProcedurePath { name, module } => {
				let ns = LibraryNamespace::from_ident_unchecked(module);
				let module = LibraryPath::from_components(ns, []);
				Ok(Self::ProcedurePath(QualifiedProcedureName {
					span,
					module,
					name,
				}))
			}
			InvocationTarget::AbsoluteProcedurePath { name, path: module } => {
				Ok(Self::AbsoluteProcedurePath(QualifiedProcedureName {
					span,
					module,
					name,
				}))
			}
			target @ InvocationTarget::ProcedureName(_) => Err(target),
		}
	}
}
