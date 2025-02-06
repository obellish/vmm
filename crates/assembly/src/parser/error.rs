use alloc::{
	borrow::ToOwned,
	string::{String, ToString},
	vec::Vec,
};
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::Range,
};

use thiserror::Error;

use super::{ParseError, Token};
use crate::{SourceId, SourceSpan, diagnostics::Diagnostic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralErrorKind {
	Empty,
	InvalidDigit,
	U32Overflow,
	FeltOverflow,
	InvalidBitSize,
}

impl Display for LiteralErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Empty => "input was empty",
			Self::InvalidDigit => "invalid digit",
			Self::U32Overflow => "value overflowed the u32 range",
			Self::FeltOverflow => "value overflowed the field modulus",
			Self::InvalidBitSize => "expected value to be a valid bit size, e.g. 0..63",
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexErrorKind {
	MissingDigits,
	Invalid,
	Overflow,
	TooLong,
}

impl Display for HexErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::MissingDigits => "expected number of hex digits to be a multiple of 2",
			Self::Invalid => "expected 2, 4, 8, 16, or 64 hex digits",
			Self::Overflow => "value overflowed the field modulus",
			Self::TooLong => {
				"value has too many digits, long hex strings must contain exactly 64 digits"
			}
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinErrorKind {
	TooLong,
}

impl Display for BinErrorKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::TooLong => {
				"value has too many digits, binary string can contain no more than 32 digits"
			}
		})
	}
}

#[derive(Debug, Default, Error, Diagnostic)]
#[repr(u8)]
pub enum ParsingError {
	#[default]
	#[error("parsing failed due to unexpected input")]
	#[diagnostic()]
	Failed = 0,
	#[error("expected input to be valid utf8, but invalid byte sequences were found")]
	#[diagnostic()]
	InvalidUtf8 {
		#[label("invalid byte sequence starts here")]
		span: SourceSpan,
	},
	#[error(
		"expected input to be valid utf8, but end-of-file was reached before final codepoint was read"
	)]
	#[diagnostic()]
	IncompleteUtf8 {
		#[label("the codepoint starting here is incomplete")]
		span: SourceSpan,
	},
	#[error("invalid syntax")]
	#[diagnostic()]
	InvalidToken {
		#[label("occurs here")]
		span: SourceSpan,
	},
	#[error("invalid syntax")]
	#[diagnostic(help("expected {}", expected.as_slice().join(", or ")))]
	UnrecognizedToken {
		#[label("found a {token} here")]
		span: SourceSpan,
		token: String,
		expected: Vec<String>,
	},
	#[error("unexpected trailing tokens")]
	#[diagnostic()]
	ExtraToken {
		#[label("{token} was found here, but was not expected")]
		span: SourceSpan,
		token: String,
	},
	#[error("unexpected end of file")]
	#[diagnostic(help("expected {}", expected.as_slice().join(", or ")))]
	UnrecognizedEof {
		#[label("reached end of file here")]
		span: SourceSpan,
		expected: Vec<String>,
	},
	#[error("invalid character in identifier")]
	#[diagnostic(help(
		"bare identifiers must be lowercase alphanumeric with '_', quoted identifiers can include uppercase, as well as '.' and '$'"
	))]
	InvalidIdentCharacter {
		#[label]
		span: SourceSpan,
	},
	#[error("unclosed quoted identifier")]
	#[diagnostic()]
	UnclosedQuote {
		#[label("no match for quotation mark starting here")]
		start: SourceSpan,
	},
	#[error("too many instructions in a single code block")]
	#[diagnostic()]
	CodeBlockTooBig {
		#[label]
		span: SourceSpan,
	},
	#[error("invalid constant expression: division by zero")]
	DivisionByZero {
		#[label]
		span: SourceSpan,
	},
	#[error("doc comment is too large")]
	#[diagnostic(help("make sure it is less than u16::MAX bytes in length"))]
	DocsTooLarge {
		#[label]
		span: SourceSpan,
	},
	#[error("invalid literal: {}", kind)]
	#[diagnostic()]
	InvalidLiteral {
		#[label]
		span: SourceSpan,
		kind: LiteralErrorKind,
	},
	#[error("invalid literal: {}", kind)]
	#[diagnostic()]
	InvalidHexLiteral {
		#[label]
		span: SourceSpan,
		kind: HexErrorKind,
	},
	#[error("invalid literal: {}", kind)]
	#[diagnostic()]
	InvalidBinaryLiteral {
		#[label]
		span: SourceSpan,
		kind: BinErrorKind,
	},
	#[error("invalid MAST root literal")]
	InvalidMastRoot {
		#[label]
		span: SourceSpan,
	},
	#[error("invalid library path: {}", message)]
	InvalidLibraryPath {
		#[label]
		span: SourceSpan,
		message: String,
	},
	#[error("invalid immediate: value must be in the range {}..{} (exclusive)", range.start, range.end)]
	ImmediateOutOfRange {
		#[label]
		span: SourceSpan,
		range: Range<usize>,
	},
	#[error("too many procedures in this module")]
	#[diagnostic()]
	ModuleTooLarge {
		#[label]
		span: SourceSpan,
	},
	#[error("too many re-exported procedures in this module")]
	#[diagnostic()]
	ModuleTooManyReexports {
		#[label]
		span: SourceSpan,
	},
	#[error(
		"too many operands for `push`: tried to push {} elements, but only 16 can be pushed at one time",
		count
	)]
	#[diagnostic()]
	PushOverflow {
		#[label]
		span: SourceSpan,
		count: usize,
	},
	#[error("expected a fully-qualified module path, e.g. `std::u64`")]
	UnqualifiedImport {
		#[label]
		span: SourceSpan,
	},
	#[error(
		"re-exporting a procedure identified by digest requires giving it a name, e.g. `export.DIGEST->foo`"
	)]
	UnnamedReexportOfMastRoot {
		#[label]
		span: SourceSpan,
	},
	#[error("conflicting attributes for procedure definition")]
	#[diagnostic()]
	AttributeConflict {
		#[label(
			"conflict occurs because an attribute with the same name has already been defined"
		)]
		span: SourceSpan,
		#[label("previously defined here")]
		prev: SourceSpan,
	},
	#[error("conflicting key-value attributes for procedure definition")]
	#[diagnostic()]
	AttributeKeyValueConflict {
		#[label(
			"conflict occurs because a key with the same name has already been set in a previous declaration"
		)]
		span: SourceSpan,
		#[label("previously defined here")]
		prev: SourceSpan,
	},
}

impl ParsingError {
	fn tag(&self) -> u8 {
		unsafe { *<*const _>::from(self).cast::<u8>() }
	}

	pub(crate) fn from_utf8_error(source_id: SourceId, err: core::str::Utf8Error) -> Self {
		let start = u32::try_from(err.valid_up_to()).ok().unwrap_or(u32::MAX);
		err.error_len().map_or_else(
			|| Self::IncompleteUtf8 {
				span: SourceSpan::at(source_id, start),
			},
			|len| Self::InvalidUtf8 {
				span: SourceSpan::new(source_id, start..(start + len as u32)),
			},
		)
	}

	pub(crate) fn from_parse_error(source_id: SourceId, err: ParseError<'_>) -> Self {
		match err {
			ParseError::InvalidToken { location: at } => Self::InvalidToken {
				span: SourceSpan::at(source_id, at),
			},
			ParseError::UnrecognizedToken {
				token: (l, Token::Eof, r),
				expected,
			} => Self::UnrecognizedEof {
				span: SourceSpan::new(source_id, l..r),
				expected: simplify_expected_tokens(expected).collect(),
			},
			ParseError::UnrecognizedToken {
				token: (l, tok, r),
				expected,
			} => Self::UnrecognizedToken {
				span: SourceSpan::new(source_id, l..r),
				token: tok.to_string(),
				expected: simplify_expected_tokens(expected).collect(),
			},
			ParseError::ExtraToken { token: (l, tok, r) } => Self::ExtraToken {
				span: SourceSpan::new(source_id, l..r),
				token: tok.to_string(),
			},
			ParseError::UnrecognizedEof {
				location: at,
				expected,
			} => Self::UnrecognizedEof {
				span: SourceSpan::new(source_id, at..at),
				expected: simplify_expected_tokens(expected).collect(),
			},
			ParseError::User { error } => error,
		}
	}
}

impl Eq for ParsingError {}

impl PartialEq for ParsingError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Failed, Self::Failed) => true,
			(Self::InvalidLiteral { kind: l, .. }, Self::InvalidLiteral { kind: r, .. }) => l == r,
			(Self::InvalidHexLiteral { kind: l, .. }, Self::InvalidHexLiteral { kind: r, .. }) => {
				l == r
			}
			(
				Self::InvalidLibraryPath { message: l, .. },
				Self::InvalidLibraryPath { message: r, .. },
			) => l == r,
			(
				Self::ImmediateOutOfRange { range: l, .. },
				Self::ImmediateOutOfRange { range: r, .. },
			) => l == r,
			(Self::PushOverflow { count: l, .. }, Self::PushOverflow { count: r, .. }) => l == r,
			(
				Self::UnrecognizedToken {
					token: ltok,
					expected: lexpect,
					..
				},
				Self::UnrecognizedToken {
					token: rtok,
					expected: rexpect,
					..
				},
			) => ltok == rtok && lexpect == rexpect,
			(Self::ExtraToken { token: ltok, .. }, Self::ExtraToken { token: rtok, .. }) => {
				ltok == rtok
			}
			(
				Self::UnrecognizedEof {
					expected: lexpect, ..
				},
				Self::UnrecognizedEof {
					expected: rexpect, ..
				},
			) => lexpect == rexpect,
			(x, y) => x.tag() == y.tag(),
		}
	}
}

fn simplify_expected_tokens(
	expected: impl IntoIterator<Item = String>,
) -> impl Iterator<Item = String> {
	let mut has_instruction = false;
	let mut has_ctrl = false;
	expected.into_iter().filter_map(move |t| {
		let tok = match t.as_str() {
			"bare_ident" => return Some("identifier".to_owned()),
			"const_ident" => return Some("constant identifier".to_owned()),
			"quoted_ident" => return Some("quoted identifier".to_owned()),
			"doc_comment" => return Some("doc comment".to_owned()),
			"hex_value" => return Some("hex-encoded literal".to_owned()),
			"bin_value" => return Some("bin-encoded literal".to_owned()),
			"uint" => return Some("integer literal".to_owned()),
			"EOF" => return Some("end of file".to_owned()),
			other => other[1..].strip_suffix('"').and_then(Token::parse),
		};
		match tok {
			Some(Token::If | Token::While | Token::Repeat) => {
				if has_ctrl {
					None
				} else {
					has_ctrl = true;
					Some("control flow opcode (e.g. \"if.true\")".to_owned())
				}
			}
			Some(tok) if tok.is_instruction() => {
				if has_instruction {
					None
				} else {
					has_instruction = true;
					Some("primitive opcode (e.g. \"add\")".to_owned())
				}
			}
			_ => Some(t),
		}
	})
}
