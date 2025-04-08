#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod storage;

use serde::{Deserialize, Serialize};
use vmm_blocks::{
	BlockPos,
	blocks::{Block, entities::BlockEntity},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TickPriority {
	Highest,
	Higher,
	High,
	Normal,
}
