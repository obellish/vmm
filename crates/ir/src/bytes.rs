use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Bytes {
	Single(u8),
	Many(Vec<u8>),
}

impl From<u8> for Bytes {
	fn from(value: u8) -> Self {
		Self::Single(value)
	}
}

impl From<Vec<u8>> for Bytes {
	fn from(value: Vec<u8>) -> Self {
		Self::from_iter(value)
	}
}

impl<const N: usize> From<[u8; N]> for Bytes {
	fn from(value: [u8; N]) -> Self {
		Self::from_iter(value)
	}
}

impl FromIterator<u8> for Bytes {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = u8>,
	{
		Self::Many(iter.into_iter().collect())
	}
}
