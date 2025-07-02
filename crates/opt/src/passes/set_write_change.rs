use vmm_ir::Instruction;
use vmm_num::ops::WrappingAdd;
use vmm_utils::GetOrZero as _;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct OptimizeSetWriteIncPass;

impl PeepholePass for OptimizeSetWriteIncPass {
	const SIZE: usize = 3;

    #[allow(clippy::many_single_char_names)]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::SetVal {
					value: a,
					offset: x,
				},
				Instruction::Write { offset: y },
				Instruction::IncVal {
					value: b,
					offset: z,
				},
			] if *x == *y && *y == *z => Some(Change::swap([
				Instruction::set_val_at(a.get_or_zero(), x),
				Instruction::write_once_at(x),
				Instruction::set_val_at(WrappingAdd::wrapping_add(a.get_or_zero(), b), x),
			])),
			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::SetVal { offset: x, .. },
				Instruction::Write { offset: y },
				Instruction::IncVal { offset: z, .. },
			]
			if *x == *y && *y == *z
		)
	}
}
