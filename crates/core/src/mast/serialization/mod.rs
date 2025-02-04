mod basic_block;
mod decorator;
mod info;
mod string_table;

use alloc::vec::Vec;
use core::iter;

use self::{
	basic_block::{BasicBlockDataBuilder, BasicBlockDataDecoder},
	decorator::{DecoratorDataBuilder, DecoratorInfo},
	info::MastNodeInfo,
	string_table::StringTable,
};
use super::{DecoratorId, MastForest, MastNode};
use crate::{
	AdviceMap, DecoratorList,
	mast::MastNodeId,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

const MAGIC: &[u8; 5] = b"MAST\0";

const VERSION: [u8; 3] = [0; 3];

type NodeDataOffset = u32;
type DecoratorDataOffset = u32;
type StringDataOffset = usize;
type StringIndex = usize;

impl Deserializable for MastForest {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		read_and_validate_magic(source)?;
		read_and_validate_version(source)?;

		let node_count = source.read_usize()?;
		let decorator_count = source.read_usize()?;

		let roots = Vec::<u32>::read_from(source)?;

		let basic_block_data = Vec::<u8>::read_from(source)?;
		let mast_node_infos = node_infos_iter(source, node_count).collect::<Result<Vec<_>, _>>()?;

		let advice_map = AdviceMap::read_from(source)?;

		let decorator_data = Vec::<u8>::read_from(source)?;
		let string_table = StringTable::read_from(source)?;
		let decorator_infos = decorator_infos_iter(source, decorator_count);

		let mut mast_forest = {
			let mut mast_forest = Self::new();

			for decorator_info in decorator_infos {
				let decorator_info = decorator_info?;
				let decorator =
					decorator_info.try_into_decorator(&string_table, &decorator_data)?;

				mast_forest.add_decorator(decorator).map_err(|e| {
					DeserializationError::InvalidValue(format!(
						"failed to add decorator to MAST forest while deserializing: {e}"
					))
				})?;
			}

			let basic_block_data_decoder = BasicBlockDataDecoder::new(&basic_block_data);
			for mast_node_info in mast_node_infos {
				let node =
					mast_node_info.try_into_mast_node(node_count, &basic_block_data_decoder)?;

				mast_forest.add_node(node).map_err(|e| {
					DeserializationError::InvalidValue(format!(
						"failed to add node to MAST forest while deserializing: {e}"
					))
				})?;
			}

			for root in roots {
				let root = MastNodeId::from_u32(root, &mast_forest)?;
				mast_forest.make_root(root);
			}

			mast_forest.advice_map = advice_map;

			mast_forest
		};

		let basic_block_decorators = read_block_decorators(source, &mast_forest)?;
		for (node_id, decorator_list) in basic_block_decorators {
			let node_id = MastNodeId::from_usize(node_id, &mast_forest)?;

			match &mut mast_forest[node_id] {
				MastNode::Block(basic_block) => {
					basic_block.set_decorators(decorator_list);
				}
				other => {
					return Err(DeserializationError::InvalidValue(format!(
						"expected mast node with id {node_id} to be a basic block, found {other:?}"
					)));
				}
			}
		}

		let before_enter_decorators = read_before_after_decorators(source, &mast_forest)?;
		for (node_id, decorator_ids) in before_enter_decorators {
			let node_id = MastNodeId::from_usize(node_id, &mast_forest)?;
			mast_forest.set_before_enter(node_id, decorator_ids);
		}

		let after_exit_decorators = read_before_after_decorators(source, &mast_forest)?;
		for (node_id, decorator_ids) in after_exit_decorators {
			let node_id = MastNodeId::from_usize(node_id, &mast_forest)?;
			mast_forest.set_after_exit(node_id, decorator_ids);
		}

		Ok(mast_forest)
	}
}

impl Serializable for MastForest {
	#[allow(clippy::collection_is_never_read)]
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let mut basic_block_data_builder = BasicBlockDataBuilder::new();

		let mut before_enter_decorators = Vec::new();
		let mut after_exit_decorators = Vec::new();

		let mut basic_block_decorators = Vec::new();

		target.write_bytes(MAGIC);
		target.write_bytes(&VERSION);

		target.write_usize(self.nodes.len());
		target.write_usize(self.decorators.len());

		let roots = self
			.roots
			.iter()
			.copied()
			.map(u32::from)
			.collect::<Vec<_>>();
		roots.write_into(target);

		let mast_node_infos = self
			.nodes
			.iter()
			.enumerate()
			.map(|(mast_node_id, mast_node)| {
				if !mast_node.before_enter().is_empty() {
					before_enter_decorators.push((mast_node_id, mast_node.before_enter().to_vec()));
				}

				if !mast_node.after_exit().is_empty() {
					after_exit_decorators.push((mast_node_id, mast_node.after_exit().to_vec()));
				}

				let ops_offset = if let MastNode::Block(basic_block) = mast_node {
					let ops_offset = basic_block_data_builder.encode_last_block(basic_block);

					basic_block_decorators.push((mast_node_id, basic_block.decorators().to_vec()));

					ops_offset
				} else {
					0
				};

				MastNodeInfo::new(mast_node, ops_offset)
			})
			.collect::<Vec<_>>();

		let basic_block_data = basic_block_data_builder.finalize();
		basic_block_data.write_into(target);

		for mast_node_info in mast_node_infos {
			mast_node_info.write_into(target);
		}

		self.advice_map.write_into(target);

		let mut decorator_data_builder = DecoratorDataBuilder::new();
		for decorator in &self.decorators {
			decorator_data_builder.add_decorator(decorator);
		}

		let (decorator_data, decorator_infos, string_table) = decorator_data_builder.finalize();

		decorator_data.write_into(target);
		string_table.write_into(target);

		for decorator_info in decorator_infos {
			decorator_info.write_into(target);
		}

		basic_block_decorators.write_into(target);

		before_enter_decorators.write_into(target);
		after_exit_decorators.write_into(target);
	}
}

fn read_and_validate_magic<R: ByteReader>(source: &mut R) -> Result<[u8; 5], DeserializationError> {
	let magic: [u8; 5] = source.read_array()?;
	if magic == *MAGIC {
		Ok(magic)
	} else {
		Err(DeserializationError::InvalidValue(format!(
			"invalid magic bytes. expected '{:?}', got '{magic:?}'",
			*MAGIC
		)))
	}
}

fn read_and_validate_version<R: ByteReader>(
	source: &mut R,
) -> Result<[u8; 3], DeserializationError> {
	let version: [u8; 3] = source.read_array()?;
	if version == VERSION {
		Ok(version)
	} else {
		Err(DeserializationError::InvalidValue(format!(
			"unsupported version. got '{version:?}', but only '{VERSION:?}' is supported"
		)))
	}
}

fn read_block_decorators<R: ByteReader>(
	source: &mut R,
	mast_forest: &MastForest,
) -> Result<Vec<(usize, DecoratorList)>, DeserializationError> {
	let vec_len: usize = source.read()?;
	let mut out_vec = Vec::with_capacity(vec_len);

	for _ in 0..vec_len {
		let node_id: usize = source.read()?;

		let decorator_vec_len: usize = source.read()?;
		let mut inner_vec = Vec::with_capacity(decorator_vec_len);
		for _ in 0..decorator_vec_len {
			let op_id: usize = source.read()?;
			let decorator_id = DecoratorId::from_u32(source.read()?, mast_forest)?;
			inner_vec.push((op_id, decorator_id));
		}

		out_vec.push((node_id, inner_vec));
	}

	Ok(out_vec)
}

fn decorator_infos_iter<'a, R>(
	source: &'a mut R,
	decorator_count: usize,
) -> impl Iterator<Item = Result<DecoratorInfo, DeserializationError>> + 'a
where
	R: ByteReader + 'a,
{
	let mut remaining = decorator_count;
	iter::from_fn(move || {
		if matches!(remaining, 0) {
			None
		} else {
			remaining -= 1;
			Some(DecoratorInfo::read_from(source))
		}
	})
}

fn node_infos_iter<'a, R>(
	source: &'a mut R,
	node_count: usize,
) -> impl Iterator<Item = Result<MastNodeInfo, DeserializationError>> + 'a
where
	R: ByteReader + 'a,
{
	let mut remaining = node_count;
	iter::from_fn(move || {
		if matches!(remaining, 0) {
			None
		} else {
			remaining -= 1;
			Some(MastNodeInfo::read_from(source))
		}
	})
}

fn read_before_after_decorators<R: ByteReader>(
	source: &mut R,
	mast_forest: &MastForest,
) -> Result<Vec<(usize, Vec<DecoratorId>)>, DeserializationError> {
	let vec_len: usize = source.read()?;
	let mut out_vec = Vec::with_capacity(vec_len);

	for _ in 0..vec_len {
		let node_id: usize = source.read()?;

		let inner_vec_len: usize = source.read()?;
		let mut inner_vec = Vec::with_capacity(inner_vec_len);
		for _ in 0..inner_vec_len {
			let decorator_id = DecoratorId::from_u32(source.read()?, mast_forest)?;
			inner_vec.push(decorator_id);
		}

		out_vec.push((node_id, inner_vec));
	}

	Ok(out_vec)
}

#[cfg(test)]
mod tests {
	use alloc::{borrow::ToOwned as _, sync::Arc};

	use assert_matches::assert_matches;

	use crate::{
		AssemblyOp, DebugOptions, Decorator, Felt, ONE, Operation,
		crypto::hash::RpoDigest,
		mast::{MastForest, MastForestError},
		utils::{Deserializable, DeserializationError, Serializable},
	};

	#[test]
	#[allow(clippy::match_same_arms)]
	fn confirm_operation_and_decorator_structure() {
		match Operation::Noop {
			Operation::Noop => (),
			Operation::Assert(_) => (),
			Operation::FmpAdd => (),
			Operation::FmpUpdate => (),
			Operation::SDepth => (),
			Operation::Caller => (),
			Operation::Clk => (),
			Operation::Join => (),
			Operation::Split => (),
			Operation::Loop => (),
			Operation::Call => (),
			Operation::Dyn => (),
			Operation::DynCall => (),
			Operation::SysCall => (),
			Operation::Span => (),
			Operation::End => (),
			Operation::Repeat => (),
			Operation::Respan => (),
			Operation::Halt => (),
			Operation::Add => (),
			Operation::Neg => (),
			Operation::Mul => (),
			Operation::Inv => (),
			Operation::Incr => (),
			Operation::And => (),
			Operation::Or => (),
			Operation::Not => (),
			Operation::Eq => (),
			Operation::Eqz => (),
			Operation::Expacc => (),
			Operation::Ext2Mul => (),
			Operation::U32split => (),
			Operation::U32add => (),
			Operation::U32assert2(_) => (),
			Operation::U32add3 => (),
			Operation::U32sub => (),
			Operation::U32mul => (),
			Operation::U32madd => (),
			Operation::U32div => (),
			Operation::U32and => (),
			Operation::U32xor => (),
			Operation::Pad => (),
			Operation::Drop => (),
			Operation::Dup0 => (),
			Operation::Dup1 => (),
			Operation::Dup2 => (),
			Operation::Dup3 => (),
			Operation::Dup4 => (),
			Operation::Dup5 => (),
			Operation::Dup6 => (),
			Operation::Dup7 => (),
			Operation::Dup9 => (),
			Operation::Dup11 => (),
			Operation::Dup13 => (),
			Operation::Dup15 => (),
			Operation::Swap => (),
			Operation::SwapW => (),
			Operation::SwapW2 => (),
			Operation::SwapW3 => (),
			Operation::SwapDW => (),
			Operation::MovUp2 => (),
			Operation::MovUp3 => (),
			Operation::MovUp4 => (),
			Operation::MovUp5 => (),
			Operation::MovUp6 => (),
			Operation::MovUp7 => (),
			Operation::MovUp8 => (),
			Operation::MovDn2 => (),
			Operation::MovDn3 => (),
			Operation::MovDn4 => (),
			Operation::MovDn5 => (),
			Operation::MovDn6 => (),
			Operation::MovDn7 => (),
			Operation::MovDn8 => (),
			Operation::CSwap => (),
			Operation::CSwapW => (),
			Operation::Push(_) => (),
			Operation::AdvPop => (),
			Operation::AdvPopW => (),
			Operation::MLoadW => (),
			Operation::MStoreW => (),
			Operation::MLoad => (),
			Operation::MStore => (),
			Operation::MStream => (),
			Operation::Pipe => (),
			Operation::HPerm => (),
			Operation::MpVerify(_) => (),
			Operation::MrUpdate => (),
			Operation::FriE2F4 => (),
			Operation::RCombBase => (),
			Operation::Emit(_) => (),
		};

		match Decorator::Trace(0) {
			Decorator::AsmOp(_) => (),
			Decorator::Debug(debug_options) => match debug_options {
				DebugOptions::StackAll => (),
				DebugOptions::StackTop(_) => (),
				DebugOptions::MemAll => (),
				DebugOptions::MemInterval(..) => (),
				DebugOptions::LocalInterval(..) => (),
			},
			Decorator::Trace(_) => (),
		};
	}

	#[test]
	fn serialize_deserialize_all_nodes() -> eyre::Result<()> {
		let mut mast_forest = MastForest::new();

		let basic_block_id = {
			let operations = vec![
				Operation::Noop,
				Operation::Assert(42),
				Operation::FmpAdd,
				Operation::FmpUpdate,
				Operation::SDepth,
				Operation::Caller,
				Operation::Clk,
				Operation::Join,
				Operation::Split,
				Operation::Loop,
				Operation::Call,
				Operation::Dyn,
				Operation::SysCall,
				Operation::Span,
				Operation::End,
				Operation::Repeat,
				Operation::Respan,
				Operation::Halt,
				Operation::Add,
				Operation::Neg,
				Operation::Mul,
				Operation::Inv,
				Operation::Incr,
				Operation::And,
				Operation::Or,
				Operation::Not,
				Operation::Eq,
				Operation::Eqz,
				Operation::Expacc,
				Operation::Ext2Mul,
				Operation::U32split,
				Operation::U32add,
				Operation::U32assert2(222),
				Operation::U32add3,
				Operation::U32sub,
				Operation::U32mul,
				Operation::U32madd,
				Operation::U32div,
				Operation::U32and,
				Operation::U32xor,
				Operation::Pad,
				Operation::Drop,
				Operation::Dup0,
				Operation::Dup1,
				Operation::Dup2,
				Operation::Dup3,
				Operation::Dup4,
				Operation::Dup5,
				Operation::Dup6,
				Operation::Dup7,
				Operation::Dup9,
				Operation::Dup11,
				Operation::Dup13,
				Operation::Dup15,
				Operation::Swap,
				Operation::SwapW,
				Operation::SwapW2,
				Operation::SwapW3,
				Operation::SwapDW,
				Operation::MovUp2,
				Operation::MovUp3,
				Operation::MovUp4,
				Operation::MovUp5,
				Operation::MovUp6,
				Operation::MovUp7,
				Operation::MovUp8,
				Operation::MovDn2,
				Operation::MovDn3,
				Operation::MovDn4,
				Operation::MovDn5,
				Operation::MovDn6,
				Operation::MovDn7,
				Operation::MovDn8,
				Operation::CSwap,
				Operation::CSwapW,
				Operation::Push(Felt::new(45)),
				Operation::AdvPop,
				Operation::AdvPopW,
				Operation::MLoadW,
				Operation::MStoreW,
				Operation::MLoad,
				Operation::MStore,
				Operation::MStream,
				Operation::Pipe,
				Operation::HPerm,
				Operation::MpVerify(1022),
				Operation::MrUpdate,
				Operation::FriE2F4,
				Operation::RCombBase,
				Operation::Emit(42),
			];

			let num_operations = operations.len();

			let decorators = vec![
				(
					0,
					Decorator::AsmOp(AssemblyOp::new(
						Some(crate::debuginfo::Location {
							path: Arc::from("test"),
							start: 42.into(),
							end: 43.into(),
						}),
						"context".to_owned(),
						"op".to_owned(),
						15,
						false,
					)),
				),
				(0, Decorator::Debug(DebugOptions::StackAll)),
				(15, Decorator::Debug(DebugOptions::StackTop(255))),
				(15, Decorator::Debug(DebugOptions::MemAll)),
				(15, Decorator::Debug(DebugOptions::MemInterval(0, 16))),
				(17, Decorator::Debug(DebugOptions::LocalInterval(1, 2, 3))),
				(num_operations, Decorator::Trace(55)),
			];

			mast_forest.add_block_with_raw_decorators(operations, decorators)?
		};

		let decorator_id1 = mast_forest.add_decorator(Decorator::Trace(1))?;
		let decorator_id2 = mast_forest.add_decorator(Decorator::Trace(2))?;

		let call_node_id = mast_forest.add_call(basic_block_id)?;
		mast_forest.set_before_enter(call_node_id, [decorator_id1]);
		mast_forest.set_after_exit(call_node_id, [decorator_id2]);

		let syscall_node_id = mast_forest.add_syscall(basic_block_id)?;
		mast_forest.set_before_enter(syscall_node_id, [decorator_id1]);
		mast_forest.set_after_exit(syscall_node_id, [decorator_id2]);

		let loop_node_id = mast_forest.add_loop(basic_block_id)?;
		mast_forest.set_before_enter(loop_node_id, [decorator_id1]);
		mast_forest.set_after_exit(loop_node_id, [decorator_id2]);

		let join_node_id = mast_forest.add_join(basic_block_id, call_node_id)?;
		mast_forest.set_before_enter(join_node_id, [decorator_id1]);
		mast_forest.set_after_exit(join_node_id, [decorator_id2]);

		let split_node_id = mast_forest.add_split(basic_block_id, call_node_id)?;
		mast_forest.set_before_enter(split_node_id, [decorator_id1]);
		mast_forest.set_after_exit(split_node_id, [decorator_id2]);

		let dyn_node_id = mast_forest.add_dyn()?;
		mast_forest.set_before_enter(dyn_node_id, [decorator_id1]);
		mast_forest.set_after_exit(dyn_node_id, [decorator_id2]);

		let dyncall_node_id = mast_forest.add_dyncall()?;
		mast_forest.set_before_enter(dyncall_node_id, [decorator_id1]);
		mast_forest.set_after_exit(dyncall_node_id, [decorator_id2]);

		let external_node_id = mast_forest.add_external(RpoDigest::default())?;
		mast_forest.set_before_enter(external_node_id, [decorator_id1]);
		mast_forest.set_after_exit(external_node_id, [decorator_id2]);

		mast_forest.make_root(join_node_id);
		mast_forest.make_root(syscall_node_id);
		mast_forest.make_root(loop_node_id);
		mast_forest.make_root(split_node_id);
		mast_forest.make_root(dyn_node_id);
		mast_forest.make_root(dyncall_node_id);
		mast_forest.make_root(external_node_id);

		let serialized_mast_forest = mast_forest.to_bytes();
		let deserialized_mast_forest = MastForest::read_from_bytes(&serialized_mast_forest)?;

		assert_eq!(mast_forest, deserialized_mast_forest);

		Ok(())
	}

	#[test]
	fn mast_forest_serialize_deserialize_with_child_ids_exceeding_parent_id() -> eyre::Result<()> {
		let mut forest = MastForest::new();
		let deco0 = forest.add_decorator(Decorator::Trace(0))?;
		let deco1 = forest.add_decorator(Decorator::Trace(1))?;
		let zero = forest.add_block(vec![Operation::U32div], None)?;
		let first = forest.add_block(vec![Operation::U32add], Some(vec![(0, deco0)]))?;
		let second = forest.add_block(vec![Operation::U32and], Some(vec![(1, deco1)]))?;

		forest.add_join(first, second)?;

		forest.nodes.swap_remove(zero.as_usize());

		MastForest::read_from_bytes(&forest.to_bytes())?;

		Ok(())
	}

	#[test]
	fn mast_forest_serialize_deserialize_with_overflowing_ids_fails() -> eyre::Result<()> {
		let mut overflow_forest = MastForest::new();
		let id0 = overflow_forest.add_block(vec![Operation::Eqz], None)?;
		overflow_forest.add_block(vec![Operation::Eqz], None)?;
		let id2 = overflow_forest.add_block(vec![Operation::Eqz], None)?;
		let id_join = overflow_forest.add_join(id0, id2)?;

		let join_node = overflow_forest[id_join].clone();

		let mut forest = MastForest::new();
		let deco0 = forest.add_decorator(Decorator::Trace(0))?;
		let deco1 = forest.add_decorator(Decorator::Trace(1))?;
		forest.add_block(vec![Operation::U32add], Some(vec![(0, deco0), (1, deco1)]))?;
		forest.add_node(join_node)?;

		assert_matches!(MastForest::read_from_bytes(&forest.to_bytes()), Err(DeserializationError::InvalidValue(msg)) if msg.contains("number of nodes"));

		Ok(())
	}

	#[test]
	fn mast_forest_invalid_node_id() -> eyre::Result<()> {
		let mut forest = MastForest::new();
		let first = forest.add_block(vec![Operation::U32div], None)?;
		let second = forest.add_block(vec![Operation::U32div], None)?;

		let mut overflow_forest = MastForest::new();

		let overflow = (0..=3)
			.map(|_| overflow_forest.add_block(vec![Operation::U32div], None))
			.last()
			.unwrap()?;

		let join = forest.add_join(overflow, second);
		assert_eq!(join, Err(MastForestError::NodeIdOverflow(overflow, 2)));
		let join = forest.add_join(first, overflow);
		assert_eq!(join, Err(MastForestError::NodeIdOverflow(overflow, 2)));

		let split = forest.add_split(overflow, second);
		assert_eq!(split, Err(MastForestError::NodeIdOverflow(overflow, 2)));
		let split = forest.add_split(first, overflow);
		assert_eq!(split, Err(MastForestError::NodeIdOverflow(overflow, 2)));

		assert_eq!(
			forest.add_loop(overflow),
			Err(MastForestError::NodeIdOverflow(overflow, 2))
		);

		assert_eq!(
			forest.add_call(overflow),
			Err(MastForestError::NodeIdOverflow(overflow, 2))
		);
		assert_eq!(
			forest.add_syscall(overflow),
			Err(MastForestError::NodeIdOverflow(overflow, 2))
		);

		assert!(forest.add_join(first, second).is_ok());

		Ok(())
	}

	#[test]
	fn mast_forest_serialize_deserialize_advice_map() -> eyre::Result<()> {
		let mut forest = MastForest::new();
		let deco0 = forest.add_decorator(Decorator::Trace(0))?;
		let deco1 = forest.add_decorator(Decorator::Trace(1))?;
		let first = forest.add_block(vec![Operation::U32add], Some(vec![(0, deco0)]))?;
		let second = forest.add_block(vec![Operation::U32add], Some(vec![(1, deco1)]))?;
		forest.add_join(first, second)?;

		let key = RpoDigest::new([ONE; 4]);
		let value = vec![ONE, ONE];

		forest.advice_map_mut().insert(key, value);

		let parsed = MastForest::read_from_bytes(&forest.to_bytes())?;
		assert_eq!(forest.advice_map(), parsed.advice_map());

		Ok(())
	}
}
