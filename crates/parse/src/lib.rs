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
use tracing::{debug, info, trace, trace_span};
use vmm_ir::Instruction;

pub use self::opcode::*;

#[derive(Debug, Clone)]
pub struct Parser<'source> {
	inner: Lexer<'source, OpCode>,
}

impl<'source> Parser<'source> {
	#[inline]
	#[must_use]
	pub fn new(source: &'source <OpCode as Logos<'source>>::Source) -> Self {
		debug!("got source with length {}", source.len());
		Self {
			inner: Lexer::new(source),
		}
	}

	#[inline]
	#[tracing::instrument(skip(self))]
	pub fn scan(self) -> Result<Vec<Instruction>, ParseError> {
		info!("scanning {} chars", self.inner.source().len());

		let mut parsed = parse(self.inner.filter_map(Result::ok), 0)?;

		parsed.insert(0, Instruction::Boundary);

		parsed.push(Instruction::Boundary);

		Ok(parsed)
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

#[inline]
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

	opcodes
		.iter()
		.copied()
		.enumerate()
		.try_for_each(|(i, op)| {
			if matches!(loop_stack, 0) {
				if let Some(instr) = match op {
					OpCode::Increment => Some(Instruction::inc_val(1)),
					OpCode::Decrement => Some(Instruction::inc_val(-1)),
					OpCode::Output => Some(Instruction::write_once()),
					OpCode::MoveRight => Some(Instruction::move_ptr(1)),
					OpCode::MoveLeft => Some(Instruction::move_ptr(-1)),
					OpCode::Input => Some(Instruction::read()),
					OpCode::JumpRight => {
						loop_start = i;
						loop_stack += 1;
						None
					}
					OpCode::JumpLeft => return Err(ParseError::UnmatchedBracket(i)),
				} {
					trace!(parent: &span, "got instruction {op}");
					program.push(instr);
				}
			} else {
				match op {
					OpCode::JumpRight => loop_stack += 1,
					OpCode::JumpLeft => {
						loop_stack -= 1;
						if matches!(loop_stack, 0) {
							program.push(Instruction::dynamic_loop(parse(
								opcodes[loop_start + 1..i].iter().copied(),
								depth + 1,
							)?));
						}
					}
					_ => {}
				}
			}

			Ok(())
		})?;

	drop(guard);

	Ok(program)
}
