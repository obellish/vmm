use logos::{Lexer, Logos, Span, SpannedIter};

use super::{CompileError, Token};

pub struct Scanner<'source> {
	inner: Lexer<'source, Token>,
}

impl<'source> Scanner<'source> {
	#[must_use]
	pub fn new(content: &'source str) -> Self {
		Self {
			inner: Token::lexer(content),
		}
	}
}

impl Iterator for Scanner<'_> {
	type Item = Result<Token, CompileError>;

	fn next(&mut self) -> Option<Self::Item> {
		let token = self.inner.next()?;

		Some(token.map_err(Into::into))
	}
}
