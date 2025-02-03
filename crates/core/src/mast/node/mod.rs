mod basic_block_node;
mod call_node;
mod dyn_node;
mod external_node;
mod join_node;
mod loop_node;
mod split_node;

use alloc::{boxed::Box, vec::Vec};
use core::fmt::{Display, Formatter, Result as FmtResult};

pub use self::{
	basic_block_node::{
		BATCH_SIZE as OP_BATCH_SIZE, BasicBlockNode, GROUP_SIZE as OP_GROUP_SIZE, OpBatch,
		OperationOrDecorator,
	},
	call_node::CallNode,
	dyn_node::DynNode,
	external_node::ExternalNode,
	join_node::JoinNode,
	loop_node::LoopNode,
	split_node::SplitNode,
};
use super::{DecoratorId, MastForest, MastForestError, MastNodeId};
use crate::{DecoratorList, Felt, Operation, crypto::hash::RpoDigest, prettier::PrettyPrint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MastNode {
	Block(BasicBlockNode),
	Join(JoinNode),
	Split(SplitNode),
	Loop(LoopNode),
	Dyn(DynNode),
	Call(CallNode),
	External(ExternalNode),
}

impl MastNode {
	pub fn basic_block(
		operations: Vec<Operation>,
		decorators: Option<DecoratorList>,
	) -> Result<Self, MastForestError> {
		BasicBlockNode::new(operations, decorators).map(Self::Block)
	}

	pub fn join(
		left_child: MastNodeId,
		right_child: MastNodeId,
		mast_forest: &MastForest,
	) -> Result<Self, MastForestError> {
		JoinNode::new([left_child, right_child], mast_forest).map(Self::Join)
	}

	pub fn split(
		if_branch: MastNodeId,
		else_branch: MastNodeId,
		mast_forest: &MastForest,
	) -> Result<Self, MastForestError> {
		SplitNode::new([if_branch, else_branch], mast_forest).map(Self::Split)
	}

	pub fn r#loop(body: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		LoopNode::new(body, mast_forest).map(Self::Loop)
	}

	pub fn call(callee: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		CallNode::new(callee, mast_forest).map(Self::Call)
	}

	pub fn syscall(callee: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		CallNode::syscall(callee, mast_forest).map(Self::Call)
	}

	#[must_use]
	pub const fn r#dyn() -> Self {
		Self::Dyn(DynNode::r#dyn())
	}

	#[must_use]
	pub const fn dyncall() -> Self {
		Self::Dyn(DynNode::dyncall())
	}

	#[must_use]
	pub const fn external(mast_root: RpoDigest) -> Self {
		Self::External(ExternalNode::new(mast_root))
	}

	#[cfg(test)]
	pub fn basic_block_with_raw_decorators(
		operations: impl IntoIterator<Item = Operation>,
		decorators: impl IntoIterator<Item = (usize, crate::Decorator)>,
		mast_forest: &mut MastForest,
	) -> Result<Self, MastForestError> {
		BasicBlockNode::with_raw_decorators(operations, decorators, mast_forest).map(Self::Block)
	}

	#[must_use]
	pub const fn is_external(&self) -> bool {
		matches!(self, Self::External(_))
	}

	#[must_use]
	pub const fn is_dyn(&self) -> bool {
		matches!(self, Self::Dyn(_))
	}

	#[must_use]
	pub const fn is_basic_block(&self) -> bool {
		matches!(self, Self::Block(_))
	}

	#[must_use]
	pub const fn get_basic_block(&self) -> Option<&BasicBlockNode> {
		let Self::Block(basic_block_node) = self else {
			return None;
		};

		Some(basic_block_node)
	}

	#[must_use]
	pub fn to_pretty_print<'a>(&'a self, mast_forest: &'a MastForest) -> impl PrettyPrint + 'a {
		match self {
			Self::Block(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::Join(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::Split(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::Loop(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::Dyn(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::Call(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
			Self::External(node) => MastNodePrettyPrint::new(node.to_pretty_print(mast_forest)),
		}
	}

	#[must_use]
	pub const fn domain(&self) -> Felt {
		match self {
			Self::Block(_) => BasicBlockNode::DOMAIN,
			Self::Join(_) => JoinNode::DOMAIN,
			Self::Split(_) => SplitNode::DOMAIN,
			Self::Loop(_) => LoopNode::DOMAIN,
			Self::Call(call_node) => call_node.domain(),
			Self::Dyn(dyn_node) => dyn_node.domain(),
			Self::External(_) => panic!("can't fetch domain for an external node"),
		}
	}

	#[must_use]
	pub const fn digest(&self) -> RpoDigest {
		match self {
			Self::Block(node) => node.digest(),
			Self::Join(node) => node.digest(),
			Self::Split(node) => node.digest(),
			Self::Loop(node) => node.digest(),
			Self::Dyn(node) => node.digest(),
			Self::Call(node) => node.digest(),
			Self::External(node) => node.digest(),
		}
	}

	#[must_use]
	pub fn to_display<'a>(&'a self, mast_forest: &'a MastForest) -> impl Display + 'a {
		match self {
			Self::Block(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::Join(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::Split(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::Loop(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::Call(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::Dyn(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
			Self::External(node) => MastNodeDisplay::new(node.to_display(mast_forest)),
		}
	}

	#[must_use]
	pub fn before_enter(&self) -> &[DecoratorId] {
		match self {
			Self::Block(_) => &[],
			Self::Join(node) => node.before_enter(),
			Self::Split(node) => node.before_enter(),
			Self::Loop(node) => node.before_enter(),
			Self::Call(node) => node.before_enter(),
			Self::Dyn(node) => node.before_enter(),
			Self::External(node) => node.before_enter(),
		}
	}

	#[must_use]
	pub fn after_exit(&self) -> &[DecoratorId] {
		match self {
			Self::Block(_) => &[],
			Self::Join(node) => node.after_exit(),
			Self::Split(node) => node.after_exit(),
			Self::Loop(node) => node.after_exit(),
			Self::Call(node) => node.after_exit(),
			Self::Dyn(node) => node.after_exit(),
			Self::External(node) => node.after_exit(),
		}
	}

	pub fn set_before_enter(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		match self {
			Self::Block(node) => node.prepend_decorators(decorator_ids),
			Self::Join(node) => node.set_before_enter(decorator_ids),
			Self::Split(node) => node.set_before_enter(decorator_ids),
			Self::Loop(node) => node.set_before_enter(decorator_ids),
			Self::Call(node) => node.set_before_enter(decorator_ids),
			Self::Dyn(node) => node.set_before_enter(decorator_ids),
			Self::External(node) => node.set_before_enter(decorator_ids),
		}
	}

	pub fn set_after_exit(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		match self {
			Self::Block(node) => node.append_decorators(decorator_ids),
			Self::Join(node) => node.set_after_exit(decorator_ids),
			Self::Split(node) => node.set_after_exit(decorator_ids),
			Self::Loop(node) => node.set_after_exit(decorator_ids),
			Self::Call(node) => node.set_after_exit(decorator_ids),
			Self::Dyn(node) => node.set_after_exit(decorator_ids),
			Self::External(node) => node.set_after_exit(decorator_ids),
		}
	}
}

#[repr(transparent)]
struct MastNodePrettyPrint<'a> {
	node: Box<dyn PrettyPrint + 'a>,
}

impl<'a> MastNodePrettyPrint<'a> {
	pub fn new(node_pretty_print: impl PrettyPrint + 'a) -> Self {
		Self {
			node: Box::new(node_pretty_print),
		}
	}
}

impl PrettyPrint for MastNodePrettyPrint<'_> {
	fn render(&self) -> vmm_formatting::prettier::Document {
		self.node.render()
	}
}

#[repr(transparent)]
struct MastNodeDisplay<'a> {
	node: Box<dyn Display + 'a>,
}

impl<'a> MastNodeDisplay<'a> {
	pub fn new(node: impl Display + 'a) -> Self {
		Self {
			node: Box::new(node),
		}
	}
}

impl Display for MastNodeDisplay<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.node.fmt(f)
	}
}
