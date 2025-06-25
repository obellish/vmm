use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
pub enum OpCode {
	#[token("<")]
	MoveLeft,
	#[token(">")]
	MoveRight,
	#[token("+")]
	Increment,
	#[token("-")]
	Decrement,
	#[token(",")]
	Input,
	#[token(".")]
	Output,
	#[token("]")]
	JumpLeft,
	#[token("[")]
	JumpRight,
}

impl Display for OpCode {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char(match self {
			Self::MoveLeft => '<',
			Self::MoveRight => '>',
			Self::Increment => '+',
			Self::Decrement => '-',
			Self::Input => ',',
			Self::Output => '.',
			Self::JumpLeft => ']',
			Self::JumpRight => '[',
		})
	}
}
