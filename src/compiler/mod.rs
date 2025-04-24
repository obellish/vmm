mod scanner;
mod token;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use logos::Span;
use serde::{Deserialize, Serialize};

pub use self::{scanner::*, token::*};
use super::Chunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
	tokens: Vec<(Token, Span)>,
	current_chunk: Chunk,
}

impl Compiler {
	pub fn compile(self) -> Result<Chunk, CompileError> {
		Ok(self.current_chunk)
	}
}

impl FromIterator<(Token, Span)> for Compiler {
	fn from_iter<T: IntoIterator<Item = (Token, Span)>>(iter: T) -> Self {
		Self {
			tokens: Vec::from_iter(iter),
			current_chunk: Chunk::default(),
		}
	}
}

#[derive(Debug)]
pub enum CompileError {
	ParseToken(ParseTokenError),
}

impl Display for CompileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::ParseToken(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for CompileError {}

impl From<ParseTokenError> for CompileError {
	fn from(value: ParseTokenError) -> Self {
		Self::ParseToken(value)
	}
}
