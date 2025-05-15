use tracing::trace;
use vmm_ir::Instruction;
use vmm_utils::InsertOrPush as _;

#[derive(Debug, Clone)]
pub enum Change {
	Remove,
	Replace(Vec<Instruction>),
	ReplaceOne(Instruction),
}

impl Change {
	pub fn apply(self, ops: &mut Vec<Instruction>, i: usize, size: usize) -> (bool, usize) {
		match self {
			Self::Remove => {
				let removed = ops.drain(i..(i + size)).collect::<Vec<_>>();

				trace!("removing instructions {removed:?}");

				(true, 0)
			}
			Self::Replace(instrs) => {
				let mut replaced = Vec::with_capacity(size);
				for _ in 0..size {
					replaced.push(ops.remove(i));
				}

				trace!("replacing instructions {replaced:?} with {instrs:?}");

				for instr in instrs.into_iter().rev() {
					ops.insert_or_push(i, instr);
				}

				(true, 0)
			}
			Self::ReplaceOne(instr) => {
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
