use alloc::boxed::Box;
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
	slice,
};

use serde::{Deserialize, Serialize};

use super::{HasIo, Instruction, IsOffsetable, IsZeroingCell, MinimumOutputs, Offset, PtrMovement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockInstruction {
	/// A dynamic loop, which checks the current cell for zero before executing
	DynamicLoop(Box<[Instruction]>),
	/// An if-non-zero block, which zeros out the cell after executing
	IfNz(Box<[Instruction]>),
}

impl BlockInstruction {
	pub fn dynamic(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::DynamicLoop(i.into_iter().collect())
	}

	pub fn if_nz(i: impl IntoIterator<Item = Instruction>) -> Self {
		Self::IfNz(i.into_iter().collect())
	}
}

impl Deref for BlockInstruction {
	type Target = [Instruction];

	fn deref(&self) -> &Self::Target {
		match self {
			Self::DynamicLoop(block) | Self::IfNz(block) => block,
		}
	}
}

impl DerefMut for BlockInstruction {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::DynamicLoop(block) | Self::IfNz(block) => block,
		}
	}
}

impl Display for BlockInstruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::DynamicLoop(instrs) => {
				writeln!(f, "dylop")?;
				for i in instrs {
					writeln!(f, "{i}")?;
				}
				write!(f, "end dylop")?;
			}
			Self::IfNz(instrs) => {
				writeln!(f, "ifnz")?;
				for i in instrs {
					writeln!(f, "{i}")?;
				}
				write!(f, "end ifnz")?;
			}
		}

		Ok(())
	}
}

impl HasIo for BlockInstruction {
	fn has_read(&self) -> bool {
		self.deref().has_read()
	}

	fn has_write(&self) -> bool {
		self.deref().has_write()
	}
}

impl<'a> IntoIterator for &'a BlockInstruction {
	type IntoIter = slice::Iter<'a, Instruction>;
	type Item = &'a Instruction;

	fn into_iter(self) -> Self::IntoIter {
		self.deref().iter()
	}
}

impl IsOffsetable for BlockInstruction {
	fn is_offsetable(&self) -> bool {
		self.deref().is_offsetable()
	}

	fn offset(&self) -> Option<Offset> {
		self.deref().offset()
	}

	fn set_offset(&mut self, offset: Offset) {
		self.deref_mut().set_offset(offset);
	}
}

impl IsZeroingCell for BlockInstruction {
	#[inline]
	fn is_zeroing_cell(&self) -> bool {
		true
	}
}

impl MinimumOutputs for BlockInstruction {
	fn min_outputs(&self) -> usize {
		self.deref().min_outputs()
	}
}

impl PtrMovement for BlockInstruction {
	#[inline]
	fn ptr_movement(&self) -> Option<Offset> {
		self.deref().ptr_movement()
	}
}
