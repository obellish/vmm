use std::fmt::Display;

use crate::{
	Instruction::{self, *},
	StackedInstruction::{self, *},
};

fn verify(input: impl Display, expected: &str) {
	assert_eq!(input.to_string(), expected);
}

#[test]
fn inc() {
	verify(IncVal(5), "+++++");
}

#[test]
fn move_ptr() {
	verify(MovePtr(3), ">>>");
}

#[test]
fn set() {
	verify(SetVal(0), "[-]");

	verify(SetVal(5), "[-]+++++");
}

#[test]
fn find_zero() {
	verify(FindZero(-9), "[<<<<<<<<<]");
}

#[test]
fn write() {
	verify(Write(1), ".");
}

#[test]
fn read() {
	verify(Read, ",");
}

#[test]
fn raw_loop() {
	verify(
		RawLoop(vec![
			Stacked(IncVal(-1)),
			Stacked(MovePtr(1)),
			Stacked(IncVal(1)),
			Stacked(MovePtr(-1)),
		]),
		"[->+<]",
	);
}
