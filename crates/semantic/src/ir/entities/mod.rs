mod iter;

use alloc::{string::String, vec::Vec};

use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

pub use self::iter::*;
use super::{
	Aggregate, Alloc, Binary, BlockArgRef, Branch, Call, FuncArgRef, GetElementPtr, GetPtr,
	GlobalAlloc, Integer, Jump, Load, Return, Store, Type, Undef, ValueId, ZeroInit,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicBlockData {
	name: Option<String>,
	params: Vec<ValueId>,
	pub(crate) used_by: HashSet<ValueId>,
}

impl BasicBlockData {
	pub(crate) fn new() -> Self {
		Self::create(None, Vec::new())
	}

	pub(crate) fn with_name(name: impl Into<String>) -> Self {
		Self::create(Some(name.into()), Vec::new())
	}

	pub(crate) fn with_params(params: Vec<ValueId>) -> Self {
		Self::create(None, params)
	}

	pub(crate) fn with_name_and_params(name: impl Into<String>, params: Vec<ValueId>) -> Self {
		Self::create(Some(name.into()), params)
	}

	#[must_use]
	pub const fn params(&self) -> &Vec<ValueId> {
		&self.params
	}

	pub const fn params_mut(&mut self) -> &mut Vec<ValueId> {
		&mut self.params
	}

	#[must_use]
	pub const fn used_by(&self) -> &HashSet<ValueId> {
		&self.used_by
	}

	fn create(name: Option<String>, params: Vec<ValueId>) -> Self {
		Self {
			name,
			params,
			used_by: HashSet::new(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueData {
	ty: Type,
	name: Option<String>,
	value_ty: ValueType,
	pub(crate) used_by: HashSet<ValueId>,
}

impl ValueData {
	pub(crate) fn new(ty: Type, value_ty: ValueType) -> Self {
		Self {
			ty,
			name: None,
			value_ty,
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

	pub fn set_name(&mut self, name: impl Into<String>) {
		let name: String = name.into();
		debug_assert!(name.len() > 1 && name.starts_with(['%', '@']));

		self.name = Some(name);
	}

	#[must_use]
	pub const fn value_ty(&self) -> &ValueType {
		&self.value_ty
	}

	pub const fn value_ty_mut(&mut self) -> &mut ValueType {
		&mut self.value_ty
	}

	#[must_use]
	pub const fn used_by(&self) -> &HashSet<ValueId> {
		&self.used_by
	}
}

impl Clone for ValueData {
	fn clone(&self) -> Self {
		Self {
			ty: self.ty.clone(),
			name: self.name.clone(),
			value_ty: self.value_ty.clone(),
			used_by: HashSet::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
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
	Branch(Branch),
	Jump(Jump),
	Call(Call),
	Return(Return),
}

impl ValueType {
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
	pub const fn is_local_instruction(&self) -> bool {
		matches!(
			self,
			Self::Alloc(..)
				| Self::Load(..)
				| Self::Store(..)
				| Self::GetPtr(..)
				| Self::GetElementPtr(..)
				| Self::Binary(..)
				| Self::Jump(..)
				| Self::Call(..)
				| Self::Return(..)
		)
	}

	#[must_use]
	pub const fn values(&self) -> ValueIter<'_> {
		ValueIter::new(self)
	}

	#[must_use]
	pub const fn basic_blocks(&self) -> BasicBlockIter<'_> {
		BasicBlockIter::new(self)
	}
}
