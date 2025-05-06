mod parse;
mod simd;

use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::{parse::*, simd::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	MovePtr(isize),
	Add(i8),
	Set(u8),
	FindZero(isize),
	Write,
	Read,
	Loop(Vec<Self>),
	Simd(SimdInstruction),
}

impl Instruction {
	#[must_use]
	pub const fn needs_input(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_set(&self) -> bool {
		matches!(self, Self::Set(_) | Self::Simd(SimdInstruction::Set { .. }))
	}

	#[must_use]
	pub const fn is_clear(&self) -> bool {
		matches!(
			self,
			Self::Set(0) | Self::Simd(SimdInstruction::Set { value: 0, .. })
		)
	}

	#[must_use]
	pub fn count(&self) -> usize {
		match self {
			Self::Loop(l) => l.len(),
			Self::Add(i) => i.unsigned_abs() as usize,
			Self::MovePtr(i) => i.unsigned_abs(),
			Self::Set(i) => *i as usize,
			_ => 1,
		}
	}

	#[must_use]
	pub fn is_empty_loop(&self) -> bool {
		matches!(self, Self::Loop(x) if x.is_empty())
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Add(i) => {
				let c = if *i > 0 { '+' } else { '-' };

				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::MovePtr(i) => {
				let c = if *i > 0 { '>' } else { '<' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::FindZero(i) => {
				f.write_char('[')?;
				let c = if *i > 0 { '>' } else { '<' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
				f.write_char(']')?;
			}
			Self::Read => f.write_char(',')?,
			Self::Write => f.write_char('.')?,
			Self::Set(0) => f.write_str("[-]")?,
			Self::Set(i) => {
				for _ in 0..(*i) {
					f.write_char('+')?;
				}
			}
			Self::Loop(instructions) => {
				f.write_char('[')?;
				for instr in instructions {
					Display::fmt(&instr, f)?;
				}
				f.write_char(']')?;
			}
			Self::Simd(s) => Display::fmt(&s, f)?,
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}
