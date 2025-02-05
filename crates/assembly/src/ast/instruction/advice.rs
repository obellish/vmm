use core::fmt::{Display, Formatter, Result as FmtResult};

use vmm_core::{
	prettier::{Document, PrettyPrint, display},
	sys_events::SystemEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemEventNode {
	PushU64Div,
	PushExt2Intt,
	PushSmtPeek,
	PushMapVal,
	PushMapValN,
	PushMtNode,
	InsertMem,
	InsertHdword,
	InsertHdwordWithDomain,
	InsertHperm,
	PushSignature { kind: SignatureKind },
}

impl Display for SystemEventNode {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::PushU64Div => f.write_str("push_u64div"),
			Self::PushExt2Intt => f.write_str("push_ext2intt"),
			Self::PushSmtPeek => f.write_str("push_smtpeek"),
			Self::PushMapVal => f.write_str("push_mapval"),
			Self::PushMapValN => f.write_str("push_mapvaln"),
			Self::PushMtNode => f.write_str("push_mtnode"),
			Self::InsertMem => f.write_str("insert_mem"),
			Self::InsertHdword => f.write_str("insert_hdword"),
			Self::InsertHdwordWithDomain => f.write_str("insert_hdword_d"),
			Self::InsertHperm => f.write_str("insert_hperm"),
			Self::PushSignature { kind } => {
				f.write_str("push_sig.")?;
				Display::fmt(&kind, f)
			}
		}
	}
}

impl From<&SystemEventNode> for SystemEvent {
	fn from(value: &SystemEventNode) -> Self {
		match value {
			SystemEventNode::PushU64Div => Self::U64Div,
			SystemEventNode::PushExt2Intt => Self::Ext2Intt,
			SystemEventNode::PushSmtPeek => Self::SmtPeek,
			SystemEventNode::PushMapVal => Self::MapValueToStack,
			SystemEventNode::PushMapValN => Self::MapValueToStackN,
			SystemEventNode::PushMtNode => Self::MerkleNodeToStack,
			SystemEventNode::InsertMem => Self::MemToMap,
			SystemEventNode::InsertHdword => Self::HdwordToMap,
			SystemEventNode::InsertHdwordWithDomain => Self::HdwordToMapWithDomain,
			SystemEventNode::InsertHperm => Self::HpermToMap,
			SystemEventNode::PushSignature { kind } => match kind {
				SignatureKind::RpoFalcon512 => Self::FalconSigToStack,
			},
		}
	}
}

impl PrettyPrint for SystemEventNode {
	fn render(&self) -> Document {
		display(self)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignatureKind {
	RpoFalcon512 = 0,
}

impl From<SignatureKind> for vmm_core::SignatureKind {
	fn from(value: SignatureKind) -> Self {
		match value {
			SignatureKind::RpoFalcon512 => Self::RpoFalcon512,
		}
	}
}

impl Display for SignatureKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let kind: vmm_core::SignatureKind = (*self).into();
		Display::fmt(&kind, f)
	}
}
