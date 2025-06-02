use vmm_ir::{Instruction, Offset};
use vmm_utils::{GetOrZero as _, Sorted as _};

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct ReorderRelativeChangesPass;

impl PeepholePass for ReorderRelativeChangesPass {
	const SIZE: usize = 3;

	#[allow(clippy::many_single_char_names)]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					offset: None,
					value: a,
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					value: b,
				},
				Instruction::IncVal {
					offset: None,
					value: c,
				},
			] => Some(Change::Replace(vec![
				Instruction::inc_val(*a),
				Instruction::inc_val(*c),
				Instruction::inc_val_at(*b, x),
			])),
			[
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					value: a,
				},
				Instruction::IncVal {
					offset: None,
					value: b,
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value: c,
				},
			] if *x == *y => Some(Change::Replace(vec![
				Instruction::inc_val_at(*a, x),
				Instruction::inc_val_at(*c, y),
				Instruction::inc_val(*b),
			])),
			[
				Instruction::SetVal { offset: None, .. },
				Instruction::SetVal {
					offset: Some(Offset::Relative(x)),
					value: a,
				},
				Instruction::SetVal {
					offset: None,
					value: b,
				},
			] => Some(Change::Replace(vec![
				Instruction::set_val(b.get_or_zero()),
				Instruction::set_val_at(a.get_or_zero(), x),
			])),
			[
				Instruction::IncVal {
					offset: Some(Offset::Relative(x)),
					value: a,
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(y)),
					value: b,
				},
				Instruction::IncVal {
					offset: Some(Offset::Relative(z)),
					value: c,
				},
			] if *x == *z && *x != *y => Some(Change::Replace(
				vec![
					Instruction::inc_val_at(*a, *x),
					Instruction::inc_val_at(*c, *z),
					Instruction::inc_val_at(*b, *y),
				]
				.sorted_by_key(Instruction::offset),
			)),
			_ => None,
		}
	}

	#[allow(clippy::many_single_char_names)]
	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::IncVal {
					offset: Some(Offset::Relative(_)),
					..
				},
				Instruction::IncVal { offset: None, .. }
			] | [
				Instruction::SetVal { offset: None, .. },
				Instruction::SetVal {
					offset: Some(Offset::Relative(_)),
					..
				},
				Instruction::SetVal { offset: None, .. }
			]
		) || matches!(
			window,
			[
				Instruction::IncVal {
					value: a,
					offset: Some(Offset::Relative(x)),
					..
				},
				Instruction::IncVal { offset: Some(Offset::Relative(z)), .. },
				Instruction::IncVal {
					value: b,
					offset: Some(Offset::Relative(y)),
					..
				}
			]
			if *x == *y && a != b && *x != *z
		)
	}
}
