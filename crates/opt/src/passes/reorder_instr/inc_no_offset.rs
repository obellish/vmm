use vmm_ir::Instruction;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderIncNoOffsetPass;

impl PeepholePass for ReorderIncNoOffsetPass {
	const SIZE: usize = 3;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: x,
					offset: None,
				},
				instr,
				Instruction::IncVal {
					value: y,
					offset: None,
				},
			] if matches!(instr.ptr_movement(), Some(0)) && !instr.is_loop() => Some(Change::Replace(vec![
				Instruction::inc_val(*x),
				Instruction::inc_val(*y),
				instr.clone(),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				instr,
				Instruction::IncVal { offset: None, .. }
			]
			if matches!(instr.ptr_movement(), Some(0)) && !instr.is_loop()
		)
	}
}
