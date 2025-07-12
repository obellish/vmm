use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
};

use super::{
	BasicBlock, Function, IntoValueData, IntoValueDataWith, Type, Value, ValueData, ValueKind,
};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Integer(i32);

impl Integer {
	#[must_use]
	pub const fn value(self) -> i32 {
		self.0
	}

	pub const fn value_mut(&mut self) -> &mut i32 {
		&mut self.0
	}
}

impl Display for Integer {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl IntoValueDataWith<i32> for Integer {
	fn into_value_data(ty: Type, data: i32) -> ValueData {
		assert!(ty.is_i32());
		ValueData::new(ty, ValueKind::Integer(Self(data)))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ZeroInit;

impl IntoValueData for ZeroInit {
	fn into_value_data(ty: Type) -> ValueData {
		ValueData::new(ty, ValueKind::ZeroInit(Self))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Undef;

impl IntoValueData for Undef {
	fn into_value_data(ty: Type) -> ValueData {
		ValueData::new(ty, ValueKind::Undef(Self))
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Aggregate {
	elements: Vec<Value>,
}

impl Aggregate {
	pub(crate) fn into_value_data(ty: Type, elems: impl IntoIterator<Item = Value>) -> ValueData {
		ValueData::new(
			ty,
			ValueKind::Aggregate(Self {
				elements: elems.into_iter().collect(),
			}),
		)
	}

	#[must_use]
	pub const fn elements(&self) -> &Vec<Value> {
		&self.elements
	}

	pub const fn elements_mut(&mut self) -> &mut Vec<Value> {
		&mut self.elements
	}
}

impl Deref for Aggregate {
	type Target = Vec<Value>;

	fn deref(&self) -> &Self::Target {
		self.elements()
	}
}

impl DerefMut for Aggregate {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.elements_mut()
	}
}

impl<I> IntoValueDataWith<I> for Aggregate
where
	I: IntoIterator<Item = Value>,
{
	fn into_value_data(ty: Type, data: I) -> ValueData {
		ValueData::new(
			ty,
			ValueKind::Aggregate(Self {
				elements: data.into_iter().collect(),
			}),
		)
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
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

impl IntoValueDataWith<usize> for FuncArgRef {
	fn into_value_data(ty: Type, data: usize) -> ValueData {
		ValueData::new(ty, ValueKind::FuncArgRef(Self { index: data }))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
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

impl IntoValueDataWith<usize> for BlockArgRef {
	fn into_value_data(ty: Type, data: usize) -> ValueData {
		ValueData::new(ty, ValueKind::BlockArgRef(Self { index: data }))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Alloc;

impl IntoValueData for Alloc {
	fn into_value_data(ty: Type) -> ValueData {
		assert!(!ty.is_unit(), "can not create alloc for unit type");
		ValueData::new(ty, ValueKind::Alloc(Self))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GlobalAlloc {
	init: Value,
}

impl GlobalAlloc {
	#[must_use]
	pub const fn init(self) -> Value {
		self.init
	}

	pub const fn init_mut(&mut self) -> &mut Value {
		&mut self.init
	}
}

impl IntoValueDataWith<Value> for GlobalAlloc {
	fn into_value_data(ty: Type, data: Value) -> ValueData {
		ValueData::new(ty, ValueKind::GlobalAlloc(Self { init: data }))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Load {
	src: Value,
}

impl Load {
	#[must_use]
	pub const fn src(self) -> Value {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut Value {
		&mut self.src
	}
}

impl IntoValueDataWith<Value> for Load {
	fn into_value_data(ty: Type, data: Value) -> ValueData {
		ValueData::new(ty, ValueKind::Load(Self { src: data }))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Store {
	value: Value,
	dest: Value,
}

impl Store {
	#[must_use]
	pub const fn value(self) -> Value {
		self.value
	}

	pub const fn value_mut(&mut self) -> &mut Value {
		&mut self.value
	}

	#[must_use]
	pub const fn dest(self) -> Value {
		self.dest
	}

	pub const fn dest_mut(&mut self) -> &mut Value {
		&mut self.dest
	}
}

impl IntoValueDataWith<(Value, Value)> for Store {
	fn into_value_data(ty: Type, (value, dest): (Value, Value)) -> ValueData {
		assert!(ty.is_unit());
		ValueData::new(ty, ValueKind::Store(Self { value, dest }))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GetPtr {
	src: Value,
	index: Value,
}

impl GetPtr {
	#[must_use]
	pub const fn src(self) -> Value {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut Value {
		&mut self.src
	}

	#[must_use]
	pub const fn index(self) -> Value {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut Value {
		&mut self.index
	}
}

impl IntoValueDataWith<(Value, Value)> for GetPtr {
	fn into_value_data(ty: Type, (src, index): (Value, Value)) -> ValueData {
		ValueData::new(ty, ValueKind::GetPtr(Self { src, index }))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GetElementPtr {
	src: Value,
	index: Value,
}

impl GetElementPtr {
	#[must_use]
	pub const fn src(self) -> Value {
		self.src
	}

	pub const fn src_mut(&mut self) -> &mut Value {
		&mut self.src
	}

	#[must_use]
	pub const fn index(self) -> Value {
		self.index
	}

	pub const fn index_mut(&mut self) -> &mut Value {
		&mut self.index
	}
}

impl IntoValueDataWith<(Value, Value)> for GetElementPtr {
	fn into_value_data(ty: Type, (src, index): (Value, Value)) -> ValueData {
		ValueData::new(ty, ValueKind::GetElementPtr(Self { src, index }))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Binary {
	lhs: Value,
	rhs: Value,
	op: BinaryOp,
}

impl Binary {
	#[must_use]
	pub const fn lhs(self) -> Value {
		self.lhs
	}

	pub const fn lhs_mut(&mut self) -> &mut Value {
		&mut self.lhs
	}

	#[must_use]
	pub const fn rhs(self) -> Value {
		self.rhs
	}

	pub const fn rhs_mut(&mut self) -> &mut Value {
		&mut self.rhs
	}

	#[must_use]
	pub const fn op(self) -> BinaryOp {
		self.op
	}

	pub const fn op_mut(&mut self) -> &mut BinaryOp {
		&mut self.op
	}
}

impl IntoValueDataWith<(Value, BinaryOp, Value)> for Binary {
	fn into_value_data(ty: Type, (lhs, op, rhs): (Value, BinaryOp, Value)) -> ValueData {
		ValueData::new(ty, ValueKind::Binary(Self { lhs, rhs, op }))
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Return {
	value: Option<Value>,
}

impl Return {
	#[must_use]
	pub const fn value(self) -> Option<Value> {
		self.value
	}

	pub const fn value_mut(&mut self) -> &mut Option<Value> {
		&mut self.value
	}
}

impl IntoValueDataWith<Option<Value>> for Return {
	fn into_value_data(ty: Type, data: Option<Value>) -> ValueData {
		assert!(ty.is_unit());
		ValueData::new(ty, ValueKind::Return(Self { value: data }))
	}
}

#[derive(Debug, Clone)]
pub struct Branch {
	cond: Value,
	true_basic_block: BasicBlock,
	false_basic_block: BasicBlock,
	true_args: Vec<Value>,
	false_args: Vec<Value>,
}

impl Branch {
	#[must_use]
	pub const fn cond(&self) -> Value {
		self.cond
	}

	pub const fn cond_mut(&mut self) -> &mut Value {
		&mut self.cond
	}

	#[must_use]
	pub const fn true_basic_block(&self) -> BasicBlock {
		self.true_basic_block
	}

	pub const fn true_basic_block_mut(&mut self) -> &mut BasicBlock {
		&mut self.true_basic_block
	}

	#[must_use]
	pub const fn false_basic_block(&self) -> BasicBlock {
		self.false_basic_block
	}

	pub const fn false_basic_block_mut(&mut self) -> &mut BasicBlock {
		&mut self.false_basic_block
	}

	#[must_use]
	pub const fn true_args(&self) -> &Vec<Value> {
		&self.true_args
	}

	pub const fn true_args_mut(&mut self) -> &mut Vec<Value> {
		&mut self.true_args
	}

	#[must_use]
	pub const fn false_args(&self) -> &Vec<Value> {
		&self.false_args
	}

	pub const fn false_args_mut(&mut self) -> &mut Vec<Value> {
		&mut self.false_args
	}
}

impl IntoValueDataWith<(Value, BasicBlock, BasicBlock)> for Branch {
	fn into_value_data(
		ty: Type,
		(cond, true_bb, false_bb): (Value, BasicBlock, BasicBlock),
	) -> ValueData {
		Self::into_value_data(ty, (cond, true_bb, false_bb, [], []))
	}
}

impl<T, F> IntoValueDataWith<(Value, BasicBlock, BasicBlock, T, F)> for Branch
where
	T: IntoIterator<Item = Value>,
	F: IntoIterator<Item = Value>,
{
	fn into_value_data(
		ty: Type,
		(cond, true_bb, false_bb, true_args, false_args): (Value, BasicBlock, BasicBlock, T, F),
	) -> ValueData {
		assert!(ty.is_unit());
		ValueData::new(
			ty,
			ValueKind::Branch(Self {
				cond,
				true_basic_block: true_bb,
				false_basic_block: false_bb,
				true_args: true_args.into_iter().collect(),
				false_args: false_args.into_iter().collect(),
			}),
		)
	}
}

#[derive(Debug, Clone)]
pub struct Jump {
	target: BasicBlock,
	args: Vec<Value>,
}

impl Jump {
	#[must_use]
	pub const fn target(&self) -> BasicBlock {
		self.target
	}

	pub const fn target_mut(&mut self) -> &mut BasicBlock {
		&mut self.target
	}

	#[must_use]
	pub const fn args(&self) -> &Vec<Value> {
		&self.args
	}

	pub const fn args_mut(&mut self) -> &mut Vec<Value> {
		&mut self.args
	}
}

impl IntoValueDataWith<BasicBlock> for Jump {
	fn into_value_data(ty: Type, data: BasicBlock) -> ValueData {
		Self::into_value_data(ty, (data, []))
	}
}

impl<I> IntoValueDataWith<(BasicBlock, I)> for Jump
where
	I: IntoIterator<Item = Value>,
{
	fn into_value_data(ty: Type, (target, args): (BasicBlock, I)) -> ValueData {
		ValueData::new(
			ty,
			ValueKind::Jump(Self {
				target,
				args: args.into_iter().collect(),
			}),
		)
	}
}

#[derive(Debug, Clone)]
pub struct Call {
	callee: Function,
	args: Vec<Value>,
}

impl Call {
	#[must_use]
	pub const fn callee(&self) -> Function {
		self.callee
	}

	pub const fn callee_mut(&mut self) -> &mut Function {
		&mut self.callee
	}

	#[must_use]
	pub const fn args(&self) -> &Vec<Value> {
		&self.args
	}

	pub const fn args_mut(&mut self) -> &mut Vec<Value> {
		&mut self.args
	}
}

impl<I> IntoValueDataWith<(Function, I)> for Call
where
	I: IntoIterator<Item = Value>,
{
	fn into_value_data(ty: Type, (callee, args): (Function, I)) -> ValueData {
		ValueData::new(
			ty,
			ValueKind::Call(Self {
				callee,
				args: args.into_iter().collect(),
			}),
		)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
	Eq,
	NotEq,
	Gt,
	Ge,
	Lt,
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
		f.write_str(match *self {
			Self::Eq => "eq",
			Self::NotEq => "ne",
			Self::Gt => "gt",
			Self::Ge => "ge",
			Self::Lt => "lt",
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
