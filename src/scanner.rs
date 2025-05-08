use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
};

use logos::{Lexer, Logos};

use super::{Instruction, OpCode};

#[derive(Debug, Clone)]
pub struct Scanner<'source> {
	inner: Lexer<'source, OpCode>,
}

impl<'source> Scanner<'source> {
	#[must_use]
	pub fn new(source: &'source <OpCode as Logos<'source>>::Source) -> Self {
		Self {
			inner: Lexer::new(source),
		}
	}

	pub fn scan(self) -> Result<impl Iterator<Item = Instruction>, ScannerError> {
		parse(self.inner.filter_map(Result::ok))
	}
}

#[derive(Debug, Clone)]
pub enum ScannerError {
	UnmatchedBracket(usize),
}

impl Display for ScannerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::UnmatchedBracket(index) => {
				f.write_str("loop ending at #")?;
				Display::fmt(&index, f)?;
				f.write_str(" has no beginning")?;
			}
		}

		Ok(())
	}
}

impl StdError for ScannerError {}

fn parse(
	opcodes: impl IntoIterator<Item = OpCode>,
) -> Result<impl Iterator<Item = Instruction>, ScannerError> {
	let opcodes = opcodes.into_iter().collect::<Vec<_>>();
	let mut program = Vec::new();
	let mut loop_stack = 0;
	let mut loop_start = 0;

	for (i, op) in opcodes.iter().copied().enumerate() {
		if matches!(loop_stack, 0) {
			if let Some(instr) = match op {
				OpCode::Increment => Some(Instruction::IncVal(1)),
				OpCode::Decrement => Some(Instruction::IncVal(-1)),
				OpCode::Input => Some(Instruction::Read),
				OpCode::Output => Some(Instruction::Write),
				OpCode::MoveRight => Some(Instruction::MovePtr(1)),
				OpCode::MoveLeft => Some(Instruction::MovePtr(-1)),
				OpCode::JumpRight => {
					loop_start = i;
					loop_stack += 1;
					None
				}
				OpCode::JumpLeft => return Err(ScannerError::UnmatchedBracket(i)),
			} {
				program.push(instr);
			}
		} else {
			match op {
				OpCode::JumpRight => loop_stack += 1,
				OpCode::JumpLeft => {
					loop_stack -= 1;

					if matches!(loop_stack, 0) {
						program.push(Instruction::RawLoop(
							parse(opcodes[loop_start + 1..i].iter().copied())?.collect(),
						));
					}
				}
				_ => {}
			}
		}
	}

	Ok(program.into_iter())
}
