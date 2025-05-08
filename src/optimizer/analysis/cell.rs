use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CellState {
	Untouched,
	Touched,
	InLoop,
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
		}

		Ok(())
	}
}
