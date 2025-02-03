use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use crate::{
	Felt, OPCODE_CALL, OPCODE_SYSCALL,
	chiplets::hasher,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest, MastForestError, MastNodeId},
	prettier::{Document, PrettyPrint, const_text, hex::ToHex, nl, text},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallNode {
	callee: MastNodeId,
	is_syscall: bool,
	digest: RpoDigest,
	before_enter: Vec<DecoratorId>,
	after_exit: Vec<DecoratorId>,
}

impl CallNode {
	pub const CALL_DOMAIN: Felt = Felt::new(OPCODE_CALL as u64);
	pub const SYSCALL_DOMAIN: Felt = Felt::new(OPCODE_SYSCALL as u64);

	pub fn new(callee: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		if callee.as_usize() >= mast_forest.nodes.len() {
			return Err(MastForestError::NodeIdOverflow(
				callee,
				mast_forest.nodes.len(),
			));
		}

		let digest = {
			let callee_digest = mast_forest[callee].digest();

			hasher::merge_in_domain(&[callee_digest, RpoDigest::default()], Self::CALL_DOMAIN)
		};

		Ok(Self::new_unchecked(callee, digest))
	}

	#[must_use]
	pub const fn new_unchecked(callee: MastNodeId, digest: RpoDigest) -> Self {
		Self {
			callee,
			is_syscall: false,
			digest,
			before_enter: Vec::new(),
			after_exit: Vec::new(),
		}
	}

	pub fn syscall(callee: MastNodeId, mast_forest: &MastForest) -> Result<Self, MastForestError> {
		if callee.as_usize() >= mast_forest.nodes.len() {
			return Err(MastForestError::NodeIdOverflow(
				callee,
				mast_forest.nodes.len(),
			));
		}

		let digest = {
			let callee_digest = mast_forest[callee].digest();

			hasher::merge_in_domain(&[callee_digest, RpoDigest::default()], Self::SYSCALL_DOMAIN)
		};

		Ok(Self::syscall_unchecked(callee, digest))
	}

	#[must_use]
	pub const fn syscall_unchecked(callee: MastNodeId, digest: RpoDigest) -> Self {
		Self {
			callee,
			is_syscall: true,
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
	pub const fn callee(&self) -> MastNodeId {
		self.callee
	}

	#[must_use]
	pub const fn is_syscall(&self) -> bool {
		self.is_syscall
	}

	#[must_use]
	pub const fn domain(&self) -> Felt {
		if self.is_syscall() {
			Self::SYSCALL_DOMAIN
		} else {
			Self::CALL_DOMAIN
		}
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
		CallNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		CallNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct CallNodePrettyPrint<'a> {
	node: &'a CallNode,
	mast_forest: &'a MastForest,
}

impl CallNodePrettyPrint<'_> {
	fn concatenate_decorators(
		&self,
		decorator_ids: &[DecoratorId],
		prepend: Document,
		append: Document,
	) -> Document {
		let decorators = decorator_ids
			.iter()
			.map(|&decorator_id| self.mast_forest[decorator_id].render())
			.reduce(|acc, doc| acc + const_text(" ") + doc)
			.unwrap_or_default();

		if decorators.is_empty() {
			decorators
		} else {
			prepend + decorators + append
		}
	}

	fn single_line_pre_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.before_enter(), Document::Empty, const_text(" "))
	}

	fn single_line_post_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.after_exit(), const_text(" "), Document::Empty)
	}

	fn multi_line_pre_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.before_enter(), Document::Empty, nl())
	}

	fn multi_line_post_decorators(&self) -> Document {
		self.concatenate_decorators(self.node.after_exit(), nl(), Document::Empty)
	}
}

impl Display for CallNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for CallNodePrettyPrint<'_> {
	fn render(&self) -> Document {
		let call_or_syscall =
			{
				let callee_digest = self.mast_forest[self.node.callee].digest();

				if self.node.is_syscall() {
					const_text("syscall")
						+ const_text(".") + text(callee_digest.as_bytes().to_hex_with_prefix())
				} else {
					const_text("call")
						+ const_text(".") + text(callee_digest.as_bytes().to_hex_with_prefix())
				}
			};

		let single_line = self.single_line_pre_decorators()
			+ call_or_syscall.clone()
			+ self.single_line_post_decorators();

		let multi_line =
			self.multi_line_pre_decorators() + call_or_syscall + self.multi_line_post_decorators();

		single_line | multi_line
	}
}
