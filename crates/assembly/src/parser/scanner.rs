use core::{iter::Peekable, ops::Range, str::CharIndices};

pub struct Scanner<'input> {
	input: &'input str,
	chars: Peekable<CharIndices<'input>>,
	current: (usize, char),
	pending: (usize, char),
	start: usize,
	end: usize,
}

impl<'input> Scanner<'input> {
	pub fn new(input: &'input str) -> Self {
		let end = input.len();
		assert!(end < u32::MAX as usize, "file too large");

		let mut chars = input.char_indices().peekable();
		let current = chars.next().unwrap_or((0, '\0'));
		let pending = chars.next().unwrap_or((end, '\0'));

		Self {
			input,
			chars,
			current,
			pending,
			start: 0,
			end,
		}
	}

	pub const fn start(&self) -> usize {
		self.start
	}

	pub fn advance(&mut self) {
		self.current = self.pending;
		self.pending = self.chars.next().unwrap_or((self.end, '\0'));
	}

	pub fn pop(&mut self) -> (usize, char) {
		let current = self.current;
		self.advance();
		current
	}

	pub const fn peek(&self) -> (usize, char) {
		self.pending
	}

	pub fn peek_next(&mut self) -> (usize, char) {
		self.chars.peek().copied().unwrap_or((self.end, '\0'))
	}

	pub const fn read(&self) -> (usize, char) {
		self.current
	}

	pub fn slice(&self, span: impl Into<Range<usize>>) -> &'input str {
		let range = span.into();
		let bytes = &self.input.as_bytes()[range];
		core::str::from_utf8(bytes).expect("invalid slice indices")
	}
}
