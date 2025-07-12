use std::collections::HashMap;

use super::*;

macro_rules! data {
	($self:ident, $value:expr) => {
		$self
			.globals
			.upgrade()
			.unwrap()
			.borrow()
			.get(&$value)
			.or_else(|| $self.values.get(&$value))
			.expect("value does not exist")
	};
}

macro_rules! data_mut {
	($self:ident, $value:expr) => {
		$self
			.globals
			.upgrade()
			.unwrap()
			.borrow_mut()
			.get_mut(&$value)
			.or_else(|| $self.values.get_mut(&$value))
			.expect("value does not exist")
	};
}

pub struct DataFlowGraph {
	pub(crate) globals: GlobalValueMapCell,
	pub(crate) func_tys: FuncTypeMapCell,
	values: HashMap<Value, ValueData>,
	basic_blocks: HashMap<BasicBlock, BasicBlockData>,
}

impl DataFlowGraph {
	pub(crate) fn new() -> Self {
		Self {
			globals: GlobalValueMapCell::new(),
			func_tys: FuncTypeMapCell::new(),
			values: HashMap::new(),
			basic_blocks: HashMap::new(),
		}
	}

	pub const fn new_value_builder(&mut self) -> LocalBuilder<'_> {
		LocalBuilder { dfg: self }
	}

	pub(crate) fn insert_value_data(&mut self, data: ValueData) -> Value {
		let value = Value(next_local_value_id());
		for v in data.kind().value_uses() {
			data_mut!(self, v).used_by.insert(value);
		}

		for bb in data.kind().basic_block_uses() {
			self.basic_block_mut(bb).used_by.insert(value);
		}

		self.values.insert(value, data);

		value
	}

	pub fn basic_block_mut(&mut self, basic_block: BasicBlock) -> &mut BasicBlockData {
		self.basic_blocks
			.get_mut(&basic_block)
			.expect("basic block not found")
	}
}
