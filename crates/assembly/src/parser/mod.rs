mod error;
mod token;

pub use self::{
	error::{BinErrorKind, HexErrorKind, LiteralErrorKind, ParsingError},
	token::{BinEncodedValue, DocumentationType, HexEncodedValue, Token},
};

type ParseError<'a> = lalrpop_util::ParseError<u32, Token<'a>, ParsingError>;
