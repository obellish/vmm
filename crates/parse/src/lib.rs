#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod opcode;

use alloc::vec::Vec;
use core::{
	error::Error as StdError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use logos::{Lexer, Logos};
use tracing::{info, trace, trace_span};
use vmm_ir::Instruction;

pub use self::opcode::*;

#[derive(Debug, Clone)]
pub struct Parser<'source> {
	inner: Lexer<'source, OpCode>,
}

impl<'source> Parser<'source> {
	#[must_use]
	pub fn new(source: &'source <OpCode as Logos<'source>>::Source) -> Self {
		Self {
			inner: Lexer::new(source),
		}
	}

	#[tracing::instrument(skip(self))]
	pub fn scan(self) -> Result<impl Iterator<Item = Instruction>, ParseError> {
		info!("scanning {} chars", self.inner.source().len());
		parse(self.inner.filter_map(Result::ok), 0).map(IntoIterator::into_iter)
	}
}

#[derive(Debug)]
pub enum ParseError {
	UnmatchedBracket(usize),
}

impl Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::UnmatchedBracket(index) => {
				f.write_str("loop ending at #")?;
				Display::fmt(&index, f)?;
				f.write_str(" has no beginning")
			}
		}
	}
}

impl StdError for ParseError {}

fn parse(
	opcodes: impl Iterator<Item = OpCode>,
	depth: usize,
) -> Result<Vec<Instruction>, ParseError> {
	let span = trace_span!("parse", depth);

	let guard = span.enter();

	let opcodes = opcodes.into_iter().collect::<Vec<_>>();
	let mut program = Vec::new();
	let mut loop_stack = 0;
	let mut loop_start = 0;

	for (i, op) in opcodes.iter().copied().enumerate() {
		if matches!(loop_stack, 0) {
			if let Some(instr) = match op {
				OpCode::Increment => Some(Instruction::inc_val(1)),
				OpCode::Decrement => Some(Instruction::inc_val(-1)),
				OpCode::Output => Some(Instruction::Write),
				OpCode::MoveRight => Some(Instruction::MovePtr(1isize.into())),
				OpCode::MoveLeft => Some(Instruction::MovePtr((-1isize).into())),
				OpCode::Input => Some(Instruction::Read),
				OpCode::JumpRight => {
					loop_start = i;
					loop_stack += 1;
					None
				}
				OpCode::JumpLeft => return Err(ParseError::UnmatchedBracket(i)),
			} {
				trace!(parent: &span, "got instruction {instr}");
				program.push(instr);
			}
		} else {
			match op {
				OpCode::JumpRight => loop_stack += 1,
				OpCode::JumpLeft => {
					loop_stack -= 1;
					if matches!(loop_stack, 0) {
						program.push(Instruction::RawLoop(parse(
							opcodes[loop_start + 1..i].iter().copied(),
							depth + 1,
						)?));
					}
				}
				_ => {}
			}
		}
	}

	drop(guard);

	Ok(program)
}
