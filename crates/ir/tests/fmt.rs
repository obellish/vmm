use vmm_ir::Instruction::{self, *};

fn verify(input: Instruction, expected: &str) {
	assert_eq!(input.to_string(), expected);
}

#[test]
fn inc_val() {
	verify(IncVal(5), "+++++");
}

#[test]
fn move_ptr() {
	verify(MovePtr(3), ">>>");
}

#[test]
fn set_val() {
	verify(SetVal(0), "[-]");

	verify(SetVal(5), "[-]+++++");
}

#[test]
fn find_zero() {
	verify(FindZero(-9), "[<<<<<<<<<]");
}

#[test]
fn write() {
	verify(Write, ".");
}

#[test]
fn read() {
	verify(Read, ",");
}

#[test]
fn raw_loop() {
	verify(
		RawLoop(vec![IncVal(-1), MovePtr(1), IncVal(1), MovePtr(-1)]),
		"[->+<]",
	);
}
