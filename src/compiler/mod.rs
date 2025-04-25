mod scanner;
mod token;

use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	mem,
};

use logos::Span;
use serde::{Deserialize, Serialize};

pub use self::{scanner::*, token::*};
use super::{Chunk, OpCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
	tokens: Vec<Token>,
	current_chunk: Chunk,
}

impl Compiler {
	pub fn compile(mut self) -> Result<Chunk, CompileError> {
		for token in mem::take(&mut self.tokens) {
			match token {
				Token::Number(num) => self.emit_number(num),
				_ => {}
			}
		}

		self.emit_return();

		Ok(self.current_chunk)
	}

	fn emit_return(&mut self) {
		self.current_chunk.push(OpCode::Return);
	}

	fn emit_number(&mut self, num: f64) {
		let idx = self.current_chunk.push_constant(num);
		self.current_chunk.push(OpCode::Constant(idx));
	}
}

impl FromIterator<Token> for Compiler {
	fn from_iter<T: IntoIterator<Item = Token>>(iter: T) -> Self {
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
