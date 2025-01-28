#[cfg(feature = "serde")]
mod serde;

use alloc::{collections::BTreeMap, vec::Vec};
use core::borrow::Borrow;

use super::mmr::Mmr;
use crate::{
	hash::rpo::RpoDigest,
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
		collections::{KvMap, RecordingMap},
	},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StoreNode {
	left: RpoDigest,
	right: RpoDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct MerkleStore<T = BTreeMap<RpoDigest, StoreNode>>
where
	T: KvMap<RpoDigest, StoreNode>,
{
	nodes: T,
}
