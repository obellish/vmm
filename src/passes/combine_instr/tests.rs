use std::fmt::Debug;

use crate::{
	Instruction, Pass, PeepholePass, Program,
	passes::{CombineAddInstrPass, CombineMoveInstrPass},
};

fn combine_instructions<P, const SIZE: usize>(
	pass: P,
	instructions: [Instruction; SIZE],
	expected: Option<Instruction>,
) where
	P: Debug + PeepholePass,
{
	assert_eq!(P::SIZE, SIZE);

	// let mut unit = ExecutionUnit::raw(instructions);

	let mut program = Program::Raw(instructions.to_vec());

	// assert!(<P as Pass>::run_pass(&pass, unit.program_mut().as_raw()));

	assert!(<P as Pass>::run_pass(&pass, program.as_raw()));

	assert_eq!(&*program, expected.into_iter().collect::<Vec<_>>());
}

#[test]
fn combine_add_instructions() {
	combine_instructions(
		CombineAddInstrPass,
		[Instruction::Add(1), Instruction::Add(2)],
		Some(Instruction::Add(3)),
	);
}

#[test]
fn remove_add_instructions() {
	combine_instructions(
		CombineAddInstrPass,
		[Instruction::Add(-1), Instruction::Add(1)],
		None,
	);
}

#[test]
fn combine_move_instructions() {
	combine_instructions(
		CombineMoveInstrPass,
		[Instruction::MovePtr(3), Instruction::MovePtr(2)],
		Some(Instruction::MovePtr(5)),
	);
}

#[test]
fn remove_move_instructions() {
	combine_instructions(
		CombineMoveInstrPass,
		[Instruction::MovePtr(-2), Instruction::MovePtr(2)],
		None,
	);
}
