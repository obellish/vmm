use serde::{Deserialize, Serialize};

use super::Instruction;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profiler {
	pub add: u64,
	pub r#move: u64,
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
			add: 0,
			r#move: 0,
			while_loop: 0,
			input: 0,
			output: 0,
			set: 0,
			find_zero: 0,
			unknown: 0,
			clear: 0,
		}
	}

	pub const fn handle(&mut self, instruction: &Instruction) {
		match instruction {
			Instruction::SetVal(0) => self.clear += 1,
			Instruction::SetVal(_) => self.set += 1,
			Instruction::IncVal(_) => self.add += 1,
			Instruction::MovePtr(_) => self.r#move += 1,
			Instruction::Read => self.input += 1,
			Instruction::Write(x) => self.output += *x as u64,
			Instruction::FindZero { .. } => self.find_zero += 1,
			Instruction::RawLoop(_) => self.while_loop += 1,
			_ => self.unknown += 1,
		}
	}
}
