use logos::{Lexer, Logos};

use crate::{Instruction, ParsedInstruction};

#[derive(Debug, Clone)]
pub struct Scanner<'source> {
	inner: Lexer<'source, ParsedInstruction>,
}

impl<'source> Scanner<'source> {
	#[must_use]
	pub fn new(source: &'source <ParsedInstruction as Logos<'source>>::Source) -> Self {
		Self {
			inner: Lexer::new(source),
		}
	}
}

impl Iterator for Scanner<'_> {
	type Item = Instruction;

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().and_then(Result::ok).map(Into::into)
	}
}
