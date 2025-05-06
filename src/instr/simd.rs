use std::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimdInstruction {
	Set { len: usize, value: u8 },
}

#[expect(clippy::trivially_copy_pass_by_ref)]
impl SimdInstruction {
	#[must_use]
	pub const fn is_set(&self) -> bool {
		matches!(self, Self::Set { .. })
	}

	#[must_use]
	pub const fn is_clear(&self) -> bool {
		matches!(self, Self::Set { value: 0, .. })
	}
}

#[allow(clippy::match_wildcard_for_single_variants)]
impl Display for SimdInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Set { value: 0, len } => {
				for _ in 0..(*len - 1) {
					f.write_str("[-]>")?;
				}

				f.write_str("[-]")?;
			}
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}
