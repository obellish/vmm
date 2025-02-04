#[rustfmt::skip]
mod constants {
    pub const EVENT_MERKLE_NODE_MERGE: u32            = 276_124_218;
    pub const EVENT_MERKLE_NODE_TO_STACK: u32         = 361_943_238;
    pub const EVENT_MAP_VALUE_TO_STACK: u32           = 574_478_993;
    pub const EVENT_MAP_VALUE_TO_STACK_N: u32         = 630_847_990;
    pub const EVENT_U64_DIV: u32                      = 678_156_251;
    pub const EVENT_EXT2_INV: u32                     = 1_251_967_401;
    pub const EVENT_EXT2_INTT: u32                    = 1_347_499_010;
    pub const EVENT_SMT_PEEK: u32                     = 1_889_584_556;
    pub const EVENT_U32_CLZ: u32                      = 1_951_932_030;
    pub const EVENT_U32_CTZ: u32                      = 2_008_979_519;
    pub const EVENT_U32_CLO: u32                      = 2_032_895_094;
    pub const EVENT_U32_CTO: u32                      = 2_083_700_134;
    pub const EVENT_ILOG2: u32                        = 2_297_972_669;
    pub const EVENT_MEM_TO_MAP: u32                   = 2_389_394_361;
    pub const EVENT_HDWORD_TO_MAP: u32                = 2_391_452_729;
    pub const EVENT_HDWORD_TO_MAP_WITH_DOMAIN: u32    = 2_822_590_340;
    pub const EVENT_HPERM_TO_MAP: u32                 = 3_297_060_969;
    pub const EVENT_FALCON_SIG_TO_STACK: u32          = 3_419_226_139;
}

use core::fmt::{Display, Formatter, Result as FmtResult};

pub use self::constants::*;
use super::{
	SignatureKind,
	prettier::{Document, PrettyPrint, display},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemEvent {
	MerkleNodeMerge,
	MerkleNodeToStack,
	MapValueToStack,
	MapValueToStackN,
	U64Div,
	Ext2Inv,
	Ext2Intt,
	SmtPeek,
	U32Clz,
	U32Ctz,
	U32Clo,
	U32Cto,
	ILog2,
	MemToMap,
	HdwordToMap,
	HdwordToMapWithDomain,
	HpermToMap,
	FalconSigToStack,
}

impl SystemEvent {
	#[must_use]
	pub const fn into_event_id(self) -> u32 {
		match self {
			Self::MerkleNodeMerge => EVENT_MERKLE_NODE_MERGE,
			Self::MerkleNodeToStack => EVENT_MERKLE_NODE_TO_STACK,
			Self::MapValueToStack => EVENT_MAP_VALUE_TO_STACK,
			Self::MapValueToStackN => EVENT_MAP_VALUE_TO_STACK_N,
			Self::U64Div => EVENT_U64_DIV,
			Self::Ext2Inv => EVENT_EXT2_INV,
			Self::Ext2Intt => EVENT_EXT2_INTT,
			Self::SmtPeek => EVENT_SMT_PEEK,
			Self::U32Clz => EVENT_U32_CLZ,
			Self::U32Ctz => EVENT_U32_CTZ,
			Self::U32Clo => EVENT_U32_CLO,
			Self::U32Cto => EVENT_U32_CTO,
			Self::ILog2 => EVENT_ILOG2,
			Self::MemToMap => EVENT_MEM_TO_MAP,
			Self::HdwordToMap => EVENT_HDWORD_TO_MAP,
			Self::HdwordToMapWithDomain => EVENT_HDWORD_TO_MAP_WITH_DOMAIN,
			Self::HpermToMap => EVENT_HPERM_TO_MAP,
			Self::FalconSigToStack => EVENT_FALCON_SIG_TO_STACK,
		}
	}

	#[must_use]
	pub const fn from_event_id(event_id: u32) -> Option<Self> {
		match event_id {
			EVENT_MERKLE_NODE_MERGE => Some(Self::MerkleNodeMerge),
			EVENT_MERKLE_NODE_TO_STACK => Some(Self::MerkleNodeToStack),
			EVENT_MAP_VALUE_TO_STACK => Some(Self::MapValueToStack),
			EVENT_MAP_VALUE_TO_STACK_N => Some(Self::MapValueToStackN),
			EVENT_U64_DIV => Some(Self::U64Div),
			EVENT_EXT2_INV => Some(Self::Ext2Inv),
			EVENT_EXT2_INTT => Some(Self::Ext2Intt),
			EVENT_SMT_PEEK => Some(Self::SmtPeek),
			EVENT_U32_CLZ => Some(Self::U32Clz),
			EVENT_U32_CTZ => Some(Self::U32Ctz),
			EVENT_U32_CLO => Some(Self::U32Clo),
			EVENT_U32_CTO => Some(Self::U32Cto),
			EVENT_ILOG2 => Some(Self::ILog2),
			EVENT_MEM_TO_MAP => Some(Self::MemToMap),
			EVENT_HDWORD_TO_MAP => Some(Self::HdwordToMap),
			EVENT_HDWORD_TO_MAP_WITH_DOMAIN => Some(Self::HdwordToMapWithDomain),
			EVENT_HPERM_TO_MAP => Some(Self::HpermToMap),
			EVENT_FALCON_SIG_TO_STACK => Some(Self::FalconSigToStack),
			_ => None,
		}
	}
}

impl Display for SystemEvent {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::MerkleNodeMerge => f.write_str("merkle_node_merge"),
			Self::MerkleNodeToStack => f.write_str("merkle_node_to_stack"),
			Self::MapValueToStack => f.write_str("map_value_to_stack"),
			Self::MapValueToStackN => f.write_str("map_value_to_stack_with_len"),
			Self::U64Div => f.write_str("div_u64"),
			Self::Ext2Inv => f.write_str("ext2_inv"),
			Self::Ext2Intt => f.write_str("ext2_intt"),
			Self::SmtPeek => f.write_str("smt_peek"),
			Self::U32Clz => f.write_str("u32clz"),
			Self::U32Ctz => f.write_str("u32ctz"),
			Self::U32Clo => f.write_str("u32clo"),
			Self::U32Cto => f.write_str("u32cto"),
			Self::ILog2 => f.write_str("ilog2"),
			Self::MemToMap => f.write_str("mem_to_map"),
			Self::HdwordToMap => f.write_str("hdword_to_map"),
			Self::HdwordToMapWithDomain => f.write_str("hdword_to_map_with_domain"),
			Self::HpermToMap => f.write_str("hperm_to_map"),
			Self::FalconSigToStack => {
				f.write_str("sig_to_stack.")?;
				Display::fmt(&SignatureKind::RpoFalcon512, f)
			}
		}
	}
}

impl PrettyPrint for SystemEvent {
	fn render(&self) -> Document {
		display(self)
	}
}
