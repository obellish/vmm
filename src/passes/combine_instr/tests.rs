use crate::{
	ExecutionUnit, Instruction, Pass, PeepholePass, Program,
	passes::{CombineAddInstrPass, CombineMoveInstrPass},
};

fn combine_instructions<P: PeepholePass, const SIZE: usize>(
	pass: P,
	instructions: [Instruction; SIZE],
	expected: Option<Instruction>,
) {
	assert_eq!(P::SIZE, SIZE);

	let mut unit = ExecutionUnit::raw(instructions);

	assert!(<P as Pass>::run_pass(&pass, unit.program_mut().as_raw()));

	assert_eq!(**unit.program(), expected.into_iter().collect::<Vec<_>>());
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
		[Instruction::Move(3), Instruction::Move(2)],
		Some(Instruction::Move(5)),
	);
}

#[test]
fn remove_move_instructions() {
	combine_instructions(
		CombineMoveInstrPass,
		[Instruction::Move(-2), Instruction::Move(2)],
		None,
	);
}
