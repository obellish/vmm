pub trait MinimumOutputs: super::sealed::Sealed {
	fn min_outputs(&self) -> usize;
}

impl<T: MinimumOutputs> MinimumOutputs for [T] {
	fn min_outputs(&self) -> usize {
		self.iter().map(MinimumOutputs::min_outputs).sum()
	}
}

#[cfg(test)]
mod tests {
	use crate::{Instruction, MinimumOutputs};

	#[test]
	fn no_output() {
		assert_eq!(
			[Instruction::inc_val(2), Instruction::set_val(4)].min_outputs(),
			0
		);
	}

	#[test]
	fn basic_output() {
		assert_eq!(
			[Instruction::set_val(10), Instruction::write_once()].min_outputs(),
			1
		);
	}

	#[test]
	fn in_loop() {
		assert_eq!(
			[
				Instruction::write_once(),
				Instruction::dynamic_loop([Instruction::inc_val(-1), Instruction::write_once()])
			]
			.min_outputs(),
			2
		);
	}
}
