use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	iter::FusedIterator,
	rc::Weak,
};

use super::{
	Aggregate, Alloc, BasicBlockId, Binary, BlockArgRef, Branch, Call, FuncArgRef, FunctionId,
	GetElementPtr, GetPtr, GlobalAlloc, Integer, Jump, Load, Return, Store, Type, Undef, ValueId,
	ZeroInit, is_global_id,
};

#[derive(Debug)]
pub struct ValueData {
	ty: Type,
	name: Option<String>,
	kind: ValueKind,
	pub(crate) used_by: HashSet<Value>,
}

impl ValueData {
	pub(crate) fn new(ty: Type, kind: ValueKind) -> Self {
		Self {
			ty,
			name: None,
			kind,
			used_by: HashSet::new(),
		}
	}

	#[must_use]
	pub const fn ty(&self) -> &Type {
		&self.ty
	}

	#[must_use]
	pub const fn name(&self) -> Option<&String> {
		self.name.as_ref()
	}

	pub(crate) fn set_name(&mut self, name: Option<String>) {
		assert!(
			name.as_ref()
				.is_none_or(|n| n.len() > 1 && (n.starts_with(['%', '@']))),
			"invalid value name"
		);
		self.name = name;
	}

	#[must_use]
	pub const fn kind(&self) -> &ValueKind {
		&self.kind
	}

	pub const fn kind_mut(&mut self) -> &mut ValueKind {
		&mut self.kind
	}

	#[must_use]
	pub const fn used_by(&self) -> &HashSet<Value> {
		&self.used_by
	}
}

impl Clone for ValueData {
	fn clone(&self) -> Self {
		Self {
			ty: self.ty.clone(),
			name: self.name.clone(),
			kind: self.kind.clone(),
			used_by: HashSet::new(),
		}
	}
}

impl From<i32> for ValueData {
	fn from(value: i32) -> Self {
		Integer::into_value_data(Type::i32(), value)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Value(pub(crate) ValueId);

impl Value {
	#[must_use]
	pub fn is_global(self) -> bool {
		is_global_id(self.0)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Function(FunctionId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BasicBlock(pub(crate) BasicBlockId);

pub struct BasicBlockData {
	name: Option<String>,
	params: Vec<Value>,
	pub(crate) used_by: HashSet<Value>,
}

impl BasicBlockData {
	pub(crate) fn new(name: Option<impl Into<String>>) -> Self {
		Self {
			name: name.map(Into::into),
			params: Vec::new(),
			used_by: HashSet::new(),
		}
	}

	pub(crate) fn with_params(
		name: Option<impl Into<String>>,
		params: impl IntoIterator<Item = Value>,
	) -> Self {
		Self {
			name: name.map(Into::into),
			params: params.into_iter().collect(),
			used_by: HashSet::new(),
		}
	}

	#[must_use]
	pub const fn name(&self) -> Option<&String> {
		self.name.as_ref()
	}

	pub fn set_name(&mut self, name: Option<impl Into<String>>) {
		self.name = name.map(Into::into);
	}

	#[must_use]
	pub const fn params(&self) -> &Vec<Value> {
		&self.params
	}

	pub const fn params_mut(&mut self) -> &mut Vec<Value> {
		&mut self.params
	}

	#[must_use]
	pub const fn used_by(&self) -> &HashSet<Value> {
		&self.used_by
	}
}

pub struct BasicBlockUses<'a> {
	kind: &'a ValueKind,
	index: usize,
}

impl FusedIterator for BasicBlockUses<'_> {}

impl Iterator for BasicBlockUses<'_> {
	type Item = BasicBlock;

	fn next(&mut self) -> Option<Self::Item> {
		let curr = self.index;
		self.index += 1;
		match self.kind {
			ValueKind::Branch(br) => match curr {
				0 => Some(br.true_basic_block()),
				1 => Some(br.false_basic_block()),
				_ => None,
			},
			ValueKind::Jump(jump) => match curr {
				0 => Some(jump.target()),
				_ => None,
			},
			_ => None,
		}
	}
}

pub struct ValueUses<'a> {
	kind: &'a ValueKind,
	index: usize,
}

impl FusedIterator for ValueUses<'_> {}

impl Iterator for ValueUses<'_> {
	type Item = Value;

	fn next(&mut self) -> Option<Self::Item> {
		let curr = self.index;
		self.index += 1;

		macro_rules! vec_use {
			($vec:expr) => {
				if curr < $vec.len() {
					Some($vec[curr])
				} else {
					None
				}
			};
		}

		macro_rules! field_use {
			($($field:expr),+) => {
				field_use!(@expand 0 $(,$field)+)
			};
			(@expand $index:expr) => {
				None
			};
			(@expand $index:expr, $head:expr $(,$tail:expr)*) => {
				if curr == $index {
					Some($head)
				} else {
					field_use!(@expand $index + 1 $(,$tail)*)
				}
			}
		}

		match self.kind {
			ValueKind::Aggregate(v) => vec_use!(v.elements()),
			ValueKind::GlobalAlloc(v) => field_use!(v.init()),
			ValueKind::Load(v) => field_use!(v.src()),
			ValueKind::Store(v) => field_use!(v.value(), v.dest()),
			ValueKind::GetPtr(v) => field_use!(v.src(), v.index()),
			ValueKind::GetElementPtr(v) => field_use!(v.src(), v.index()),
			ValueKind::Binary(v) => field_use!(v.lhs(), v.rhs()),
			ValueKind::Branch(v) => {
				let tlen = v.true_args().len();
				if matches!(curr, 0) {
					Some(v.cond())
				} else if curr >= 1 && curr <= tlen {
					Some(v.true_args()[curr - 1])
				} else if curr > tlen && curr <= tlen + v.false_args().len() {
					Some(v.false_args()[curr - tlen - 1])
				} else {
					None
				}
			}
			ValueKind::Jump(v) => vec_use!(v.args()),
			ValueKind::Call(v) => vec_use!(v.args()),
			ValueKind::Return(v) => match curr {
				0 => v.value(),
				_ => None,
			},
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum ValueKind {
	Integer(Integer),
	ZeroInit(ZeroInit),
	Undef(Undef),
	Aggregate(Aggregate),
	FuncArgRef(FuncArgRef),
	BlockArgRef(BlockArgRef),
	Alloc(Alloc),
	GlobalAlloc(GlobalAlloc),
	Load(Load),
	Store(Store),
	GetPtr(GetPtr),
	GetElementPtr(GetElementPtr),
	Binary(Binary),
	Return(Return),
	Call(Call),
	Jump(Jump),
	Branch(Branch),
}

impl ValueKind {
	#[must_use]
	pub const fn value_uses(&self) -> ValueUses<'_> {
		ValueUses {
			kind: self,
			index: 0,
		}
	}

	#[must_use]
	pub const fn basic_block_uses(&self) -> BasicBlockUses<'_> {
		BasicBlockUses {
			kind: self,
			index: 0,
		}
	}

	#[must_use]
	pub const fn is_const(&self) -> bool {
		matches!(
			self,
			Self::Integer(..) | Self::ZeroInit(..) | Self::Undef(..) | Self::Aggregate(..)
		)
	}

	#[must_use]
	pub const fn is_global_alloc(&self) -> bool {
		matches!(self, Self::GlobalAlloc(..))
	}

	#[must_use]
	pub const fn is_local_inst(&self) -> bool {
		matches!(
			self,
			Self::Alloc(..)
				| Self::Load(..)
				| Self::Store(..)
				| Self::GetPtr(..)
				| Self::GetElementPtr(..)
				| Self::Binary(..)
				| Self::Branch(..)
				| Self::Jump(..)
				| Self::Call(..)
				| Self::Return(..)
		)
	}
}

pub(crate) trait IntoValueData {
	fn into_value_data(ty: Type) -> ValueData;
}

pub(crate) trait IntoValueDataWith<I> {
	fn into_value_data(ty: Type, data: I) -> ValueData;
}

pub(crate) type GlobalValueMapCell = Weak<RefCell<HashMap<Value, ValueData>>>;

pub(crate) type FuncTypeMapCell = Weak<RefCell<HashMap<Function, Type>>>;
