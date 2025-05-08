use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
	#[repr(transparent)]
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
	pub struct CellState: u8 {
		const TOUCHED = 0b0000_0001;
		const IN_LOOP = 0b0000_0010;
		const WRITTEN = 0b0000_0100;
	}
}

impl CellState {

}
