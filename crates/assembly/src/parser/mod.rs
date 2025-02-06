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

pub use self::{
	error::{BinErrorKind, HexErrorKind, LiteralErrorKind, ParsingError},
	lexer::Lexer,
	scanner::Scanner,
	token::{BinEncodedValue, DocumentationType, HexEncodedValue, Token},
};

type ParseError<'a> = lalrpop_util::ParseError<u32, Token<'a>, ParsingError>;
