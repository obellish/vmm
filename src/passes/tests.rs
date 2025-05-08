use color_eyre::Result;

use crate::{
	Instruction::{self, *},
	Pass, Program, Scanner,
	passes::*,
};

fn verify_pass_works<P: Pass, const N: usize>(
	mut pass: P,
	inp: &mut Vec<Instruction>,
	expected: [Instruction; N],
) {
	assert!(pass.run_pass(inp));

	assert_eq!(*inp, expected);
}

fn verify_pass_works_raw<P: Pass, const N: usize>(
	pass: P,
	inp: &str,
	expected: [Instruction; N],
) -> Result<()> {
	let mut program = Scanner::new(inp).scan()?.collect::<Program>();

	verify_pass_works(pass, program.as_raw(), expected);

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
		[RawLoop(vec![IncVal(-1)])],
	)
}

#[test]
fn set_untouched_cells_pass() {
	let mut program = vec![IncVal(3), Write];

	verify_pass_works(SetUntouchedCellsPass, &mut program, [SetVal(3), Write]);
}

#[test]
fn unroll_constant_loops_pass() {
	let mut program = vec![
		SetVal(5),
		RawLoop(vec![IncVal(-1), MovePtr(2), IncVal(2), MovePtr(-2)]),
	];

	verify_pass_works(
		UnrollConstantLoopsPass,
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
		],
	);
}
