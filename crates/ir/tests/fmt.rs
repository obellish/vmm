use vmm_ir::Instruction::{self, *};

fn verify(input: Instruction, expected: &str) {
	assert_eq!(input.to_string(), expected);
}

#[test]
fn inc_val() {
	verify(Instruction::inc_val(5), "+++++");
}

#[test]
fn move_ptr() {
	verify(Instruction::move_ptr_by(3), ">>>");
}

#[test]
fn set_val() {
	verify(Instruction::clear_val(), "[-]");

	verify(Instruction::set_val(5), "[-]+++++");
}

#[test]
fn find_zero() {
	verify(Instruction::find_zero(-9), "[<<<<<<<<<]");
}

#[test]
fn write() {
	verify(Instruction::write(), ".");
}

#[test]
fn read() {
	verify(Instruction::read(), ",");
}

#[test]
fn raw_loop() {
	verify(
		Instruction::raw_loop([
			Instruction::inc_val(-1),
			Instruction::move_ptr_by(1),
			Instruction::inc_val(1),
			Instruction::move_ptr_by(-1),
		]),
		"[->+<]",
	);
}
