use alloc::string::String;

use crate::{
	SourceSpan, Span, Spanned,
	ast::{Block, Constant, Export, Import},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Form {
	ModuleDoc(Span<String>),
	Doc(Span<String>),
	Import(Import),
	Constant(Constant),
	Begin(Block),
	Procedure(Export),
}

impl From<Span<String>> for Form {
	fn from(value: Span<String>) -> Self {
		Self::Doc(value)
	}
}

impl From<Import> for Form {
	fn from(value: Import) -> Self {
		Self::Import(value)
	}
}

impl From<Constant> for Form {
	fn from(value: Constant) -> Self {
		Self::Constant(value)
	}
}

impl From<Block> for Form {
	fn from(value: Block) -> Self {
		Self::Begin(value)
	}
}

impl From<Export> for Form {
	fn from(value: Export) -> Self {
		Self::Procedure(value)
	}
}

impl Spanned for Form {
	fn span(&self) -> SourceSpan {
		match self {
			Self::ModuleDoc(doc) | Self::Doc(doc) => doc.span(),
			Self::Import(Import { span, .. }) | Self::Constant(Constant { span, .. }) => *span,
			Self::Begin(spanned) => spanned.span(),
			Self::Procedure(spanned) => spanned.span(),
		}
	}
}
