use tracing::{Level, trace};
use vmm_ir::Instruction;
use vmm_num::Wrapping;
use vmm_utils::InsertOrPush as _;
use vmm_vec::SmallVec;

#[derive(Debug, Clone)]
pub enum Change {
	Remove,
	RemoveOffset(isize),
	Swap(SmallVec<Instruction, 2>),
	Replace(Instruction),
}

impl Change {
	#[inline]
	#[must_use]
	pub const fn remove() -> Self {
		Self::Remove
	}

	#[inline]
	#[must_use]
	pub const fn remove_offset(offset: isize) -> Self {
		Self::RemoveOffset(offset)
	}

	#[inline]
	#[must_use]
	pub fn swap<I>(instrs: I) -> Self
	where
		I: IntoIterator<Item = Instruction>,
	{
		Self::Swap(instrs.into_iter().collect())
	}

	#[inline]
	#[must_use]
	pub const fn replace(i: Instruction) -> Self {
		Self::Replace(i)
	}

	#[inline]
	#[tracing::instrument(skip(self, ops, size), level = Level::TRACE)]
	pub fn apply(self, ops: &mut Vec<Instruction>, i: usize, size: usize) -> (bool, usize) {
		match self {
			Self::Remove => {
				let removed = ops.drain(i..(i + size)).collect::<SmallVec<_, 2>>();

				trace!("removing instructions {removed:?}");

				(true, 0)
			}
			Self::RemoveOffset(offset) => {
				let removed = ops.remove(Wrapping::add(i, offset));

				trace!("removing instruction {removed:?}");

				(true, 0)
			}
			Self::Swap(instrs) => {
				let mut replaced = SmallVec::<_, 4>::with_capacity(size);
				for _ in 0..size {
					replaced.push(ops.remove(i));
				}

				trace!("swapping instructions {replaced:?} with {instrs:?}");

				for instr in instrs.into_iter().rev() {
					ops.insert_or_push(i, instr);
				}

				(true, 0)
			}
			Self::Replace(instr) => {
				let mut replaced = SmallVec::<_, 4>::with_capacity(size);
				for _ in 0..size {
					replaced.push(ops.remove(i));
				}

				trace!("replacing instructions {replaced:?} with {instr:?}");

				ops.insert_or_push(i, instr);

				(true, 0)
			}
		}
	}
}
