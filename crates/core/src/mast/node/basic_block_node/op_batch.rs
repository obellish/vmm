use alloc::vec::Vec;

use super::{BATCH_SIZE, GROUP_SIZE};
use crate::{Felt, Operation, ZERO};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpBatch {
	pub(super) ops: Vec<Operation>,
	pub(super) groups: [Felt; BATCH_SIZE],
	pub(super) op_counts: [usize; BATCH_SIZE],
	pub(super) num_groups: usize,
}

impl OpBatch {
	#[must_use]
	pub fn ops(&self) -> &[Operation] {
		&self.ops
	}

	#[must_use]
	pub const fn groups(&self) -> &[Felt; BATCH_SIZE] {
		&self.groups
	}

	#[must_use]
	pub const fn op_counts(&self) -> &[usize; BATCH_SIZE] {
		&self.op_counts
	}

	#[must_use]
	pub const fn num_groups(&self) -> usize {
		self.num_groups
	}
}

pub(super) struct OpBatchAccumulator {
	ops: Vec<Operation>,
	groups: [Felt; BATCH_SIZE],
	op_counts: [usize; BATCH_SIZE],
	group: u64,
	op_idx: usize,
	group_idx: usize,
	next_group_idx: usize,
}

impl OpBatchAccumulator {
	pub const fn new() -> Self {
		Self {
			ops: Vec::new(),
			groups: [ZERO; BATCH_SIZE],
			op_counts: [0; BATCH_SIZE],
			group: 0,
			op_idx: 0,
			group_idx: 0,
			next_group_idx: 1,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.ops.is_empty()
	}

	pub fn can_accept_op(&self, op: Operation) -> bool {
		if op.imm_value().is_some() {
			if self.op_idx < GROUP_SIZE - 1 {
				self.next_group_idx < BATCH_SIZE
			} else {
				self.next_group_idx + 1 < BATCH_SIZE
			}
		} else {
			self.op_idx < GROUP_SIZE || self.next_group_idx < BATCH_SIZE
		}
	}

	pub fn add_op(&mut self, op: Operation) {
		if self.op_idx == GROUP_SIZE {
			self.finalize_op_group();
		}

		if let Some(imm) = op.imm_value() {
			if self.op_idx == GROUP_SIZE - 1 {
				self.finalize_op_group();
			}

			self.groups[self.next_group_idx] = imm;
			self.next_group_idx += 1;
		}

		let opcode = u64::from(op.op_code());
		self.group |= opcode << (Operation::OP_BITS * self.op_idx);
		self.ops.push(op);
		self.op_idx += 1;
	}

	pub fn into_batch(mut self) -> OpBatch {
		if !matches!(self.group, 0) || !matches!(self.op_idx, 0) {
			self.groups[self.group_idx] = Felt::new(self.group);
			self.op_counts[self.group_idx] = self.op_idx;
		}

		OpBatch {
			ops: self.ops,
			groups: self.groups,
			op_counts: self.op_counts,
			num_groups: self.next_group_idx,
		}
	}

	pub(super) fn finalize_op_group(&mut self) {
		self.groups[self.group_idx] = Felt::new(self.group);
		self.op_counts[self.group_idx] = self.op_idx;

		self.group_idx = self.next_group_idx;
		self.next_group_idx += 1;

		self.op_idx = 0;
		self.group = 0;
	}
}
