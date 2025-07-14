use alloc::vec::Vec;
use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

use super::{BasicBlockId, FunctionId, ValueId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub enum Integer {
	I32(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ZeroInit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Undef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Aggregate(Vec<ValueId>);

impl Aggregate {
	#[must_use]
	pub const fn values(&self) -> &Vec<ValueId> {
		&self.0
	}

	pub const fn values_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FuncArgRef {
	index: usize,
}

impl FuncArgRef {
	#[must_use]
	pub const fn index(self) -> usize {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut usize {
		&mut self.index
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct BlockArgRef {
	index: usize,
}

impl BlockArgRef {
	#[must_use]
	pub const fn index(self) -> usize {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut usize {
		&mut self.index
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct GlobalAlloc {
	init: ValueId,
}

impl GlobalAlloc {
	#[must_use]
	pub const fn init(self) -> ValueId {
		self.init
	}

	pub const fn init_mut(&mut self) -> &mut ValueId {
		&mut self.init
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Load {
	src: ValueId,
}

impl Load {
	#[must_use]
	pub const fn src(self) -> ValueId {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut ValueId {
		&mut self.src
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
	value: ValueId,
	dest: ValueId,
}

impl Store {
	#[must_use]
	pub const fn value(self) -> ValueId {
		self.value
	}

	pub const fn value_mut(&mut self) -> &mut ValueId {
		&mut self.value
	}

	#[must_use]
	pub const fn dest(self) -> ValueId {
		self.dest
	}

	pub const fn dest_mut(&mut self) -> &mut ValueId {
		&mut self.dest
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPtr {
	src: ValueId,
	index: ValueId,
}

impl GetPtr {
	#[must_use]
	pub const fn src(self) -> ValueId {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut ValueId {
		&mut self.src
	}

	#[must_use]
	pub const fn index(self) -> ValueId {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut ValueId {
		&mut self.index
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetElementPtr {
	src: ValueId,
	index: ValueId,
}

impl GetElementPtr {
	#[must_use]
	pub const fn src(self) -> ValueId {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut ValueId {
		&mut self.src
	}

	#[must_use]
	pub const fn index(self) -> ValueId {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut ValueId {
		&mut self.index
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binary {
	lhs: ValueId,
	op: BinaryOp,
	rhs: ValueId,
}

impl Binary {
	#[must_use]
	pub const fn lhs(self) -> ValueId {
		self.lhs
	}

	pub const fn lhs_mut(&mut self) -> &mut ValueId {
		&mut self.lhs
	}

	#[must_use]
	pub const fn op(self) -> BinaryOp {
		self.op
	}

	pub const fn op_mut(&mut self) -> &mut BinaryOp {
		&mut self.op
	}

	#[must_use]
	pub const fn rhs(self) -> ValueId {
		self.rhs
	}

	pub const fn rhs_mut(&mut self) -> &mut ValueId {
		&mut self.rhs
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
	cond: ValueId,
	true_values: (BasicBlockId, Vec<ValueId>),
	false_values: (BasicBlockId, Vec<ValueId>),
}

impl Branch {
	#[must_use]
	pub const fn cond(&self) -> ValueId {
		self.cond
	}

	pub const fn cond_mut(&mut self) -> &mut ValueId {
		&mut self.cond
	}

	#[must_use]
	pub const fn true_basic_block(&self) -> BasicBlockId {
		self.true_values.0
	}

	pub const fn true_basic_block_mut(&mut self) -> &mut BasicBlockId {
		&mut self.true_values.0
	}

	#[must_use]
	pub const fn true_args(&self) -> &Vec<ValueId> {
		&self.true_values.1
	}

	pub const fn true_args_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.true_values.1
	}

	#[must_use]
	pub const fn false_basic_block(&self) -> BasicBlockId {
		self.false_values.0
	}

	pub const fn false_basic_block_mut(&mut self) -> &mut BasicBlockId {
		&mut self.false_values.0
	}

	#[must_use]
	pub const fn false_args(&self) -> &Vec<ValueId> {
		&self.false_values.1
	}

	pub const fn false_args_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.false_values.1
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jump {
	target: BasicBlockId,
	args: Vec<ValueId>,
}

impl Jump {
	#[must_use]
	pub const fn target(&self) -> BasicBlockId {
		self.target
	}

	pub const fn target_mut(&mut self) -> &mut BasicBlockId {
		&mut self.target
	}

	#[must_use]
	pub const fn args(&self) -> &Vec<ValueId> {
		&self.args
	}

	pub const fn args_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.args
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Call {
	callee: FunctionId,
	args: Vec<ValueId>,
}

impl Call {
	#[must_use]
	pub const fn callee(&self) -> FunctionId {
		self.callee
	}

	pub const fn callee_mut(&mut self) -> &mut FunctionId {
		&mut self.callee
	}

	#[must_use]
	pub const fn args(&self) -> &Vec<ValueId> {
		&self.args
	}

	pub const fn args_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.args
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Return(Option<ValueId>);

impl Return {
	#[must_use]
	pub const fn value(self) -> Option<ValueId> {
		self.0
	}

	pub const fn value_mut(&mut self) -> &mut Option<ValueId> {
		&mut self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
	NotEq,
	Eq,
	Gt,
	Lt,
	Ge,
	Le,
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	And,
	Or,
	Xor,
	Shl,
	Shr,
	Sar,
}

impl Display for BinaryOp {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::NotEq => "ne",
			Self::Eq => "eq",
			Self::Gt => "gt",
			Self::Lt => "lt",
			Self::Ge => "ge",
			Self::Le => "le",
			Self::Add => "add",
			Self::Sub => "sub",
			Self::Mul => "mul",
			Self::Div => "div",
			Self::Mod => "mod",
			Self::And => "and",
			Self::Or => "or",
			Self::Xor => "xor",
			Self::Shl => "shl",
			Self::Shr => "shr",
			Self::Sar => "sar",
		})
	}
}
