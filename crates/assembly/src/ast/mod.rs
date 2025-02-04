mod attribute;
mod ident;
mod immediate;

pub use self::{
	attribute::Attribute,
	ident::{CaseKindError, Ident, IdentError},
	immediate::{ErrorCode, ImmFelt, ImmU8, ImmU16, ImmU32, Immediate},
};

pub const MAX_STACK_WORD_OFFSET: u8 = 12;
