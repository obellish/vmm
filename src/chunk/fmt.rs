use std::fmt::{Debug, Formatter, Result as FmtResult, Write as _};

use super::Chunk;
use crate::{OpCode, Value};

#[expect(clippy::missing_fields_in_debug)]
impl Debug for Chunk {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let mut fmt = f.debug_struct("Chunk");

		fmt.finish()?;

		f.write_char(' ')?;

		let mut l = f.debug_list();

		for op in &self.code {
			match op {
				OpCode::Constant(idx) => {
					l.entry(&Constant(self.constants.get(*idx).copied()));
				}
				_ => {
					l.entry(&op);
				}
			}
		}

		l.finish()
	}
}

#[repr(transparent)]
struct Constant(Option<Value>);

impl Debug for Constant {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Constant(")?;

		if let Some(value) = self.0 {
			Debug::fmt(&value, f)?;
		} else {
			f.write_str("<unknown>")?;
		}

		f.write_char(')')
	}
}
