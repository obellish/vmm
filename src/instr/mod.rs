mod parse;

use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::parse::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	Move(isize),
	Add(i8),
	Set(u8),
	FindZero(isize),
	Write,
	Read,
	Loop(Vec<Self>),
	JumpToCell(usize),
}

impl Instruction {
	#[must_use]
	pub const fn needs_input(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub fn len(&self) -> usize {
		match self {
			Self::Loop(l) => l.len(),
			Self::Add(i) => i.unsigned_abs() as usize,
			Self::Move(i) => i.unsigned_abs(),
			Self::Set(i) => *i as usize,
			_ => 1,
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.len(), 0)
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
			Self::Move(i) => {
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
				if matches!(*i, 0) {
					f.write_str("[-]")?;
				} else {
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
