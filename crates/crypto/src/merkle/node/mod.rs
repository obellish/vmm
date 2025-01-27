#[cfg(feature = "serde")]
mod serde;

use crate::hash::rpo::RpoDigest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerNodeInfo {
	pub value: RpoDigest,
	pub left: RpoDigest,
	pub right: RpoDigest,
}
