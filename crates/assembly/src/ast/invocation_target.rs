use core::fmt::{Display, Formatter, Result as FmtResult};

use vmm_core::{
	crypto::hash::RpoDigest,
	prettier::{Document, PrettyPrint, display},
};

use super::ProcedureName;
use crate::{LibraryPath, SourceSpan, Span, Spanned, ast::Ident, utils::DisplayHex};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Invoke {
	pub kind: InvokeKind,
	pub target: InvocationTarget,
}

impl Invoke {
	#[must_use]
	pub const fn new(kind: InvokeKind, target: InvocationTarget) -> Self {
		Self { kind, target }
	}
}

impl Spanned for Invoke {
	fn span(&self) -> SourceSpan {
		self.target.span()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InvokeKind {
	Exec,
	Call,
	SysCall,
	ProcRef,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvocationTarget {
	MastRoot(Span<RpoDigest>),
	ProcedureName(ProcedureName),
	ProcedurePath {
		name: ProcedureName,
		module: Ident,
	},
	AbsoluteProcedurePath {
		name: ProcedureName,
		path: LibraryPath,
	},
}

impl Display for InvocationTarget {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for InvocationTarget {
	fn render(&self) -> Document {
		match self {
			Self::MastRoot(digest) => display(DisplayHex(digest.as_bytes().as_slice())),
			Self::ProcedureName(name) => display(name),
			Self::ProcedurePath { name, module } => display(format_args!("{module}::{name}")),
			Self::AbsoluteProcedurePath { name, path } => display(format_args!("::{path}::{name}")),
		}
	}
}

impl Spanned for InvocationTarget {
	fn span(&self) -> SourceSpan {
		match self {
			Self::MastRoot(spanned) => spanned.span(),
			Self::ProcedureName(spanned) => spanned.span(),
			Self::ProcedurePath { name, .. } | Self::AbsoluteProcedurePath { name, .. } => {
				name.span()
			}
		}
	}
}
