use crate::Instruction::{self, *};

fn verify(input: Instruction, expected: &str) {
	assert_eq!(input.to_string(), expected);
}

#[test]
fn inc() {
	verify(Inc(5), "+++++");
}

#[test]
fn move_ptr() {
	verify(MovePtr(3), ">>>");
}

#[test]
fn set() {
	verify(Set(0), "[-]");

	verify(Set(5), "[-]+++++");
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
		RawLoop(vec![Inc(-1), MovePtr(1), Inc(1), MovePtr(-1)]),
		"[->+<]",
	);
}
