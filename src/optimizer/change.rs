use tracing::trace;

use crate::{Instruction, util::InsertOrPush as _};

#[derive(Debug, Default, Clone)]
pub enum Change {
	#[default]
	Ignore,
	Remove,
	Replace(Vec<Instruction>),
}

impl Change {
	pub fn apply(self, ops: &mut Vec<Instruction>, i: usize, size: usize) -> (bool, usize) {
		match self {
			Self::Ignore => (false, 0),
			Self::Remove => {
				let removed = ops.drain(i..(i + size)).collect::<Vec<_>>();

				trace!("removed instructions {:?}", removed);

				(true, 0)
			}

			Self::Replace(instructions) => {
				let mut replaced = Vec::with_capacity(size);
				for _ in 0..size {
					replaced.push(ops.remove(i));
				}

				trace!("replacing instructions {replaced:?} with {instructions:?}");

				for instr in instructions.into_iter().rev() {
					ops.insert_or_push(i, instr);
				}

				(true, 0)
			}
		}
	}
}
