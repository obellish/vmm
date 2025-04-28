use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug)]
pub struct CombineInstrPass;

impl PeepholePass for CombineInstrPass {
	const SIZE: usize = 2;

	#[tracing::instrument]
	fn run_pass(&self, window: &[Instruction]) -> Change {
		assert_eq!(window.len(), Self::SIZE);
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
			_ => Change::Ignore,
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("combine instructions")
	}
}

#[cfg(test)]
mod tests {
	use super::CombineInstrPass;
	use crate::{ExecutionUnit, Instruction, Pass as _, Program};

	#[test]
	fn replaces_with_better_instruction() {
		let instructions = [Instruction::Add(1), Instruction::Add(2)];

		let mut unit = ExecutionUnit::raw(instructions);

		assert!(CombineInstrPass.run_pass(&mut unit));

		assert_eq!(**unit.program(), [Instruction::Add(3)]);
	}
}
