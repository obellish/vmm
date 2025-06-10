use tracing::{Level, trace};
use vmm_ir::Instruction;
use vmm_num::Wrapping;
use vmm_utils::InsertOrPush as _;

#[derive(Debug, Clone)]
pub enum Change {
	Remove,
	RemoveOffset(isize),
	Swap(Vec<Instruction>),
	Replace(Instruction),
}

impl Change {
	#[must_use]
	pub const fn remove() -> Self {
		Self::Remove
	}

	#[must_use]
	pub const fn remove_offset(offset: isize) -> Self {
		Self::RemoveOffset(offset)
	}

	#[must_use]
	pub fn swap<I>(instrs: I) -> Self
	where
		I: IntoIterator<Item = Instruction>,
	{
		Self::Swap(instrs.into_iter().collect())
	}

	#[must_use]
	pub const fn replace(i: Instruction) -> Self {
		Self::Replace(i)
	}

	#[tracing::instrument(skip(self, ops, size), level = Level::TRACE)]
	pub fn apply(self, ops: &mut Vec<Instruction>, i: usize, size: usize) -> (bool, usize) {
		match self {
			Self::Remove => {
				let removed = ops.drain(i..(i + size)).collect::<Vec<_>>();

				trace!("removing instructions {removed:?}");

				(true, 0)
			}
			Self::RemoveOffset(offset) => {
				let removed = ops.remove(Wrapping::add(i, offset));

				trace!("removing instruction {removed:?}");

				(true, 0)
			}
			Self::Swap(instrs) => {
				let mut replaced = Vec::with_capacity(size);
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
				let mut replaced = Vec::with_capacity(size);
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
