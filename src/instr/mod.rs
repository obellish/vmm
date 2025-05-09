mod parse;
mod stack;
#[cfg(test)]
mod tests;

use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

pub use self::{parse::*, stack::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	Stacked(StackedInstruction),
	SetVal(u8),
	MoveVal { offset: isize, multiplier: u8 },
	FindZero(isize),
	Read,
	RawLoop(Vec<Self>),
}

impl Instruction {
	#[must_use]
	pub const fn needs_input(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_stacked(&self) -> bool {
		matches!(self, Self::Stacked(_))
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
	pub fn count(&self) -> usize {
		match self {
			Self::RawLoop(l) => l.iter().map(Self::count).sum::<usize>() + 2,
			_ => 1,
		}
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
		matches!(self, Self::RawLoop(x) if x.is_empty())
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Stacked(s) => Display::fmt(&s, f)?,
			Self::FindZero(i) => {
				f.write_char('[')?;
				let c = if *i > 0 { '>' } else { '<' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
				f.write_char(']')?;
			}
			Self::Read => f.write_char(',')?,
			Self::RawLoop(instructions) => {
				f.write_char('[')?;
				for instr in instructions {
					Display::fmt(&instr, f)?;
				}
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
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl From<StackedInstruction> for Instruction {
	fn from(value: StackedInstruction) -> Self {
		Self::Stacked(value)
	}
}
