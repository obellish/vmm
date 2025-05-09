use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackedInstruction {
	IncVal(i8),
	MovePtr(isize),
	Write(usize),
}

impl StackedInstruction {
	#[must_use]
	pub const fn is_inc_val(self) -> bool {
		matches!(self, Self::IncVal(_))
	}

	#[must_use]
	pub const fn is_move_ptr(self) -> bool {
		matches!(self, Self::MovePtr(_))
	}

	#[must_use]
	pub const fn is_write(self) -> bool {
		matches!(self, Self::Write(_))
	}
}

impl Display for StackedInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::IncVal(i) => {
				let c = if i > 0 { '+' } else { '-' };

				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::MovePtr(i) => {
				let c = if i > 0 { '>' } else { '<' };
				for _ in 0..i.unsigned_abs() {
					f.write_char(c)?;
				}
			}
			Self::Write(i) => {
				for _ in 0..i {
					f.write_char('.')?;
				}
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}
