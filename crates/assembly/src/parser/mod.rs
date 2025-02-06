mod error;
mod lexer;
mod scanner;
mod token;

pub use self::{
	error::{BinErrorKind, HexErrorKind, LiteralErrorKind, ParsingError},
	lexer::Lexer,
	scanner::Scanner,
	token::{BinEncodedValue, DocumentationType, HexEncodedValue, Token},
};

type ParseError<'a> = lalrpop_util::ParseError<u32, Token<'a>, ParsingError>;
