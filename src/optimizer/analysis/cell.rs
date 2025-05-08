use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CellState {
	/// The cell is not touched once during program execution
	Untouched,
	/// The cell is touched during program execution
	Touched,
	/// The cell is touched within a loop during program execution
	InLoop,
	/// The cell is written to as an input
	Written,
}

impl CellState {
	#[must_use]
	pub const fn is_untouched(self) -> bool {
		matches!(self, Self::Untouched)
	}

	#[must_use]
	pub const fn is_touched(self) -> bool {
		!self.is_untouched()
	}

	#[must_use]
	pub const fn is_touched_in_loop(self) -> bool {
		matches!(self, Self::InLoop)
	}
}

impl Debug for CellState {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for CellState {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Untouched => f.write_char('_')?,
			Self::Touched => f.write_char('*')?,
			Self::InLoop => f.write_char('0')?,
			Self::Written => f.write_char('w')?,
		}

		Ok(())
	}
}
