mod parse;

use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::parse::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	MovePtr(isize),
	Inc(i8),
	Set(u8),
	FindZero(isize),
	Write,
	Read,
	Loop(Vec<Self>),
}

impl Instruction {
	#[must_use]
	pub const fn needs_input(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_set(&self) -> bool {
		matches!(self, Self::Set(_))
	}

	#[must_use]
	pub const fn is_clear(&self) -> bool {
		matches!(self, Self::Set(0))
	}

	#[must_use]
	pub fn count(&self) -> usize {
		match self {
			Self::Loop(l) => l.len(),
			Self::Inc(i) => i.unsigned_abs() as usize,
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
			Self::Inc(i) => {
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
			Self::Set(i) => {
				f.write_str("[-]")?;
				if *i > 0 {
					for _ in 0..(*i) {
						f.write_char('+')?;
					}
				}
			}
			Self::Loop(instructions) => {
				f.write_char('[')?;
				for instr in instructions {
					Display::fmt(&instr, f)?;
				}
				f.write_char(']')?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}
