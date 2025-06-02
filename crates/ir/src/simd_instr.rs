use alloc::{
	borrow::ToOwned,
	string::{String, ToString},
	vec::Vec,
};
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};
use vmm_utils::GetOrZero;

use super::{IsZeroingCell, Offset, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SimdInstruction {
	IncVals {
		value: i8,
		offsets: Vec<Option<Offset>>,
	},
	SetVals {
		value: Option<NonZeroU8>,
		offsets: Vec<Option<Offset>>,
	},
}

impl SimdInstruction {
	#[must_use]
	pub const fn inc_vals(value: i8, offsets: Vec<Option<Offset>>) -> Self {
		Self::IncVals { value, offsets }
	}

	#[must_use]
	pub const fn set_vals(value: u8, offsets: Vec<Option<Offset>>) -> Self {
		Self::SetVals {
			value: NonZeroU8::new(value),
			offsets,
		}
	}
}

#[allow(unreachable_patterns)]
impl Display for SimdInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::IncVals { value, offsets } => {
				f.write_str("simd(inc) ")?;
				Display::fmt(&value, f)?;
				f.write_str(" [")?;
				let offset_str = format_offsets(offsets);

				f.write_str(&offset_str)?;
				f.write_char(']')?;
			}
			Self::SetVals { value, offsets } => {
				f.write_str("simd(set) ")?;
				Display::fmt(&value.get_or_zero(), f)?;
				f.write_str(" [")?;
				let offset_str = format_offsets(offsets);

				f.write_str(&offset_str)?;
				f.write_char(']')?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl IsZeroingCell for SimdInstruction {
	#[inline]
	fn is_zeroing_cell(&self) -> bool {
		let Self::SetVals {
			value: None,
			offsets,
		} = self
		else {
			return false;
		};

		offsets.contains(&None)
	}
}

impl PtrMovement for SimdInstruction {
	#[inline]
	fn ptr_movement(&self) -> Option<isize> {
		Some(0)
	}
}

fn format_offsets(offsets: &[Option<Offset>]) -> String {
	offsets
		.iter()
		.map(|offset| match offset {
			None => "0".to_owned(),
			Some(o) => o.to_string(),
		})
		.collect::<Vec<_>>()
		.join(", ")
}
