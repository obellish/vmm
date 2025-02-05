use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use vmm_core::prettier::{Document, PrettyPrint, display};

use crate::ast::{ImmU8, ImmU16, ImmU32};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugOptions {
	StackAll,
	StackTop(ImmU8),
	MemAll,
	MemInterval(ImmU32, ImmU32),
	LocalInterval(ImmU16, ImmU16),
	LocalRangeFrom(ImmU16),
	LocalAll,
}

impl Display for DebugOptions {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::StackAll => f.write_str("stack"),
			Self::StackTop(n) => {
				f.write_str("stack.")?;
				Display::fmt(n, f)
			}
			Self::MemAll => f.write_str("mem"),
			Self::MemInterval(n, m) => {
				f.write_str("mem.")?;
				Display::fmt(&n, f)?;
				f.write_char('.')?;
				Display::fmt(&m, f)
			}
			Self::LocalAll => f.write_str("local"),
			Self::LocalRangeFrom(start) => {
				f.write_str("local.")?;
				Display::fmt(&start, f)
			}
			Self::LocalInterval(start, end) => {
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

impl TryFrom<DebugOptions> for vmm_core::DebugOptions {
	type Error = ();

	fn try_from(value: DebugOptions) -> Result<Self, Self::Error> {
		match value {
			DebugOptions::StackAll => Ok(Self::StackAll),
			DebugOptions::StackTop(ImmU8::Value(n)) => Ok(Self::StackTop(n.into_inner())),
			DebugOptions::MemAll => Ok(Self::MemAll),
			DebugOptions::MemInterval(ImmU32::Value(start), ImmU32::Value(end)) => {
				Ok(Self::MemInterval(start.into_inner(), end.into_inner()))
			}
			DebugOptions::LocalInterval(ImmU16::Value(start), ImmU16::Value(end)) => {
				let start = start.into_inner();
				let end = end.into_inner();
				Ok(Self::LocalInterval(start, end, end - start))
			}
			_ => Err(()),
		}
	}
}
