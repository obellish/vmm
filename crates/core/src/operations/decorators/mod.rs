mod assembly_op;
mod debug;

use alloc::{string::ToString, vec::Vec};
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use num_traits::ToBytes;

pub use self::{assembly_op::AssemblyOp, debug::DebugOptions};
use crate::{
	crypto::hash::Blake3_256,
	mast::{DecoratorFingerprint, DecoratorId},
	prettier::{Document, PrettyPrint, display},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decorator {
	AsmOp(AssemblyOp),
	Debug(DebugOptions),
	Trace(u32),
}

impl Decorator {
	#[must_use]
	pub fn fingerprint(&self) -> DecoratorFingerprint {
		match self {
			Self::AsmOp(asm_op) => {
				let mut bytes_to_hash = Vec::new();
				if let Some(location) = asm_op.location() {
					bytes_to_hash.extend(location.path.as_bytes());
					bytes_to_hash.extend(location.start.to_u32().to_le_bytes());
					bytes_to_hash.extend(location.end.to_u32().to_le_bytes());
				}
				bytes_to_hash.extend(asm_op.context_name().as_bytes());
				bytes_to_hash.extend(asm_op.op().as_bytes());
				bytes_to_hash.push(asm_op.num_cycles());
				bytes_to_hash.push(u8::from(asm_op.should_break()));

				Blake3_256::hash(&bytes_to_hash)
			}
			Self::Debug(debug) => Blake3_256::hash(debug.to_string().as_bytes()),
			Self::Trace(trace) => Blake3_256::hash(&trace.to_le_bytes()),
		}
	}
}

impl Display for Decorator {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::AsmOp(assembly_op) => {
				f.write_str("asmOp(")?;
				f.write_str(assembly_op.op())?;
				f.write_str(", ")?;
				Display::fmt(&assembly_op.num_cycles(), f)
			}
			Self::Debug(options) => {
				f.write_str("debug(")?;
				Display::fmt(&options, f)?;
				f.write_char(')')
			}
			Self::Trace(trace_id) => {
				f.write_str("trace(")?;
				Display::fmt(&trace_id, f)?;
				f.write_char(')')
			}
		}
	}
}

impl PrettyPrint for Decorator {
	fn render(&self) -> Document {
		display(self)
	}
}

pub struct DecoratorIterator<'a> {
	decorators: &'a DecoratorSlice,
	idx: usize,
}

impl<'a> DecoratorIterator<'a> {
	#[must_use]
	pub const fn new(decorators: &'a DecoratorSlice) -> Self {
		Self { decorators, idx: 0 }
	}

	pub fn next_filtered(&mut self, pos: usize) -> Option<&DecoratorId> {
		if self.idx < self.decorators.len() && self.decorators[self.idx].0 == pos {
			self.idx += 1;
			Some(&self.decorators[self.idx - 1].1)
		} else {
			None
		}
	}
}

impl<'a> Iterator for DecoratorIterator<'a> {
	type Item = &'a DecoratorId;

	fn next(&mut self) -> Option<Self::Item> {
		if self.idx < self.decorators.len() {
			self.idx += 1;
			Some(&self.decorators[self.idx - 1].1)
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureKind {
	RpoFalcon512,
}

impl Display for SignatureKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::RpoFalcon512 => f.write_str("rpo_falcon512"),
		}
	}
}

pub type DecoratorList = Vec<(usize, DecoratorId)>;

pub type DecoratorSlice = [(usize, DecoratorId)];
