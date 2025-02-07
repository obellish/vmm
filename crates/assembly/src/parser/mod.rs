mod error;
mod lexer;
mod scanner;
mod token;

lalrpop_util::lalrpop_mod!(
	#[allow(clippy::all)]
	grammar,
	"/parser/grammar.rs"
);

#[doc(hidden)]
#[macro_export]
macro_rules! span {
	($id:expr, $l:expr, $r:expr) => {
		$crate::SourceSpan::new($id, $l..$r)
	};
	($id:expr, $i:expr) => {
		$crate::SourceSpan::new($id, $i)
	};
}

use alloc::{collections::BTreeSet, sync::Arc};

pub use self::{
	error::{BinErrorKind, HexErrorKind, LiteralErrorKind, ParsingError},
	lexer::Lexer,
	scanner::Scanner,
	token::{BinEncodedValue, DocumentationType, HexEncodedValue, Token},
};
use crate::{
	LibraryPath, SourceManager, SourceSpan, Span, Spanned, ast,
	diagnostics::{Report, SourceFile},
	sema,
};

type ParseError<'a> = lalrpop_util::ParseError<u32, Token<'a>, ParsingError>;

pub struct ModuleParser {
	kind: ast::ModuleKind,
	interned: BTreeSet<Arc<str>>,
	warnings_as_errors: bool,
}

impl ModuleParser {}
