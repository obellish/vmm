use color_eyre::Result;

use crate::{
	Instruction::{self, *},
	Optimizer, OptimizerError, Program, Scanner,
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
    let program = Program::Raw(vec![Set(5), RawLoop(vec![Inc(-1), MovePtr(2), Inc(2), MovePtr(-2)])]);

    verify_optimizations(program, [MovePtr(2), Inc(10), MovePtr(-2)])?;

    Ok(())
}
