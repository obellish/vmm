use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Clone, Copy)]
pub struct CombineIncInstrPass;

impl PeepholePass for CombineIncInstrPass {
	const SIZE: usize = 2;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		if let [Instruction::IncVal(i1), Instruction::IncVal(i2)] = window {
			if *i1 == -*i2 {
				Some(Change::Remove)
			} else {
				Some(Change::ReplaceOne(Instruction::IncVal(
					i1.wrapping_add(*i2),
				)))
			}
		} else {
			None
		}
	}
}
