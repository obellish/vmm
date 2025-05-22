/// Simd (Single Instruction, Multiple Cells)
use serde::{Deserialize, Serialize};

use super::Offset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Simc {
	Clear {
		count: usize,
		offset: Option<Offset>,
	},
}
