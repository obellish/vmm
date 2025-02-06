mod attribute;
mod block;
mod constants;
mod form;
mod ident;
mod immediate;
mod imports;
mod instruction;
mod invocation_target;
mod module;
mod op;
mod procedure;
pub mod visit;

#[doc(inline)]
pub use self::{
	attribute::{
		Attribute, AttributeSet, AttributeSetEntry, AttributeSetOccupiedEntry,
		AttributeSetVacantEntry, Meta, MetaExpr, MetaItem, MetaKeyValue, MetaList, MetaRef,
	},
	block::Block,
	constants::{Constant, ConstantExpr, ConstantOp},
	form::Form,
	ident::{CaseKindError, Ident, IdentError},
	immediate::{ErrorCode, ImmFelt, ImmU8, ImmU16, ImmU32, Immediate},
	imports::Import,
	instruction::{DebugOptions, Instruction, SignatureKind, SystemEventNode},
	invocation_target::{InvocationTarget, Invoke, InvokeKind},
	module::{Module, ModuleKind},
	op::Op,
	procedure::*,
	visit::{Visit, VisitMut},
};

pub const MAX_STACK_WORD_OFFSET: u8 = 12;

pub(crate) type SmallOpsVec = smallvec::SmallVec<[Op; 1]>;
