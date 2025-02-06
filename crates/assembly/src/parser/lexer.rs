use alloc::string::String;
use core::{num::IntErrorKind, ops::Range};

use vmm_core::{Felt, FieldElement, StarkField};

use super::{
	BinEncodedValue, BinErrorKind, DocumentationType, HexEncodedValue, HexErrorKind,
	LiteralErrorKind, ParsingError, Scanner, Token,
};
use crate::diagnostics::{ByteOffset, SourceId, SourceSpan};

pub type Lexed<'input> = Result<(u32, Token<'input>, u32), ParsingError>;

macro_rules! pop {
	($lex:ident) => {{
		$lex.skip();
	}};
	($lex:ident, $token:expr) => {{
		pop!($lex);
		Ok($token)
	}};
}

macro_rules! pop2 {
	($lex:ident) => {{
		pop!($lex);
		pop!($lex);
	}};
	($lex:ident, $token:expr) => {{
		pop2!($lex);
		Ok($token)
	}};
}

pub struct Lexer<'input> {
	source_id: SourceId,
	scanner: Scanner<'input>,
	token: Token<'input>,
	token_start: usize,
	token_end: usize,
	line_num: usize,
	eof: bool,
	empty: bool,
	keywords: aho_corasick::AhoCorasick,
	error: Option<ParsingError>,
}

impl<'input> Lexer<'input> {
	pub fn new(source_id: SourceId, scanner: Scanner<'input>) -> Self {
		let start = scanner.start();
		let keywords = Token::keyword_searcher();
		let mut lexer = Self {
			source_id,
			scanner,
			token: Token::Eof,
			token_start: start,
			token_end: start,
			line_num: 0,
			eof: false,
			empty: false,
			keywords,
			error: None,
		};
		lexer.advance();
		lexer
	}

	pub fn lex(&mut self) -> Option<Lexed<'input>> {
		if let Some(err) = self.error.take() {
			return Some(Err(err));
		}

		if self.eof && matches!(self.token, Token::Eof) {
			if self.empty {
				return None;
			}
			self.empty = true;
			let end = self.token_end as u32;
			return Some(Ok((end, Token::Eof, end)));
		}

		let token = core::mem::replace(&mut self.token, Token::Eof);
		let start = self.token_start;
		let end = self.token_end;
		self.advance();
		Some(Ok((start as u32, token, end as u32)))
	}

	fn advance(&mut self) {
		self.advance_start();
		match self.tokenize() {
			Ok(tok) => {
				self.token = tok;
			}
			Err(err) => {
				self.error = Some(err);
			}
		}
	}

	fn advance_start(&mut self) {
		let mut position: usize;
		loop {
			let (pos, c) = self.scanner.read();

			position = pos;
			if matches!(c, '\0') {
				self.eof = true;
				return;
			}

			if c.is_whitespace() {
				if matches!(c, '\n') {
					self.line_num += 1;
				}

				self.scanner.advance();
				continue;
			}

			break;
		}

		self.token_start = position;
	}

	fn pop(&mut self) -> char {
		let (pos, c) = self.scanner.pop();
		self.token_end = pos + c.len_utf8();
		c
	}

	const fn peek(&self) -> char {
		self.scanner.peek().1
	}

	fn peek_next(&mut self) -> char {
		let (_, c) = self.scanner.peek_next();
		c
	}

	const fn read(&self) -> char {
		self.scanner.read().1
	}

	fn skip(&mut self) {
		self.pop();
	}

	fn span(&self) -> SourceSpan {
		assert!(self.token_start <= self.token_end, "invalid range");
		assert!(u32::try_from(self.token_end).is_ok(), "file too large");
		SourceSpan::new(
			self.source_id,
			(self.token_start as u32)..(self.token_end as u32),
		)
	}

	fn slice_span(&self, span: impl Into<Range<u32>>) -> &'input str {
		let range = span.into();
		self.scanner
			.slice((range.start as usize)..(range.end as usize))
	}

	fn slice(&self) -> &'input str {
		self.slice_span(self.span())
	}

	fn skip_whitespace(&mut self) {
		let mut c: char;
		loop {
			c = self.read();

			if !c.is_whitespace() {
				break;
			}

			if matches!(c, '\n') {
				self.line_num += 1;
			}

			self.skip();
		}
	}

	fn tokenize(&mut self) -> Result<Token<'input>, ParsingError> {
		let c = self.read();

		if matches!(c, '#') {
			if matches!(self.peek(), '!') {
				pop2!(self);
				return self.lex_docs();
			}
			self.skip();
			self.skip_comment();
			return Ok(Token::Comment);
		}

		if matches!(c, '\0') {
			self.eof = true;
			return Ok(Token::Eof);
		}

		if c.is_whitespace() {
			self.skip_whitespace();
		}

		match self.read() {
			'@' => pop!(self, Token::At),
			'!' => pop!(self, Token::Bang),
			':' if matches!(self.peek(), ':') => pop2!(self, Token::ColonColon),
			'.' => pop!(self, Token::Dot),
			',' => pop!(self, Token::Comma),
			'=' => pop!(self, Token::Equal),
			'(' => pop!(self, Token::Lparen),
			'[' => pop!(self, Token::Lbracket),
			')' => pop!(self, Token::Rparen),
			']' => pop!(self, Token::Rbracket),
			'-' if matches!(self.peek(), '>') => pop2!(self, Token::Rstab),
			'-' => pop!(self, Token::Minus),
			'+' => pop!(self, Token::Plus),
			'/' if matches!(self.peek(), '/') => pop2!(self, Token::SlashSlash),
			'/' => pop!(self, Token::Slash),
			'*' => pop!(self, Token::Star),
			'"' => self.lex_quoted_identifier_or_string(),
			'0' => match self.peek() {
				'x' => {
					pop2!(self);
					self.lex_hex()
				}
				'b' => {
					pop2!(self);
					self.lex_bin()
				}
				'0'..='9' => self.lex_number(),
				_ => pop!(self, Token::Int(0)),
			},
			'1'..='9' => self.lex_number(),
			'a'..='z' => self.lex_keyword_or_ident(),
			'A'..='Z' => self.lex_const_identifier(),
			'_' if matches!(self.peek(), c if c.is_alphanumeric()) => self.lex_identifier(),
			_ => Err(ParsingError::InvalidToken { span: self.span() }),
		}
	}

	fn lex_docs(&mut self) -> Result<Token<'input>, ParsingError> {
		let mut buf = String::new();

		let mut c;
		let mut line_start = self.token_start + 2;
		let is_module_doc = matches!(self.line_num, 0);
		loop {
			c = self.read();

			if matches!(c, '\0') {
				self.eof = true;
				buf.push_str(
					self.slice_span((line_start as u32)..(self.token_end as u32))
						.trim(),
				);

				let is_first_line = matches!(self.line_num, 0);
				break Ok(Token::DocComment(if is_first_line {
					DocumentationType::Module(buf)
				} else {
					DocumentationType::Form(buf)
				}));
			}

			if matches!(c, '\n') {
				self.line_num += 1;

				buf.push_str(
					self.slice_span((line_start as u32)..(self.token_end as u32))
						.trim(),
				);
				buf.push('\n');

				self.skip();
				c = self.read();
				match c {
					'#' if matches!(self.peek(), '!') => {
						pop2!(self);
						line_start = self.token_end;
						continue;
					}
					_ if is_module_doc => {
						break Ok(Token::DocComment(DocumentationType::Module(buf)));
					}
					_ => break Ok(Token::DocComment(DocumentationType::Form(buf))),
				}
			}
			self.skip();
		}
	}

	fn skip_comment(&mut self) {
		let mut c;
		loop {
			c = self.read();

			match c {
				'\n' => {
					self.skip();
					self.line_num += 1;
					break;
				}
				'\0' => {
					self.eof = true;
					break;
				}
				_ => self.skip(),
			}
		}
	}

	fn lex_keyword_or_ident(&mut self) -> Result<Token<'input>, ParsingError> {
		let c = self.pop();
		debug_assert!(c.is_ascii_alphabetic() && c.is_lowercase());

		loop {
			match self.read() {
				'_' | '0'..='9' => self.skip(),
				c if c.is_ascii_alphabetic() => self.skip(),
				_ => break,
			}
		}

		let name = self.slice();
		match name {
			"exp" => {
				if matches!((self.read(), self.peek()), ('.', 'u')) {
					pop2!(self, Token::ExpU)
				} else {
					Ok(Token::Exp)
				}
			}
			_ => Ok(Token::from_keyword_or_ident_with_searcher(
				name,
				&self.keywords,
			)),
		}
	}

	fn lex_quoted_identifier_or_string(&mut self) -> Result<Token<'input>, ParsingError> {
		self.skip();

		let mut is_identifier = true;
		let quote_size = ByteOffset::from_char_len('"');
		loop {
			match self.read() {
				'\0' | '\n' => {
					break Err(ParsingError::UnclosedQuote {
						start: SourceSpan::at(self.source_id, self.span().start()),
					});
				}
				'\\' => {
					is_identifier = false;
					self.skip();
					match self.read() {
						'\n' | '"' => {
							self.skip();
						}
						_ => {}
					}
				}
				'"' => {
					let span = self.span();
					let start = span.start() + quote_size;
					let span = SourceSpan::new(self.source_id, start..span.end());

					self.skip();
					break Ok(if is_identifier {
						Token::QuotedIdent(self.slice_span(span))
					} else {
						Token::QuotedString(self.slice_span(span))
					});
				}
				c if c.is_ascii_alphanumeric() => self.skip(),
				'_' | '$' | '!' | '<' | ':' | '>'..='?' | '-'..='.' => self.skip(),
				_ => {
					is_identifier = false;
					self.skip();
				}
			}
		}
	}

	fn lex_identifier(&mut self) -> Result<Token<'input>, ParsingError> {
		let c = self.pop();
		debug_assert!(c.is_ascii_lowercase() || matches!(c, '_'));

		loop {
			match self.read() {
				'_' | '0'..='9' => self.skip(),
				c if c.is_ascii_lowercase() => self.skip(),
				_ => break,
			}
		}

		Ok(Token::Ident(self.slice()))
	}

	fn lex_const_identifier(&mut self) -> Result<Token<'input>, ParsingError> {
		let c = self.pop();
		debug_assert!(c.is_ascii_uppercase() || matches!(c, '_'));

		loop {
			match self.read() {
				'_' | '0'..='9' => self.skip(),
				c if c.is_ascii_uppercase() => self.skip(),
				_ => break,
			}
		}

		Ok(Token::ConstantIdent(self.slice()))
	}

	fn lex_number(&mut self) -> Result<Token<'input>, ParsingError> {
		let c = self.read();
		debug_assert!(c.is_ascii_digit());

		while let '0'..='9' = self.read() {
			self.skip();
		}

		self.slice()
			.parse::<u64>()
			.map(Token::Int)
			.map_err(|error| ParsingError::InvalidLiteral {
				span: self.span(),
				kind: int_error_kind_to_literal_error_kind(
					error.kind(),
					LiteralErrorKind::FeltOverflow,
				),
			})
	}

	fn lex_hex(&mut self) -> Result<Token<'input>, ParsingError> {
		debug_assert!(self.read().is_ascii_hexdigit());

		loop {
			let c1 = self.read();
			if !c1.is_ascii_hexdigit() {
				break;
			}
			self.skip();

			let c2 = self.read();
			if !c2.is_ascii_hexdigit() {
				return Err(ParsingError::InvalidHexLiteral {
					span: self.span(),
					kind: HexErrorKind::Invalid,
				});
			}
			self.skip();
		}

		let span = self.span();
		let start = span.start();
		let end = span.end();
		let digit_start = start.to_u32() + 2;
		let span = SourceSpan::new(span.source_id(), start..end);
		let value = parse_hex(span, self.slice_span(digit_start..end.to_u32()))?;
		Ok(Token::HexValue(value))
	}

	fn lex_bin(&mut self) -> Result<Token<'input>, ParsingError> {
		debug_assert!(is_ascii_binary(self.read()));

		loop {
			let c1 = self.read();
			if !is_ascii_binary(c1) {
				break;
			}
			self.skip();
		}

		let span = self.span();
		let start = span.start();
		let digit_start = start.to_u32() + 2;
		let end = span.end();
		let span = SourceSpan::new(span.source_id(), start..end);
		let value = parse_bin(span, self.slice_span(digit_start..end.to_u32()))?;
		Ok(Token::BinValue(value))
	}
}

fn parse_hex(span: SourceSpan, hex_digits: &str) -> Result<HexEncodedValue, ParsingError> {
	match hex_digits.len() {
		n if n <= 16 && matches!(n % 2, 0) => {
			let value = u64::from_str_radix(hex_digits, 16).map_err(|error| {
				ParsingError::InvalidLiteral {
					span,
					kind: int_error_kind_to_literal_error_kind(
						error.kind(),
						LiteralErrorKind::FeltOverflow,
					),
				}
			})?;
			if value > Felt::MODULUS {
				Err(ParsingError::InvalidLiteral {
					span,
					kind: LiteralErrorKind::FeltOverflow,
				})
			} else {
				Ok(shrink_u64_hex(value))
			}
		}
		64 => {
			let mut word = [Felt::ZERO; 4];
			for (index, element) in word.iter_mut().enumerate() {
				let offset = index * 16;
				let mut felt_bytes = [0u8; 8];
				let digits = &hex_digits[offset..(offset + 16)];
				for (byte_idx, byte) in felt_bytes.iter_mut().enumerate() {
					let byte_str = &digits[(byte_idx * 2)..((byte_idx * 2) + 2)];
					*byte = u8::from_str_radix(byte_str, 16).map_err(|error| {
						ParsingError::InvalidLiteral {
							span,
							kind: int_error_kind_to_literal_error_kind(
								error.kind(),
								LiteralErrorKind::FeltOverflow,
							),
						}
					})?;
				}

				let value = u64::from_le_bytes(felt_bytes);
				if value > Felt::MODULUS {
					return Err(ParsingError::InvalidLiteral {
						span,
						kind: LiteralErrorKind::FeltOverflow,
					});
				}
				*element = Felt::new(value);
			}
			Ok(HexEncodedValue::Word(word))
		}
		65.. => Err(ParsingError::InvalidHexLiteral {
			span,
			kind: HexErrorKind::TooLong,
		}),
		n if !matches!(n % 2, 0) && n < 64 => Err(ParsingError::InvalidHexLiteral {
			span,
			kind: HexErrorKind::MissingDigits,
		}),
		_ => Err(ParsingError::InvalidHexLiteral {
			span,
			kind: HexErrorKind::Invalid,
		}),
	}
}

impl<'input> Iterator for Lexer<'input> {
	type Item = Lexed<'input>;

	fn next(&mut self) -> Option<Self::Item> {
		let mut res = self.lex();
		while let Some(Ok((_, Token::Comment, _))) = res {
			res = self.lex();
		}
		res
	}
}

fn parse_bin(span: SourceSpan, bin_digits: &str) -> Result<BinEncodedValue, ParsingError> {
	if bin_digits.len() <= 32 {
		let value =
			u32::from_str_radix(bin_digits, 2).map_err(|error| ParsingError::InvalidLiteral {
				span,
				kind: int_error_kind_to_literal_error_kind(
					error.kind(),
					LiteralErrorKind::U32Overflow,
				),
			})?;
		Ok(shrink_u32_bin(value))
	} else {
		Err(ParsingError::InvalidBinaryLiteral {
			span,
			kind: BinErrorKind::TooLong,
		})
	}
}

const fn is_ascii_binary(c: char) -> bool {
	matches!(c, '0'..='1')
}

const fn shrink_u64_hex(n: u64) -> HexEncodedValue {
	if n <= (u8::MAX as u64) {
		HexEncodedValue::U8(n as u8)
	} else if n <= (u16::MAX as u64) {
		HexEncodedValue::U16(n as u16)
	} else if n <= (u32::MAX as u64) {
		HexEncodedValue::U32(n as u32)
	} else {
		HexEncodedValue::Felt(Felt::new(n))
	}
}

const fn shrink_u32_bin(n: u32) -> BinEncodedValue {
	if n <= (u8::MAX as u32) {
		BinEncodedValue::U8(n as u8)
	} else if n <= (u16::MAX as u32) {
		BinEncodedValue::U16(n as u16)
	} else {
		BinEncodedValue::U32(n)
	}
}

fn int_error_kind_to_literal_error_kind(
	kind: &IntErrorKind,
	overflow: LiteralErrorKind,
) -> LiteralErrorKind {
	match kind {
		IntErrorKind::Empty => LiteralErrorKind::Empty,
		IntErrorKind::InvalidDigit => LiteralErrorKind::InvalidDigit,
		IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => overflow,
		_ => unreachable!(),
	}
}
