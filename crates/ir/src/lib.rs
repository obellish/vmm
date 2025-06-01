#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

mod loop_instr;
mod ptr_movement;
mod simd_instr;
mod super_instr;

use alloc::{string::ToString, vec::Vec};
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	num::NonZeroU8,
};

use serde::{Deserialize, Serialize};
use vmm_utils::GetOrZero as _;

pub use self::{loop_instr::*, ptr_movement::*, simd_instr::*, super_instr::*};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Instruction {
	/// The start of the program
	/// Is a no-op, but allows for other optimizations to be applied
	Start,
	/// Increment the value at the current cell (offset = None) or at an offset
	IncVal {
		value: i8,
		offset: Option<Offset>,
	},
	SubCell {
		offset: Offset,
	},
	/// Set the value at the current cell (offset = None) or at an offset
	SetVal {
		value: Option<NonZeroU8>,
		offset: Option<Offset>,
	},
	/// Multiply self by factor
	ScaleVal {
		factor: u8,
	},
	/// Move the pointer along the tape
	MovePtr(Offset),
	/// Find the next zero, jumping by the value
	FindZero(isize),
	/// Read a value from the input
	Read,
	/// Write the value to an output
	Write {
		count: usize,
		offset: Option<Offset>,
	},
	/// A loop-like instruction.
	Loop(LoopInstruction),
	/// A "Super" instruction, which is an instruction that does more than one action
	Super(SuperInstruction),
	Simd(SimdInstruction),
}

impl Instruction {
	#[must_use]
	pub const fn inc_val(v: i8) -> Self {
		Self::IncVal {
			value: v,
			offset: None,
		}
	}

	#[must_use]
	pub const fn changes_current_cell(&self) -> bool {
		matches!(
			self,
			Self::SetVal { offset: None, .. } | Self::IncVal { offset: None, .. }
		)
	}

	#[must_use]
	pub fn sub_cell(offset: impl Into<Offset>) -> Self {
		Self::SubCell {
			offset: offset.into(),
		}
	}

	#[must_use]
	pub fn scale_and_take_val(factor: u8, offset: impl Into<Offset>) -> Self {
		SuperInstruction::scale_and_take_val(factor, offset).into()
	}

	#[must_use]
	pub const fn dupe_val(offsets: Vec<Offset>) -> Self {
		Self::Super(SuperInstruction::dupe_val(offsets))
	}

	#[must_use]
	pub fn inc_val_at(v: i8, offset: impl Into<Offset>) -> Self {
		Self::IncVal {
			value: v,
			offset: Some(offset.into()),
		}
	}

	#[must_use]
	pub const fn simd_inc_vals(v: i8, offsets: Vec<Option<Offset>>) -> Self {
		Self::Simd(SimdInstruction::inc_vals(v, offsets))
	}

	#[must_use]
	pub const fn simd_set_vals(v: u8, offsets: Vec<Option<Offset>>) -> Self {
		Self::Simd(SimdInstruction::set_vals(v, offsets))
	}

	#[must_use]
	pub const fn find_and_set_zero(v: u8, offset: isize) -> Self {
		Self::Super(SuperInstruction::find_and_set_zero(v, offset))
	}

	#[must_use]
	pub const fn set_val(v: u8) -> Self {
		Self::SetVal {
			value: NonZeroU8::new(v),
			offset: None,
		}
	}

	#[must_use]
	pub fn set_val_at(v: u8, offset: impl Into<Offset>) -> Self {
		Self::SetVal {
			value: NonZeroU8::new(v),
			offset: Some(offset.into()),
		}
	}

	#[must_use]
	pub const fn clear_val() -> Self {
		Self::SetVal {
			value: None,
			offset: None,
		}
	}

	#[must_use]
	pub fn clear_val_at(offset: impl Into<Offset>) -> Self {
		Self::SetVal {
			value: None,
			offset: Some(offset.into()),
		}
	}

	#[must_use]
	pub fn scale_and_move_val(factor: u8, offset: impl Into<Offset>) -> Self {
		SuperInstruction::scale_and_move_val(factor, offset).into()
	}

	#[must_use]
	pub fn fetch_and_scale_val(factor: u8, offset: impl Into<Offset>) -> Self {
		SuperInstruction::fetch_and_scale_val(factor, offset).into()
	}

	#[must_use]
	pub fn move_ptr(offset: impl Into<Offset>) -> Self {
		Self::MovePtr(offset.into())
	}

	#[must_use]
	pub const fn move_ptr_by(offset: isize) -> Self {
		Self::MovePtr(Offset::Relative(offset))
	}

	#[must_use]
	pub const fn move_ptr_to(index: usize) -> Self {
		Self::MovePtr(Offset::Absolute(index))
	}

	#[must_use]
	pub const fn find_zero(jump_by: isize) -> Self {
		Self::FindZero(jump_by)
	}

	#[must_use]
	pub const fn read() -> Self {
		Self::Read
	}

	#[must_use]
	pub const fn write_once() -> Self {
		Self::Write {
			offset: None,
			count: 1,
		}
	}

	#[must_use]
	pub fn write_once_at(offset: impl Into<Offset>) -> Self {
		Self::Write {
			offset: Some(offset.into()),
			count: 1,
		}
	}

	#[must_use]
	pub const fn write_many(count: usize) -> Self {
		Self::Write {
			offset: None,
			count,
		}
	}

	#[must_use]
	pub fn write_many_at(count: usize, offset: impl Into<Offset>) -> Self {
		Self::Write {
			offset: Some(offset.into()),
			count,
		}
	}

	#[must_use]
	pub const fn scale_val(factor: u8) -> Self {
		Self::ScaleVal { factor }
	}

	#[must_use]
	pub fn dynamic_loop(instructions: impl IntoIterator<Item = Self>) -> Self {
		LoopInstruction::dynamic(instructions).into()
	}

	#[must_use]
	pub fn if_nz(instructions: impl IntoIterator<Item = Self>) -> Self {
		LoopInstruction::if_nz(instructions).into()
	}

	#[must_use]
	pub const fn is_overwriting_current_cell(&self) -> bool {
		matches!(
			self,
			Self::SetVal { offset: None, .. }
				| Self::Read | Self::Super(SuperInstruction::ScaleAnd {
				action: ScaleAnd::Move,
				..
			}) | Self::Loop(LoopInstruction::IfNz(..))
		)
	}

	pub fn needs_input(&self) -> bool {
		match self {
			Self::Read => true,
			Self::Loop(LoopInstruction::Dynamic(instrs) | LoopInstruction::IfNz(instrs)) => {
				instrs.iter().any(Self::needs_input)
			}
			_ => false,
		}
	}

	pub fn has_io(&self) -> bool {
		match self {
			Self::Read | Self::Write { .. } => true,
			Self::Loop(LoopInstruction::Dynamic(instrs) | LoopInstruction::IfNz(instrs)) => {
				instrs.iter().any(Self::has_io)
			}
			_ => false,
		}
	}

	#[must_use]
	pub const fn is_inc_val(&self) -> bool {
		matches!(self, Self::IncVal {value, ..} if *value > 0)
	}

	#[must_use]
	pub const fn is_dec_val(&self) -> bool {
		matches!(self, Self::IncVal {value, ..} if *value < 0 )
	}

	#[must_use]
	pub const fn is_set_val(&self) -> bool {
		matches!(self, Self::SetVal { .. })
	}

	#[must_use]
	pub const fn is_clear_val(&self) -> bool {
		matches!(self, Self::SetVal { .. })
	}

	#[must_use]
	pub const fn is_move_val(&self) -> bool {
		matches!(
			self,
			Self::Super(SuperInstruction::ScaleAnd {
				action: ScaleAnd::Move,
				..
			})
		)
	}

	#[must_use]
	pub const fn is_dynamic_loop(&self) -> bool {
		matches!(self, Self::Loop(LoopInstruction::Dynamic(_)))
	}

	#[must_use]
	pub fn is_empty_dynamic_loop(&self) -> bool {
		matches!(self, Self::Loop(LoopInstruction::Dynamic(l)) if l.is_empty())
	}

	#[must_use]
	pub const fn is_io(&self) -> bool {
		self.is_read() || self.is_write()
	}

	#[must_use]
	pub const fn is_read(&self) -> bool {
		matches!(self, Self::Read)
	}

	#[must_use]
	pub const fn is_write(&self) -> bool {
		matches!(self, Self::Write { .. })
	}

	#[must_use]
	pub const fn is_zeroing_cell(&self) -> bool {
		matches!(
			self,
			Self::SetVal {
				value: None,
				offset: None
			} | Self::Loop(LoopInstruction::Dynamic(..) | LoopInstruction::IfNz(..))
				| Self::Super(SuperInstruction::ScaleAnd {
					action: ScaleAnd::Move,
					..
				}) | Self::SubCell { .. }
		)
	}

	pub fn rough_estimate(&self) -> usize {
		match self {
			Self::Loop(LoopInstruction::Dynamic(l)) => {
				l.iter().map(Self::rough_estimate).sum::<usize>() + 2
			}
			Self::Loop(LoopInstruction::IfNz(l)) => {
				l.iter().map(Self::rough_estimate).sum::<usize>() + 1
			}
			_ => 1,
		}
	}

	#[must_use]
	pub fn raw_rough_estimate(&self) -> usize {
		self.to_string().len()
	}

	#[must_use]
	pub const fn offset(&self) -> Option<Offset> {
		match self {
			Self::SetVal { offset, .. } | Self::IncVal { offset, .. } => *offset,
			_ => None,
		}
	}

	#[must_use]
	pub fn nested_loops(&self) -> usize {
		let mut count = 0;

		if let Self::Loop(LoopInstruction::Dynamic(instrs) | LoopInstruction::IfNz(instrs)) = self {
			count += 1;

			for instr in instrs {
				count += instr.nested_loops();
			}
		}

		count
	}
}

impl Display for Instruction {
	#[allow(unreachable_patterns)] // For when we add more instructions.
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::IncVal { value, offset } => {
				f.write_str("inc ")?;
				Display::fmt(&value, f)?;
				if let Some(Offset::Relative(offset)) = *offset {
					f.write_str(" [")?;
					Display::fmt(&offset, f)?;
					f.write_char(']')?;
				}
			}
			Self::SetVal { value, offset } => {
				f.write_str("set ")?;
				Display::fmt(&value.get_or_zero(), f)?;
				if let Some(Offset::Relative(offset)) = *offset {
					f.write_str(" [")?;
					Display::fmt(&offset, f)?;
					f.write_char(']')?;
				}
			}
			Self::MovePtr(Offset::Relative(offset)) => {
				f.write_str("movby ")?;
				Display::fmt(&offset, f)?;
			}
			Self::ScaleVal { factor } => {
				f.write_str("scale ")?;
				Display::fmt(&factor, f)?;
			}
			Self::Write { count, offset } => {
				f.write_str("putc ")?;
				Display::fmt(&count, f)?;
				if let Some(Offset::Relative(offset)) = *offset {
					f.write_str(" [")?;
					Display::fmt(&offset, f)?;
					f.write_char(']')?;
				}
			}
			Self::FindZero(offset) => {
				f.write_str("findz [")?;
				Display::fmt(&offset, f)?;
				f.write_char(']')?;
			}
			Self::Read => f.write_str("getc")?,
			Self::Start => f.write_str("start")?,
			Self::Super(s) => Display::fmt(&s, f)?,
			Self::Loop(l) => Display::fmt(&l, f)?,
			Self::Simd(s) => Display::fmt(&s, f)?,
			_ => f.write_char('*')?,
		}

		Ok(())
	}
}

impl From<LoopInstruction> for Instruction {
	fn from(value: LoopInstruction) -> Self {
		Self::Loop(value)
	}
}

impl From<SimdInstruction> for Instruction {
	fn from(value: SimdInstruction) -> Self {
		Self::Simd(value)
	}
}

impl From<SuperInstruction> for Instruction {
	fn from(value: SuperInstruction) -> Self {
		Self::Super(value)
	}
}

impl PtrMovement for Instruction {
	fn ptr_movement(&self) -> Option<isize> {
		match self {
			Self::Super(s) => s.ptr_movement(),
			Self::Loop(l) => l.ptr_movement(),
			Self::Simd(s) => s.ptr_movement(),
			Self::ScaleVal { .. }
			| Self::SetVal { .. }
			| Self::IncVal { .. }
			| Self::Start
			| Self::Read
			| Self::Write { .. }
			| Self::SubCell { .. } => Some(0),
			Self::MovePtr(Offset::Relative(offset)) => Some(*offset),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Offset {
	Relative(isize),
	Absolute(usize),
}

impl Display for Offset {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let alt = f.alternate();

		match self {
			Self::Relative(offset) => {
				if alt {
					f.write_char('[')?;
				}
				Display::fmt(&offset, f)?;
				if alt {
					f.write_char(']')?;
				}
			}
			Self::Absolute(offset) => {
				if alt {
					f.write_char('{')?;
				}

				Display::fmt(&offset, f)?;
				if alt {
					f.write_char('}')?;
				}
			}
		}

		Ok(())
	}
}

impl From<isize> for Offset {
	fn from(value: isize) -> Self {
		Self::Relative(value)
	}
}

impl From<&isize> for Offset {
	fn from(value: &isize) -> Self {
		(*value).into()
	}
}

impl From<usize> for Offset {
	fn from(value: usize) -> Self {
		Self::Absolute(value)
	}
}

impl From<&usize> for Offset {
	fn from(value: &usize) -> Self {
		(*value).into()
	}
}
