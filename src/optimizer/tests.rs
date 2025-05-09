use color_eyre::Result;

use crate::{
	Instruction::{self, *},
	Optimizer, OptimizerError, Program, Scanner,
	StackedInstruction::{self, *},
};

fn verify_optimizations<const N: usize>(
	input: Program,
	expected: [Instruction; N],
) -> Result<(), OptimizerError> {
	let mut optimizer = Optimizer::new(input);

	let optimized_program = optimizer.optimize()?;

	assert_eq!(*optimized_program, expected);

	Ok(())
}

fn verify_optimizations_raw<const N: usize>(inp: &str, expected: [Instruction; N]) -> Result<()> {
	let mut program = Scanner::new(inp).scan()?.collect::<Program>();

	verify_optimizations(program, expected)?;

	Ok(())
}

#[test]
fn setup_loop_unrolled() -> Result<()> {
	let program = Program::Raw(vec![
		SetVal(5).into(),
		RawLoop(vec![
			IncVal(-1).into(),
			MovePtr(2).into(),
			IncVal(2).into(),
			MovePtr(-2).into(),
		]),
	]);

	verify_optimizations(
		program,
		[MovePtr(2), IncVal(10), MovePtr(-2)].map(Into::into),
	)?;

	Ok(())
}
