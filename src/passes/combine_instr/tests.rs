use std::fmt::Debug;

use crate::{
	Instruction, Pass, PeepholePass, Program,
	passes::{CombineIncValInstrPass, CombineMovePtrInstrPass, CombineSetInstrPass},
};

fn combine_instructions<P, const SIZE: usize>(
	mut pass: P,
	instructions: [Instruction; SIZE],
	expected: Option<Instruction>,
) where
	P: Debug + PeepholePass,
{
	assert_eq!(P::SIZE, SIZE);
	let mut program = Program::Raw(instructions.to_vec());

	assert!(<P as Pass>::run_pass(&mut pass, program.as_raw()));

	assert_eq!(&*program, expected.into_iter().collect::<Vec<_>>());
}

#[test]
fn combine_inc_instructions() {
	combine_instructions(
		CombineIncValInstrPass,
		[Instruction::IncVal(1), Instruction::IncVal(2)],
		Some(Instruction::IncVal(3)),
	);
}

#[test]
fn remove_inc_instructions() {
	combine_instructions(
		CombineIncValInstrPass,
		[Instruction::IncVal(-1), Instruction::IncVal(1)],
		None,
	);
}

#[test]
fn combine_move_instructions() {
	combine_instructions(
		CombineMovePtrInstrPass,
		[Instruction::MovePtr(3), Instruction::MovePtr(2)],
		Some(Instruction::MovePtr(5)),
	);
}

#[test]
fn remove_move_instructions() {
	combine_instructions(
		CombineMovePtrInstrPass,
		[Instruction::MovePtr(-2), Instruction::MovePtr(2)],
		None,
	);
}

#[test]
fn combine_clear_and_inc_instructions() {
	combine_instructions(
		CombineSetInstrPass,
		[Instruction::SetVal(0), Instruction::IncVal(5)],
		Some(Instruction::SetVal(5)),
	);
}
