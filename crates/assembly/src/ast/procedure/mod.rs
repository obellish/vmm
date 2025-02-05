mod alias;
mod id;
mod inner;
mod name;
mod resolver;

use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};

use vmm_core::prettier::{Document, PrettyPrint};

pub use self::{
	alias::{AliasTarget, ProcedureAlias},
	id::ProcedureIndex,
	inner::{Procedure, Visibility},
	name::{ProcedureName, QualifiedProcedureName},
	resolver::{LocalNameResolver, ResolvedProcedure},
};
use super::{AttributeSet, Invoke};
use crate::{SourceSpan, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Export {
	Procedure(Procedure),
	Alias(ProcedureAlias),
}

impl Export {
	#[must_use]
	pub fn with_docs(self, docs: Option<Span<String>>) -> Self {
		match self {
			Self::Procedure(proc) => Self::Procedure(proc.with_docs(docs)),
			Self::Alias(alias) => Self::Alias(alias.with_docs(docs)),
		}
	}

	#[must_use]
	pub const fn name(&self) -> &ProcedureName {
		match self {
			Self::Procedure(proc) => proc.name(),
			Self::Alias(alias) => alias.name(),
		}
	}

	#[must_use]
	pub fn docs(&self) -> Option<&str> {
		match self {
			Self::Procedure(proc) => proc.docs().map(|spanned| spanned.as_deref().into_inner()),
			Self::Alias(alias) => alias.docs().map(|spanned| spanned.as_deref().into_inner()),
		}
	}

	#[must_use]
	pub const fn attributes(&self) -> Option<&AttributeSet> {
		let Self::Procedure(proc) = self else {
			return None;
		};

		Some(proc.attributes())
	}

	#[must_use]
	pub const fn visibility(&self) -> Visibility {
		let Self::Procedure(proc) = self else {
			return Visibility::Public;
		};

		proc.visibility()
	}

	#[must_use]
	pub const fn num_locals(&self) -> usize {
		let Self::Procedure(proc) = self else {
			return 0;
		};

		proc.num_locals() as usize
	}

	#[must_use]
	pub fn is_main(&self) -> bool {
		self.name().is_main()
	}

	#[must_use]
	#[track_caller]
	pub fn unwrap_procedure(self) -> Procedure {
		match self {
			Self::Procedure(proc) => proc,
			Self::Alias(_) => panic!("attempted to unwrap alias export as procedure definition"),
		}
	}

	pub(crate) fn invoked<'a, 'b: 'a>(&'b self) -> impl Iterator<Item = &'a Invoke> + 'a {
		match self {
			Self::Procedure(proc) if proc.invoked.is_empty() => self::inner::InvokedIter::Empty,
			Self::Procedure(proc) => self::inner::InvokedIter::NonEmpty(proc.invoked.iter()),
			Self::Alias(_) => self::inner::InvokedIter::Empty,
		}
	}
}

impl Display for Export {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for Export {
	fn render(&self) -> Document {
		match self {
			Self::Procedure(proc) => proc.render(),
			Self::Alias(alias) => alias.render(),
		}
	}
}

impl Spanned for Export {
	fn span(&self) -> SourceSpan {
		match self {
			Self::Procedure(proc) => proc.span(),
			Self::Alias(alias) => alias.span(),
		}
	}
}
