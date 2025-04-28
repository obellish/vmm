use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineInstrPass<const SIZE: usize>;

impl PeepholePass for CombineInstrPass<2> {
	const SIZE: usize = 2;

	#[tracing::instrument]
	fn run_pass(&self, window: &[Instruction]) -> Change {
		match (window[0], window[1]) {
			(Instruction::Add(i1), Instruction::Add(i2)) => {
				if i1 == -i2 {
					Change::Remove
				} else {
					Change::Replace(vec![Instruction::Add(i1 + i2)])
				}
			}
			(Instruction::Move(i1), Instruction::Move(i2)) => {
				if i1 == -i2 {
					Change::Remove
				} else {
					Change::Replace(vec![Instruction::Move(i1 + i2)])
				}
			}
			(Instruction::Clear, Instruction::Add(i)) => Change::Replace(vec![Instruction::Set(i as u8)]),
			_ => Change::Ignore,
		}
	}

	fn name(&self) -> Cow<'static, str> {
		name()
	}
}

impl PeepholePass for CombineInstrPass<3> {
	const SIZE: usize = 3;

	fn run_pass(&self, window: &[Instruction]) -> Change {
		match (window[0], window[1], window[2]) {
			(Instruction::JumpRight, Instruction::Add(-1), Instruction::JumpLeft) => {
				Change::Replace(vec![Instruction::Clear])
			}
			_ => Change::Ignore,
		}
	}

	fn name(&self) -> Cow<'static, str> {
		name()
	}
}

const fn name() -> Cow<'static, str> {
	Cow::Borrowed("combine instructions")
}

#[cfg(test)]
mod tests {
	use super::CombineInstrPass;
	use crate::{ExecutionUnit, Instruction, Pass as _, Program};

	#[test]
	fn replaces_with_better_instruction() {
		let instructions = [Instruction::Add(1), Instruction::Add(2)];

		let mut unit = ExecutionUnit::raw(instructions);

		assert!(CombineInstrPass::<2>.run_pass(&mut unit));

		assert_eq!(**unit.program(), [Instruction::Add(3)]);
	}
}
