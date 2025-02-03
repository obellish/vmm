use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use crate::prettier::{Document, PrettyPrint, display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOptions {
	StackAll,
	StackTop(u8),
	MemAll,
	MemInterval(u32, u32),
	LocalInterval(u16, u16, u16),
}

impl Display for DebugOptions {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::StackAll => f.write_str("stack"),
			Self::StackTop(n) => {
				f.write_str("stack.")?;
				Display::fmt(&n, f)
			}
			Self::MemAll => f.write_str("mem"),
			Self::MemInterval(n, m) => {
				f.write_str("mem.")?;
				Display::fmt(&n, f)?;
				f.write_char('.')?;
				Display::fmt(&m, f)
			}
			Self::LocalInterval(start, end, ..) => {
				f.write_str("local.")?;
				Display::fmt(&start, f)?;
				f.write_char('.')?;
				Display::fmt(&end, f)
			}
		}
	}
}

impl PrettyPrint for DebugOptions {
	fn render(&self) -> Document {
		display(self)
	}
}
