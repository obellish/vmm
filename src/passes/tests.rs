use color_eyre::Result;

use crate::{
	Instruction::{self, *},
	Pass, Program, Scanner,
	StackedInstruction::*,
	passes::*,
};

fn verify_pass_works<P, const N: usize>(inp: &mut Vec<Instruction>, expected: [Instruction; N])
where
	P: Default + Pass,
{
	let mut pass = P::default();

	assert!(pass.run_pass(inp));

	assert_eq!(*inp, expected);
}

fn verify_pass_works_raw<P, const N: usize>(
	pass: P,
	inp: &str,
	expected: [Instruction; N],
) -> Result<()>
where
	P: Default + Pass,
{
	let mut program = Scanner::new(inp).scan()?.collect::<Program>();

	verify_pass_works::<P, N>(program.as_raw(), expected);

	Ok(())
}

#[test]
fn find_zero_pass() -> Result<()> {
	verify_pass_works_raw(FindZeroPass, "[>]", [FindZero(1)])
}

#[test]
fn remove_empty_loops_pass() -> Result<()> {
	verify_pass_works_raw(
		RemoveEmptyLoopsPass,
		"[][-][][]",
		[RawLoop(vec![IncVal(-1).into()])],
	)
}

#[test]
fn set_untouched_cells_pass() {
	let mut program = vec![IncVal(3).into(), Write(1).into()];

	verify_pass_works::<SetUntouchedCellsPass, 2>(
		&mut program,
		[SetVal(3).into(), Write(1).into()],
	);
}

#[test]
fn unroll_constant_loops_pass() {
	let mut program = vec![
		SetVal(5).into(),
		RawLoop(vec![
			IncVal(-1).into(),
			MovePtr(2).into(),
			IncVal(2).into(),
			MovePtr(-2).into(),
		]),
	];

	verify_pass_works::<UnrollConstantLoopsPass, 15>(
		&mut program,
		[
			MovePtr(2),
			IncVal(2),
			MovePtr(-2),
			MovePtr(2),
			IncVal(2),
			MovePtr(-2),
			MovePtr(2),
			IncVal(2),
			MovePtr(-2),
			MovePtr(2),
			IncVal(2),
			MovePtr(-2),
			MovePtr(2),
			IncVal(2),
			MovePtr(-2),
		]
		.map(Into::into),
	);
}
