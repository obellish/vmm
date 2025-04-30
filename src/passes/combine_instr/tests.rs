use crate::{
	ExecutionUnit, Instruction, Pass, PeepholePass, Program,
	passes::{CombineAddInstrPass, CombineMoveInstrPass, CombineZeroLoopInstrPass},
};

fn combine_instructions<P: PeepholePass, const SIZE: usize>(
	pass: P,
	instructions: [Instruction; SIZE],
	expected: Option<Instruction>,
) {
	assert_eq!(P::SIZE, SIZE);

	let mut unit = ExecutionUnit::raw(instructions);

	assert!(<P as Pass>::run_pass(&pass, &mut unit));

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

#[test]
fn combine_zero_loops_instructions() {
	combine_instructions(
		CombineZeroLoopInstrPass,
		[
			Instruction::JumpRight,
			Instruction::Add(-1),
			Instruction::JumpLeft,
		],
		Some(Instruction::Clear),
	);
}
