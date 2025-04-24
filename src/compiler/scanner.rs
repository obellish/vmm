use logos::{Lexer, Logos, Span, SpannedIter};

use super::{CompileError, Token};

pub struct Scanner<'source> {
	inner: SpannedIter<'source, Token>,
}

impl<'source> Scanner<'source> {
	#[must_use]
	pub fn new(content: &'source str) -> Self {
		Self {
			inner: Token::lexer(content).spanned(),
		}
	}
}

impl Iterator for Scanner<'_> {
	type Item = Result<(Token, Span), CompileError>;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|(res, span)| Ok((res?, span)))
	}
}
