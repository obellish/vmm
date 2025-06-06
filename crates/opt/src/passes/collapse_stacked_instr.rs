use vmm_ir::Instruction;
use vmm_utils::GetOrZero as _;
use vmm_wrap::ops::WrappingAdd;

use crate::{Change, PeepholePass};

#[derive(Debug, Default)]
pub struct CollapseStackedInstrPass;

impl PeepholePass for CollapseStackedInstrPass {
	const SIZE: usize = 2;

	#[allow(clippy::many_single_char_names)]
	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::IncVal {
					value: i1,
					offset: None,
				},
				Instruction::IncVal {
					value: i2,
					offset: None,
				},
			] if *i1 == -i2 => Some(Change::remove()),
			[
				Instruction::IncVal {
					value: i1,
					offset: None,
				},
				Instruction::IncVal {
					value: i2,
					offset: None,
				},
			] => Some(Change::replace(Instruction::inc_val(
				WrappingAdd::wrapping_add(i1, i2),
			))),
			[Instruction::MovePtr(i1), Instruction::MovePtr(i2)] if *i1 == -i2 => {
				Some(Change::remove())
			}
			[Instruction::MovePtr(i1), Instruction::MovePtr(i2)] => Some(Change::replace(
				Instruction::move_ptr(WrappingAdd::wrapping_add(i1, i2)),
			)),
			[
				Instruction::SetVal { offset: None, .. },
				Instruction::SetVal {
					value: Some(x),
					offset: None,
				},
			] => Some(Change::replace(Instruction::set_val(x.get()))),
			[
				Instruction::IncVal {
					value: i1,
					offset: Some(x),
				},
				Instruction::IncVal {
					value: i2,
					offset: Some(y),
				},
			] if *i1 == -i2 && x == y => Some(Change::remove()),
			[
				Instruction::IncVal {
					value: i1,
					offset: Some(x),
				},
				Instruction::IncVal {
					value: i2,
					offset: Some(y),
				},
			] if x == y => Some(Change::replace(Instruction::inc_val_at(
				WrappingAdd::wrapping_add(i1, i2),
				*x,
			))),
			[
				Instruction::SetVal {
					value: None,
					offset: None,
				},
				Instruction::SetVal {
					value: None,
					offset: None,
				},
			] => Some(Change::remove_offset(1)),
			[
				Instruction::SetVal {
					value: None,
					offset: Some(x),
				},
				Instruction::SetVal {
					value: None,
					offset: Some(y),
				},
			] if *x == *y => Some(Change::remove_offset(1)),
			[
				Instruction::SetVal {
					offset: Some(x), ..
				},
				Instruction::SetVal {
					offset: Some(y),
					value,
				},
			] if *x == *y => Some(Change::replace(Instruction::set_val_at(
				value.get_or_zero(),
				*x,
			))),
			[
				Instruction::SetVal { offset: None, .. },
				Instruction::SetVal {
					value,
					offset: None,
				},
			] => Some(Change::replace(Instruction::set_val(value.get_or_zero()))),
			[
				Instruction::Write {
					offset: None,
					count: a,
				},
				Instruction::Write {
					offset: None,
					count: b,
				},
			] => Some(Change::replace(Instruction::write_many(a + b))),
			[
				Instruction::Write {
					count: a,
					offset: Some(x),
				},
				Instruction::Write {
					count: b,
					offset: Some(y),
				},
			] if *x == *y => Some(Change::replace(Instruction::write_many_at(*a + *b, x))),

			_ => None,
		}
	}

	fn should_run(&self, window: &[Instruction]) -> bool {
		matches!(
			window,
			[
				Instruction::IncVal { offset: None, .. },
				Instruction::IncVal { offset: None, .. }
			] | [Instruction::MovePtr(..), Instruction::MovePtr(..)]
				| [
					Instruction::SetVal { offset: None, .. },
					Instruction::SetVal { offset: None, .. }
				] | [
				Instruction::Write { offset: None, .. },
				Instruction::Write { offset: None, .. }
			]
		) || matches!(
			window,
			[
				Instruction::IncVal {
					offset: Some(x),
					..
				},
				Instruction::IncVal {
					offset: Some(y),
					..
				}
			] | [
				Instruction::SetVal {
					offset: Some(x),
					..
				},
				Instruction::SetVal {
					offset: Some(y),
					..
				}
			] | [
				Instruction::Write {
					offset: Some(x),
					..
				},
				Instruction::Write {
					offset: Some(y),
					..
				}
			]
			if *x == *y
		)
	}
}
