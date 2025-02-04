use alloc::{string::String, sync::Arc};

use vmm_core::{
	Felt,
	prettier::{Document, PrettyPrint, text},
};

use crate::{SourceSpan, Span, Spanned, ast::Ident, parser::HexEncodedValue};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetaExpr {
	Ident(Ident),
	Int(Span<HexEncodedValue>),
	String(Ident),
}

impl PrettyPrint for MetaExpr {
	fn render(&self) -> Document {
		match self {
			Self::Ident(id) => text(id),
			Self::Int(value) => text(value),
			Self::String(id) => text(format!("\"{}\"", id.as_str().escape_default())),
		}
	}
}
