use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	Felt, OPCODE_JOIN,
	chiplets::hasher,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest, MastForestError, MastNodeId},
	prettier::{Document, PrettyPrint, const_text, indent, nl},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinNode {
	children: [MastNodeId; 2],
	digest: RpoDigest,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl JoinNode {
	pub const DOMAIN: Felt = Felt::new(OPCODE_JOIN as u64);

	pub fn new(
		children: [MastNodeId; 2],
		mast_forest: &MastForest,
	) -> Result<Self, MastForestError> {
		let forest_len = mast_forest.nodes.len();
		if children[0].as_usize() >= forest_len {
			return Err(MastForestError::NodeIdOverflow(children[0], forest_len));
		} else if children[1].as_usize() >= forest_len {
			return Err(MastForestError::NodeIdOverflow(children[1], forest_len));
		}

		let digest = {
			let left_child_hash = mast_forest[children[0]].digest();
			let right_child_hash = mast_forest[children[1]].digest();

			hasher::merge_in_domain(&[left_child_hash, right_child_hash], Self::DOMAIN)
		};

		Ok(Self::new_unchecked(children, digest))
	}

	#[must_use]
	pub const fn new_unchecked(children: [MastNodeId; 2], digest: RpoDigest) -> Self {
		Self {
			children,
			digest,
			before_enter: Vec::new(),
			after_exit: Vec::new(),
		}
	}

	#[must_use]
	pub const fn digest(&self) -> RpoDigest {
		self.digest
	}

	#[must_use]
	pub const fn first(&self) -> MastNodeId {
		self.children[0]
	}

	#[must_use]
	pub const fn second(&self) -> MastNodeId {
		self.children[1]
	}

	#[must_use]
	pub fn before_enter(&self) -> &[DecoratorId] {
		&self.before_enter
	}

	#[must_use]
	pub fn after_exit(&self) -> &[DecoratorId] {
		&self.after_exit
	}

	pub fn set_before_enter(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		self.before_enter = decorator_ids.into_iter().collect();
	}

	pub fn set_after_exit(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		self.after_exit = decorator_ids.into_iter().collect();
	}

	pub(super) fn to_display<'a>(&'a self, mast_forest: &'a MastForest) -> impl Display + 'a {
		JoinNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		JoinNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct JoinNodePrettyPrint<'a> {
	node: &'a JoinNode,
	mast_forest: &'a MastForest,
}

impl Display for JoinNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for JoinNodePrettyPrint<'_> {
	fn render(&self) -> Document {
		let pre_decorators = {
			let mut pre_decorators = self
				.node
				.before_enter()
				.iter()
				.map(|&decorator_id| self.mast_forest[decorator_id].render())
				.reduce(|acc, doc| acc + const_text(" ") + doc)
				.unwrap_or_default();

			if !pre_decorators.is_empty() {
				pre_decorators += nl();
			}

			pre_decorators
		};

		let post_decorators = {
			let mut post_decorators = self
				.node
				.after_exit()
				.iter()
				.map(|&decorator_id| self.mast_forest[decorator_id].render())
				.reduce(|acc, doc| acc + const_text(" ") + doc)
				.unwrap_or_default();

			if !post_decorators.is_empty() {
				post_decorators += nl();
			}

			post_decorators
		};

		let first_child = self.mast_forest[self.node.first()].to_pretty_print(self.mast_forest);
		let second_child = self.mast_forest[self.node.second()].to_pretty_print(self.mast_forest);

		pre_decorators
			+ indent(
				4,
				const_text("join") + nl() + first_child.render() + nl() + second_child.render(),
			) + nl() + const_text("end")
			+ post_decorators
	}
}
