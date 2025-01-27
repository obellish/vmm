use alloc::vec::Vec;

use crate::hash::rpo::RpoDigest;

#[derive(Debug)]
pub struct MmrDelta {
	pub forest: usize,
	pub data: Vec<RpoDigest>,
}
