#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	IncVal(i8),
	SetVal(u8),
	MoveVal { offset: isize, multiplier: u8 },
	MovePtr(isize),
	FindZero(isize),
	Read,
	Write,
	RawLoop(Vec<Self>),
	ConstantLoop(u8, Vec<Self>),
}

impl Instruction {
	pub fn needs_input(&self) -> bool {
		match self {
			Self::Read => true,
			Self::RawLoop(instrs) => instrs.iter().any(Self::needs_input),
			_ => false,
		}
	}

	#[must_use]
	pub const fn is_inc_val(&self) -> bool {
		matches!(self, Self::IncVal(i) if *i > 0)
	}

	#[must_use]
	pub const fn is_dec_val(&self) -> bool {
		matches!(self, Self::IncVal(i) if *i < 0)
	}

	#[must_use]
	pub const fn is_set_val(&self) -> bool {
		matches!(self, Self::SetVal(_))
	}

	#[must_use]
	pub const fn is_clear_val(&self) -> bool {
		matches!(self, Self::SetVal(0))
	}

	#[must_use]
	pub const fn is_move_val(&self) -> bool {
		matches!(self, Self::MoveVal { .. })
	}

	#[must_use]
	pub const fn is_loop(&self) -> bool {
		matches!(self, Self::RawLoop(_))
	}

	#[must_use]
	pub fn is_empty_loop(&self) -> bool {
		matches!(self, Self::RawLoop(l) if l.is_empty())
	}

	#[must_use]
	pub const fn is_read(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_write(&self) -> bool {
		matches!(self, Self::Write)
	}

	pub fn rough_estimate(&self) -> usize {
		match self {
			Self::RawLoop(l) => l.iter().map(Self::rough_estimate).sum::<usize>() + 2,
			_ => 1,
		}
	}
}

impl Display for Instruction {
	#[expect(unreachable_patterns)] // For when we add more instructions.
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::IncVal(i) => {
				let c = if *i < 0 { '-' } else { '+' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::MovePtr(i) => {
				let c = if *i < 0 { '<' } else { '>' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::SetVal(i) => {
				f.write_str("[-]")?;
				if *i > 0 {
					for _ in 0..*i {
						f.write_char('+')?;
					}
				}
			}
			Self::FindZero(i) => {
				f.write_char('[')?;
				let c = if *i < 0 { '<' } else { '>' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
				f.write_char(']')?;
			}
			Self::Read => f.write_char(',')?,
			Self::Write => f.write_char('.')?,
			Self::RawLoop(instrs) => {
				f.write_char('[')?;
				display_loop(instrs, f)?;
				f.write_char(']')?;
			}
			Self::MoveVal { offset, multiplier } => {
				let (first_move, second_move) = if *offset < 0 { ('<', '>') } else { ('>', '<') };

				f.write_char('[')?;

				f.write_char('-')?;

				for _ in 0..offset.unsigned_abs() {
					f.write_char(first_move)?;
				}

				for _ in 0..*multiplier {
					f.write_char('+')?;
				}

				for _ in 0..offset.unsigned_abs() {
					f.write_char(second_move)?;
				}

				f.write_char(']')?;
			}
			Self::ConstantLoop(step, steps) => {
				Display::fmt(&Self::SetVal(*step), f)?;

				f.write_char('[')?;
				display_loop(steps, f)?;
				f.write_char(']')?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

fn display_loop(i: &[Instruction], f: &mut Formatter<'_>) -> FmtResult {
	for instr in i {
		Display::fmt(&instr, f)?;
	}

	Ok(())
}
