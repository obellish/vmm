use serde::{Deserialize, Serialize};

use crate::Instruction;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profiler {
	pub add: u64,
	pub mov: u64,
	pub jr: u64,
	pub jl: u64,
	pub inp: u64,
	pub out: u64,
	pub set: u64,
	pub muz: u64,
}

impl Profiler {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			add: 0,
			mov: 0,
			jl: 0,
			jr: 0,
			inp: 0,
			out: 0,
			set: 0,
			muz: 0,
		}
	}

	pub const fn handle(&mut self, instruction: Instruction) {
		match instruction {
			Instruction::Set(_) => self.set += 1,
			Instruction::Add(_) => self.add += 1,
			Instruction::JumpLeft => self.jl += 1,
			Instruction::JumpRight => self.jr += 1,
			Instruction::Move(_) => self.mov += 1,
			Instruction::Read => self.inp += 1,
			Instruction::Write => self.out += 1,
			Instruction::JumpToZero {..} => self.muz += 1,
		}
	}
}
