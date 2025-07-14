use core::{
	num::NonZeroU32,
	sync::atomic::{AtomicU32, Ordering},
};

use serde::{Deserialize, Serialize};

use super::{
	Aggregate, Alloc, Binary, BlockArgRef, Branch, Call, FuncArgRef, GetElementPtr, GetPtr,
	GlobalAlloc, Integer, Jump, Load, Return, Store, Undef, ZeroInit,
};

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
}
