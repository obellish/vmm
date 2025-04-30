use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellAccess {
	#[default]
	Unused,
	NonZero,
	Set(u8),
}
