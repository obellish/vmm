use serde::{Deserialize, Serialize};
use vmm_ir::Instruction;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profiler {
	pub inc_val: u64,
	pub move_ptr: u64,
	pub move_val: u64,
	pub while_loop: u64,
	pub input: u64,
	pub output: u64,
	pub set: u64,
	pub clear: u64,
	pub find_zero: u64,
	pub unknown: u64,
}

impl Profiler {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			inc_val: 0,
			move_ptr: 0,
			while_loop: 0,
			input: 0,
			output: 0,
			set: 0,
			find_zero: 0,
			unknown: 0,
			clear: 0,
			move_val: 0,
		}
	}

	pub const fn handle(&mut self, instruction: &Instruction) {
		match instruction {
			Instruction::IncVal { .. } => self.inc_val += 1,
			Instruction::MovePtr(_) => self.move_ptr += 1,
			Instruction::SetVal(0) => self.clear += 1,
			Instruction::SetVal(_) => self.set += 1,
			Instruction::Read => self.input += 1,
			Instruction::Write => self.output += 1,
			Instruction::FindZero(..) => self.find_zero += 1,
			Instruction::RawLoop(_) => self.while_loop += 1,
			Instruction::MoveVal { .. } => self.move_val += 1,
			_ => self.unknown += 1,
		}
	}
}
