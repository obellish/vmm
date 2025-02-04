mod op_batch;

use alloc::vec::Vec;
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	mem,
};

pub use self::op_batch::OpBatch;
use self::op_batch::OpBatchAccumulator;
use crate::{
	DecoratorIterator, DecoratorList, DecoratorSlice, Felt, Operation, ZERO,
	chiplets::hasher,
	crypto::hash::RpoDigest,
	mast::{DecoratorId, MastForest, MastForestError},
	prettier::{Document, PrettyPrint, const_text, indent, nl},
};

pub const GROUP_SIZE: usize = 9;
pub const BATCH_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlockNode {
	op_batches: Vec<OpBatch>,
	digest: RpoDigest,
	decorators: DecoratorList,
}

impl BasicBlockNode {
	pub const DOMAIN: Felt = ZERO;

	pub fn new(
		operations: Vec<Operation>,
		decorators: Option<DecoratorList>,
	) -> Result<Self, MastForestError> {
		if operations.is_empty() {
			return Err(MastForestError::EmptyBasicBlock);
		}

		let decorators = decorators.unwrap_or_default();

		#[cfg(debug_assertions)]
		validate_decorators(&operations, &decorators);

		let (op_batches, digest) = batch_and_hash_ops(operations);
		Ok(Self {
			op_batches,
			digest,
			decorators,
		})
	}

	#[must_use]
	pub fn new_unchecked(
		operations: Vec<Operation>,
		decorators: DecoratorList,
		digest: RpoDigest,
	) -> Self {
		assert!(!operations.is_empty());
		let op_batches = batch_ops(operations);
		Self {
			op_batches,
			digest,
			decorators,
		}
	}

	#[cfg(test)]
	pub fn with_raw_decorators(
		operations: impl IntoIterator<Item = Operation>,
		decorators: impl IntoIterator<Item = (usize, crate::Decorator)>,
		mast_forest: &mut MastForest,
	) -> Result<Self, MastForestError> {
		let mut decorator_list = Vec::new();
		for (idx, decorator) in decorators {
			decorator_list.push((idx, mast_forest.add_decorator(decorator)?));
		}

		Self::new(operations.into_iter().collect(), Some(decorator_list))
	}

	#[must_use]
	pub const fn digest(&self) -> RpoDigest {
		self.digest
	}

	#[must_use]
	pub fn op_batches(&self) -> &[OpBatch] {
		&self.op_batches
	}

	#[must_use]
	pub fn num_op_batches(&self) -> usize {
		self.op_batches().len()
	}

	#[must_use]
	pub fn num_op_groups(&self) -> usize {
		let last_batch_num_groups = self
			.op_batches()
			.last()
			.expect("no last group")
			.num_groups();
		(self.num_op_batches() - 1) * BATCH_SIZE + last_batch_num_groups.next_power_of_two()
	}

	#[must_use]
	pub fn num_operations(&self) -> u32 {
		let num_ops = self
			.op_batches()
			.iter()
			.map(|batch| batch.ops().len())
			.sum::<usize>();
		num_ops
			.try_into()
			.expect("basic block contains more than 2^32 operations")
	}

	#[must_use]
	pub fn decorators(&self) -> &DecoratorSlice {
		&self.decorators
	}

	#[must_use]
	pub fn decorator_iter(&self) -> DecoratorIterator<'_> {
		DecoratorIterator::new(self.decorators())
	}

	pub fn operations(&self) -> impl Iterator<Item = &Operation> {
		self.op_batches().iter().flat_map(OpBatch::ops)
	}

	#[must_use]
	pub fn num_operations_and_decorators(&self) -> u32 {
		let num_ops = self.num_operations() as usize;
		let num_decorators = self.decorators().len();

		(num_ops + num_decorators)
			.try_into()
			.expect("basic block contains more than 2^32 operations and decorators")
	}

	pub fn iter(&self) -> impl Iterator<Item = OperationOrDecorator<'_>> {
		OperationOrDecoratorIterator::new(self)
	}

	pub fn prepend_decorators(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		let mut new_decorators = decorator_ids
			.into_iter()
			.map(|decorator_id| (0, decorator_id))
			.collect::<DecoratorList>();
		new_decorators.extend(mem::take(&mut self.decorators));

		self.set_decorators(new_decorators);
	}

	pub fn append_decorators(&mut self, decorator_ids: impl IntoIterator<Item = DecoratorId>) {
		let after_last_op_idx = self.num_operations() as usize;

		self.decorators.extend(
			decorator_ids
				.into_iter()
				.map(|decorator_id| (after_last_op_idx, decorator_id)),
		);
	}

	pub fn set_decorators(&mut self, decorators: impl IntoIterator<Item = (usize, DecoratorId)>) {
		self.decorators = decorators.into_iter().collect();
	}

	pub(super) fn to_display<'a>(&'a self, mast_forest: &'a MastForest) -> impl Display + 'a {
		BasicBlockNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}

	pub(super) fn to_pretty_print<'a>(
		&'a self,
		mast_forest: &'a MastForest,
	) -> impl PrettyPrint + 'a {
		BasicBlockNodePrettyPrint {
			node: self,
			mast_forest,
		}
	}
}

struct BasicBlockNodePrettyPrint<'a> {
	node: &'a BasicBlockNode,
	mast_forest: &'a MastForest,
}

impl Display for BasicBlockNodePrettyPrint<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for BasicBlockNodePrettyPrint<'_> {
	fn render(&self) -> Document {
		let single_line = const_text("basic_block")
			+ const_text(" ")
			+ self
				.node
				.iter()
				.map(|op_or_dec| match op_or_dec {
					OperationOrDecorator::Operation(op) => op.render(),
					OperationOrDecorator::Decorator(&decorator_id) => {
						self.mast_forest[decorator_id].render()
					}
				})
				.reduce(|acc, doc| acc + const_text(" ") + doc)
				.unwrap_or_default()
			+ const_text(" ")
			+ const_text("end");

		let multi_line = indent(
			4,
			const_text("basic_block")
				+ nl() + self
				.node
				.iter()
				.map(|op_or_dec| match op_or_dec {
					OperationOrDecorator::Operation(op) => op.render(),
					OperationOrDecorator::Decorator(&decorator_id) => {
						self.mast_forest[decorator_id].render()
					}
				})
				.reduce(|acc, doc| acc + nl() + doc)
				.unwrap_or_default(),
		) + nl() + const_text("end");

		single_line | multi_line
	}
}

struct OperationOrDecoratorIterator<'a> {
	node: &'a BasicBlockNode,
	batch_index: usize,
	op_index_in_batch: usize,
	op_index: usize,
	decorator_list_next_index: usize,
}

impl<'a> OperationOrDecoratorIterator<'a> {
	const fn new(node: &'a BasicBlockNode) -> Self {
		Self {
			node,
			batch_index: 0,
			op_index_in_batch: 0,
			op_index: 0,
			decorator_list_next_index: 0,
		}
	}
}

impl<'a> Iterator for OperationOrDecoratorIterator<'a> {
	type Item = OperationOrDecorator<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((op_index, decorator)) =
			self.node.decorators().get(self.decorator_list_next_index)
		{
			if *op_index == self.op_index {
				self.decorator_list_next_index += 1;
				return Some(OperationOrDecorator::Decorator(decorator));
			}
		}

		if let Some(batch) = self.node.op_batches().get(self.batch_index) {
			if let Some(operation) = batch.ops.get(self.op_index_in_batch) {
				self.op_index_in_batch += 1;
				self.op_index += 1;

				Some(OperationOrDecorator::Operation(operation))
			} else {
				self.batch_index += 1;
				self.op_index_in_batch = 0;

				self.next()
			}
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationOrDecorator<'a> {
	Operation(&'a Operation),
	Decorator(&'a DecoratorId),
}

fn batch_and_hash_ops(ops: impl IntoIterator<Item = Operation>) -> (Vec<OpBatch>, RpoDigest) {
	let batches = batch_ops(ops);

	let op_groups = batches
		.iter()
		.flat_map(|batch| batch.groups)
		.collect::<Vec<_>>();
	let hash = hasher::hash_elements(&op_groups);

	(batches, hash)
}

fn batch_ops(ops: impl IntoIterator<Item = Operation>) -> Vec<OpBatch> {
	let mut batches = Vec::new();
	let mut batch_acc = OpBatchAccumulator::new();

	for op in ops {
		if !batch_acc.can_accept_op(op) {
			let batch = batch_acc.into_batch();
			batch_acc = OpBatchAccumulator::new();

			batches.push(batch);
		}

		batch_acc.add_op(op);
	}

	if !batch_acc.is_empty() {
		let batch = batch_acc.into_batch();
		batches.push(batch);
	}

	batches
}

#[cfg(debug_assertions)]
fn validate_decorators(operations: &[Operation], decorators: &DecoratorSlice) {
	if !decorators.is_empty() {
		for i in 0..(decorators.len() - 1) {
			debug_assert!(
				decorators[i + 1].0 >= decorators[i].0,
				"unsorted decorators list"
			);
		}

		debug_assert!(
			operations.len() >= decorators.last().expect("empty decorators list").0,
			"last op index in decorator list should be less than or equal to the number of ops"
		);
	}
}

#[cfg(test)]
mod tests {
	use super::{BasicBlockNode, batch_and_hash_ops};
	use crate::{
		Decorator, Felt, ONE, Operation, ZERO,
		chiplets::hasher,
		mast::{DecoratorId, MastForest, MastForestError, OP_BATCH_SIZE, OperationOrDecorator},
	};

	fn build_group(ops: &[Operation]) -> Felt {
		let mut group = 0u64;
		for (i, op) in ops.iter().enumerate() {
			group |= u64::from(op.op_code()) << (Operation::OP_BITS * i);
		}
		Felt::new(group)
	}

	#[test]
	fn batch_ops_one_operation() {
		let ops = [Operation::Add];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 1);

		let mut batch_groups = [ZERO; OP_BATCH_SIZE];
		batch_groups[0] = build_group(&ops);

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [1, 0, 0, 0, 0, 0, 0, 0]);
		assert_eq!(hasher::hash_elements(&batch_groups), hash);
	}

	#[test]
	fn batch_ops_two_operations() {
		let ops = [Operation::Add, Operation::Mul];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 1);

		let mut batch_groups = [ZERO; OP_BATCH_SIZE];
		batch_groups[0] = build_group(&ops);

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [2, 0, 0, 0, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_one_group_one_immediate() {
		let ops = [Operation::Add, Operation::Push(Felt::new(12_345_678))];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 2);

		let mut batch_groups = [ZERO; OP_BATCH_SIZE];
		batch_groups[0] = build_group(&ops);
		batch_groups[1] = Felt::new(12_345_678);

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [2, 0, 0, 0, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_one_group_7_immediates() {
		let ops = [
			Operation::Push(ONE),
			Operation::Push(Felt::new(2)),
			Operation::Push(Felt::new(3)),
			Operation::Push(Felt::new(4)),
			Operation::Push(Felt::new(5)),
			Operation::Push(Felt::new(6)),
			Operation::Push(Felt::new(7)),
			Operation::Add,
		];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 8);

		let batch_groups = [
			build_group(&ops),
			ONE,
			Felt::new(2),
			Felt::new(3),
			Felt::new(4),
			Felt::new(5),
			Felt::new(6),
			Felt::new(7),
		];

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [8, 0, 0, 0, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_two_groups_7_immediates() {
		let ops = [
			Operation::Add,
			Operation::Mul,
			Operation::Push(ONE),
			Operation::Push(Felt::new(2)),
			Operation::Push(Felt::new(3)),
			Operation::Push(Felt::new(4)),
			Operation::Push(Felt::new(5)),
			Operation::Push(Felt::new(6)),
			Operation::Add,
			Operation::Push(Felt::new(7)),
		];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 2);

		let batch0 = &batches[0];
		assert_eq!(batch0.ops(), &ops[..9]);
		assert_eq!(batch0.num_groups(), 7);

		let batch0_groups = [
			build_group(&ops[..9]),
			ONE,
			Felt::new(2),
			Felt::new(3),
			Felt::new(4),
			Felt::new(5),
			Felt::new(6),
			ZERO,
		];

		assert_eq!(*batch0.groups(), batch0_groups);
		assert_eq!(*batch0.op_counts(), [9, 0, 0, 0, 0, 0, 0, 0]);

		let batch1 = &batches[1];
		assert_eq!(batch1.ops(), [ops[9]]);
		assert_eq!(batch1.num_groups(), 2);

		let mut batch1_groups = [ZERO; OP_BATCH_SIZE];
		batch1_groups[0] = build_group(&[ops[9]]);
		batch1_groups[1] = Felt::new(7);

		assert_eq!(*batch1.groups(), batch1_groups);
		assert_eq!(*batch1.op_counts(), [1, 0, 0, 0, 0, 0, 0, 0]);

		let all_groups = [batch0_groups, batch1_groups].concat();
		assert_eq!(hash, hasher::hash_elements(&all_groups));
	}

	#[test]
	fn batch_ops_immediate_values_between_groups() {
		let ops = [
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Push(Felt::new(7)),
			Operation::Add,
			Operation::Add,
			Operation::Push(Felt::new(11)),
			Operation::Mul,
			Operation::Mul,
			Operation::Add,
		];

		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 4);

		let batch_groups = [
			build_group(&ops[..9]),
			Felt::new(7),
			Felt::new(11),
			build_group(&ops[9..]),
			ZERO,
			ZERO,
			ZERO,
			ZERO,
		];

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [9, 0, 0, 1, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_push_moved_to_next_group_1() {
		let ops = [
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Add,
			Operation::Add,
			Operation::Mul,
			Operation::Mul,
			Operation::Add,
			Operation::Push(Felt::new(11)),
		];

		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 3);

		let batch_groups = [
			build_group(&ops[..8]),
			build_group(&[ops[8]]),
			Felt::new(11),
			ZERO,
			ZERO,
			ZERO,
			ZERO,
			ZERO,
		];

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [8, 1, 0, 0, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_push_moved_to_next_group_2() {
		let ops = [
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Add,
			Operation::Add,
			Operation::Mul,
			Operation::Mul,
			Operation::Push(ONE),
			Operation::Push(Felt::new(2)),
		];
		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 1);

		let batch = &batches[0];
		assert_eq!(batch.ops(), ops);
		assert_eq!(batch.num_groups(), 4);

		let batch_groups = [
			build_group(&ops[..8]),
			ONE,
			build_group(&[ops[8]]),
			Felt::new(2),
			ZERO,
			ZERO,
			ZERO,
			ZERO,
		];

		assert_eq!(*batch.groups(), batch_groups);
		assert_eq!(*batch.op_counts(), [8, 0, 1, 0, 0, 0, 0, 0]);
		assert_eq!(hash, hasher::hash_elements(&batch_groups));
	}

	#[test]
	fn batch_ops_push_7th_group_overflows() {
		let ops = [
			Operation::Add,
			Operation::Mul,
			Operation::Push(ONE),
			Operation::Push(Felt::new(2)),
			Operation::Push(Felt::new(3)),
			Operation::Push(Felt::new(4)),
			Operation::Push(Felt::new(5)),
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Mul,
			Operation::Add,
			Operation::Mul,
			Operation::Push(Felt::new(6)),
			Operation::Pad,
		];

		let (batches, hash) = batch_and_hash_ops(ops);
		assert_eq!(batches.len(), 2);

		let batch0 = &batches[0];
		assert_eq!(batch0.ops(), &ops[..17]);
		assert_eq!(batch0.num_groups(), 7);

		let batch0_groups = [
			build_group(&ops[..9]),
			ONE,
			Felt::new(2),
			Felt::new(3),
			Felt::new(4),
			Felt::new(5),
			build_group(&ops[9..17]),
			ZERO,
		];

		assert_eq!(*batch0.groups(), batch0_groups);
		assert_eq!(*batch0.op_counts(), [9, 0, 0, 0, 0, 0, 8, 0]);

		let batch1 = &batches[1];
		assert_eq!(batch1.ops(), &ops[17..]);
		assert_eq!(batch1.num_groups(), 2);

		let batch1_groups = [
			build_group(&ops[17..]),
			Felt::new(6),
			ZERO,
			ZERO,
			ZERO,
			ZERO,
			ZERO,
			ZERO,
		];

		assert_eq!(*batch1.groups(), batch1_groups);
		assert_eq!(*batch1.op_counts(), [2, 0, 0, 0, 0, 0, 0, 0]);

		let all_groups = [batch0_groups, batch1_groups].concat();
		assert_eq!(hash, hasher::hash_elements(&all_groups));
	}

	#[test]
	fn operation_or_decorator_iterator() -> Result<(), MastForestError> {
		let mut mast_forest = MastForest::new();
		let operations = [
			Operation::Add,
			Operation::Mul,
			Operation::MovDn2,
			Operation::MovDn3,
		];

		let decorators = [
			(0, Decorator::Trace(0)),
			(0, Decorator::Trace(1)),
			(3, Decorator::Trace(2)),
			(4, Decorator::Trace(3)),
			(4, Decorator::Trace(4)),
		];

		let node = BasicBlockNode::with_raw_decorators(operations, decorators, &mut mast_forest)?;

		let mut iterator = node.iter();

		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Decorator(&DecoratorId(0)))
		);
		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Decorator(&DecoratorId(1)))
		);
		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Operation(&Operation::Add))
		);

		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Operation(&Operation::Mul))
		);
		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Operation(&Operation::MovDn2))
		);

		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Decorator(&DecoratorId(2)))
		);
		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Operation(&Operation::MovDn3))
		);

		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Decorator(&DecoratorId(3)))
		);
		assert_eq!(
			iterator.next(),
			Some(OperationOrDecorator::Decorator(&DecoratorId(4)))
		);
		assert_eq!(iterator.next(), None);

		Ok(())
	}
}
